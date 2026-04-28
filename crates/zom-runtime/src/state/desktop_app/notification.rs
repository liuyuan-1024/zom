//! 通知写入与悬浮提示同步逻辑。

use std::time::{SystemTime, UNIX_EPOCH};

use zom_protocol::FocusTarget;

use super::{
    DesktopAppState, DesktopNotification, DesktopNotificationEvent, DesktopNotificationKind,
    DesktopNotificationLevel, DesktopNotificationSource,
};

const MAX_NOTIFICATION_HISTORY: usize = 200;
const DEDUPE_WINDOW_MS: u128 = 3_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NotificationChannels {
    toast: bool,
    panel: bool,
    status_bar: bool,
}

impl DesktopAppState {
    /// 依据事件语义分发通知通道并完成落库/聚合。
    /// 返回写入或更新后的通知 id（若未写入面板则为 `None`）。
    pub fn publish_notification_event(&mut self, event: DesktopNotificationEvent) -> Option<u64> {
        let channels = channels_for_event(&event);
        let now_ms = now_ms();
        let mut should_emit_toast = channels.toast;
        let mut panel_notification_id = None;

        if channels.panel {
            if let Some(existing_index) = self.find_dedupe_candidate(
                event.level,
                event.source,
                event.dedupe_key.as_deref(),
                now_ms,
            ) {
                let mut notification = self.notifications.remove(existing_index);
                notification.updated_at_ms = now_ms;
                notification.occurrence_count = notification.occurrence_count.saturating_add(1);
                notification.is_read = false;
                let notification_id = notification.id;
                self.notifications.push(notification.clone());
                panel_notification_id = Some(notification_id);
                self.unread_notification_count = self.unread_notification_count.saturating_add(1);
                if channels.status_bar {
                    self.active_status_notification = Some(notification);
                }
                // 聚合后不重复弹 toast，避免短时间连环打断。
                should_emit_toast = false;
            } else {
                let id = self.next_notification_id;
                self.next_notification_id = self.next_notification_id.saturating_add(1);
                let notification = DesktopNotification {
                    id,
                    level: event.level,
                    source: event.source,
                    message: event.message,
                    created_at_ms: now_ms,
                    updated_at_ms: now_ms,
                    is_read: false,
                    dedupe_key: event.dedupe_key,
                    occurrence_count: 1,
                };
                self.notifications.push(notification.clone());
                if self.notifications.len() > MAX_NOTIFICATION_HISTORY {
                    let overflow = self.notifications.len() - MAX_NOTIFICATION_HISTORY;
                    let removed_unread = self
                        .notifications
                        .iter()
                        .take(overflow)
                        .filter(|item| !item.is_read)
                        .count();
                    self.notifications.drain(0..overflow);
                    self.unread_notification_count = self
                        .unread_notification_count
                        .saturating_sub(removed_unread);
                }
                self.unread_notification_count = self.unread_notification_count.saturating_add(1);
                panel_notification_id = Some(id);
                if channels.status_bar {
                    self.active_status_notification = Some(notification);
                }
            }
        } else if channels.status_bar {
            self.active_status_notification = Some(DesktopNotification {
                id: 0,
                level: event.level,
                source: event.source,
                message: event.message,
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
                is_read: true,
                dedupe_key: event.dedupe_key,
                occurrence_count: 1,
            });
        }

        if should_emit_toast {
            if let Some(notification_id) = panel_notification_id {
                self.active_toast_notification = self
                    .notifications
                    .iter()
                    .find(|notification| notification.id == notification_id)
                    .cloned();
            } else {
                self.active_toast_notification = self.active_status_notification.clone();
            }
        }

        panel_notification_id
    }

    /// 追加一条通知：写入通知侧边栏并刷新当前悬浮提示。
    pub fn push_notification(
        &mut self,
        level: DesktopNotificationLevel,
        message: impl Into<String>,
    ) -> u64 {
        self.publish_notification_event(DesktopNotificationEvent::new(
            level,
            DesktopNotificationSource::System,
            message,
        ))
        .unwrap_or(0)
    }

    /// 标记全部通知为已读，并同步未读计数。
    pub fn mark_all_notifications_read(&mut self) {
        for notification in &mut self.notifications {
            notification.is_read = true;
        }
        self.unread_notification_count = 0;
    }

    /// 清空通知历史与状态栏提示。
    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
        self.active_toast_notification = None;
        self.active_status_notification = None;
        self.unread_notification_count = 0;
        self.selected_notification_id = None;
        self.pending_notification_selection_id = None;
    }

    /// 清空已读通知，保留未读项。
    pub fn clear_read_notifications(&mut self) {
        self.notifications
            .retain(|notification| !notification.is_read);
        self.unread_notification_count = self.notifications.len();
        if let Some(selected_id) = self.selected_notification_id {
            let still_exists = self
                .notifications
                .iter()
                .any(|notification| notification.id == selected_id);
            if !still_exists {
                self.selected_notification_id = self.notifications.last().map(|item| item.id);
                self.pending_notification_selection_id = self.selected_notification_id;
            }
        }
        if let Some(active_toast_id) = self.active_toast_notification.as_ref().map(|item| item.id) {
            let still_exists = self
                .notifications
                .iter()
                .any(|notification| notification.id == active_toast_id);
            if !still_exists {
                self.active_toast_notification = None;
            }
        }
    }

    /// 聚焦通知面板并优先选中最近的未读错误通知。
    pub fn focus_unread_error_notification(&mut self) {
        self.focus_panel(FocusTarget::NotificationPanel);
        self.selected_notification_id = self
            .notifications
            .iter()
            .rev()
            .find(|notification| {
                notification.level == DesktopNotificationLevel::Error && !notification.is_read
            })
            .map(|notification| notification.id)
            .or_else(|| {
                self.notifications
                    .last()
                    .map(|notification| notification.id)
            });
        self.pending_notification_selection_id = self.selected_notification_id;
    }

    /// 选择通知面板上一条通知（视觉向上）。
    pub fn select_prev_notification(&mut self) {
        self.shift_notification_selection(-1);
    }

    /// 选择通知面板下一条通知（视觉向下）。
    pub fn select_next_notification(&mut self) {
        self.shift_notification_selection(1);
    }

    /// 清空当前悬浮提示。
    pub fn clear_active_toast_notification(&mut self) {
        self.active_toast_notification = None;
    }

    fn find_dedupe_candidate(
        &self,
        level: DesktopNotificationLevel,
        source: DesktopNotificationSource,
        dedupe_key: Option<&str>,
        now_ms: u128,
    ) -> Option<usize> {
        let dedupe_key = dedupe_key?;
        self.notifications.iter().rposition(|notification| {
            notification.level == level
                && notification.source == source
                && notification.dedupe_key.as_deref() == Some(dedupe_key)
                && now_ms.saturating_sub(notification.updated_at_ms) <= DEDUPE_WINDOW_MS
        })
    }

    fn shift_notification_selection(&mut self, step: i8) {
        if self.notifications.is_empty() {
            self.selected_notification_id = None;
            self.pending_notification_selection_id = None;
            return;
        }
        let ids: Vec<u64> = self
            .notifications
            .iter()
            .rev()
            .map(|item| item.id)
            .collect();
        let current_index = self
            .selected_notification_id
            .and_then(|selected_id| ids.iter().position(|id| *id == selected_id))
            .unwrap_or(0);
        let next_index = if step < 0 {
            current_index.saturating_sub(step.unsigned_abs() as usize)
        } else {
            let max_index = ids.len().saturating_sub(1);
            (current_index + step as usize).min(max_index)
        };
        let next_selected_id = Some(ids[next_index]);
        self.selected_notification_id = next_selected_id;
        self.pending_notification_selection_id = next_selected_id;
    }
}

fn channels_for_event(event: &DesktopNotificationEvent) -> NotificationChannels {
    match event.kind {
        DesktopNotificationKind::Progress => NotificationChannels {
            toast: false,
            panel: false,
            status_bar: true,
        },
        DesktopNotificationKind::General => match event.level {
            DesktopNotificationLevel::Error => NotificationChannels {
                toast: true,
                panel: true,
                status_bar: true,
            },
            DesktopNotificationLevel::Warning => NotificationChannels {
                toast: true,
                panel: true,
                status_bar: false,
            },
            DesktopNotificationLevel::Info => NotificationChannels {
                toast: event.user_initiated,
                panel: true,
                status_bar: false,
            },
        },
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
