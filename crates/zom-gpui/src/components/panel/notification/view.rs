//! 通知面板视图。

use gpui::{
    AnyElement, App, Context, FocusHandle, Focusable, ParentElement, Render, ScrollHandle, Styled,
    Window, div, prelude::*, px, rgb,
};
use zom_runtime::state::{
    DesktopNotification, DesktopNotificationLevel, DesktopNotificationSource,
};

use crate::{
    components::panel::shell,
    theme::{color, size},
};

/// 通知面板。
pub(crate) struct NotificationPanel {
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    notifications: Vec<DesktopNotification>,
    selected_notification_id: Option<u64>,
    is_logically_focused: bool,
    pending_scroll_to_selection: bool,
}

impl NotificationPanel {
    /// 创建通知面板。
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            notifications: Vec::new(),
            selected_notification_id: None,
            is_logically_focused: false,
            pending_scroll_to_selection: false,
        }
    }

    /// 用最新通知列表与焦点状态刷新通知面板。
    pub(crate) fn set_state(
        &mut self,
        notifications: Vec<DesktopNotification>,
        is_logically_focused: bool,
        preferred_selected_id: Option<u64>,
        cx: &mut Context<Self>,
    ) {
        let focus_gained = !self.is_logically_focused && is_logically_focused;
        self.is_logically_focused = is_logically_focused;
        self.notifications = notifications;
        let previous_selected = self.selected_notification_id;
        self.selected_notification_id = selected_notification_id(
            previous_selected,
            preferred_selected_id,
            &self.notifications,
        );
        if self.is_logically_focused
            && (focus_gained || previous_selected != self.selected_notification_id)
        {
            self.pending_scroll_to_selection = true;
        }
        cx.notify();
    }
}

impl Focusable for NotificationPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotificationPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let selected_row_index = self
            .notifications
            .iter()
            .rev()
            .position(|notification| self.selected_notification_id == Some(notification.id));
        if self.pending_scroll_to_selection {
            if let Some(selected_row_index) = selected_row_index {
                self.scroll_handle.scroll_to_item(selected_row_index);
            }
            self.pending_scroll_to_selection = false;
        }

        let body = if self.notifications.is_empty() {
            render_empty_placeholder().into_any_element()
        } else {
            let selected_notification_id = self.selected_notification_id;
            let panel_has_focus = self.is_logically_focused;
            div()
                .id("notification-panel-scroll")
                .size_full()
                .flex()
                .flex_col()
                .overflow_scroll()
                .track_scroll(&self.scroll_handle)
                .bg(rgb(color::COLOR_BG_PANEL))
                .gap(px(size::GAP_1))
                .px(px(size::GAP_1))
                .py(px(size::GAP_1))
                .children(self.notifications.iter().rev().map(|notification| {
                    let is_selected = selected_notification_id == Some(notification.id);
                    render_notification_item(notification, panel_has_focus && is_selected)
                }))
                .into_any_element()
        };

        shell::render_shell("notification-panel-container", &self.focus_handle, body)
    }
}

fn render_empty_placeholder() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .text_color(rgb(color::COLOR_FG_MUTED))
        .child("暂无通知")
}

fn render_notification_item(
    notification: &DesktopNotification,
    focus_emphasis: bool,
) -> AnyElement {
    let badge = level_badge(notification.level);
    let source = source_badge(notification.source);
    let item_id = gpui::SharedString::from(format!("notification-item-{}", notification.id));
    let (level_fg_color, unread_bg_color) = level_palette(notification.level);
    let is_unread = !notification.is_read;
    let border_color = if focus_emphasis {
        color::COLOR_FG_PRIMARY
    } else if is_unread {
        level_fg_color
    } else {
        color::COLOR_BORDER
    };
    let background_color = if is_unread {
        unread_bg_color
    } else {
        color::COLOR_BG_ELEMENT
    };
    let meta_text_color = if is_unread {
        color::COLOR_FG_PRIMARY
    } else {
        color::COLOR_FG_MUTED
    };
    let message_text_color = if is_unread {
        color::COLOR_FG_PRIMARY
    } else {
        color::COLOR_FG_MUTED
    };
    let count_suffix = if notification.occurrence_count > 1 {
        format!(" x{}", notification.occurrence_count)
    } else {
        String::new()
    };

    div()
        .id(item_id)
        .w_full()
        .flex()
        .flex_col()
        .gap(px(size::GAP_1))
        .p(px(size::GAP_1))
        .bg(rgb(background_color))
        .border_1()
        .border_color(rgb(border_color))
        .rounded_sm()
        .child(
            div()
                .text_xs()
                .text_color(rgb(meta_text_color))
                .child(format!("{badge} · {source}{count_suffix}")),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(message_text_color))
                .child(notification.message.clone()),
        )
        .into_any_element()
}

fn source_badge(source: DesktopNotificationSource) -> &'static str {
    match source {
        DesktopNotificationSource::Workspace => "WORKSPACE",
        DesktopNotificationSource::System => "SYSTEM",
        DesktopNotificationSource::Debug => "DEBUG",
    }
}

fn level_badge(level: DesktopNotificationLevel) -> &'static str {
    match level {
        DesktopNotificationLevel::Info => "INFO",
        DesktopNotificationLevel::Warning => "WARN",
        DesktopNotificationLevel::Error => "ERROR",
    }
}

fn level_palette(level: DesktopNotificationLevel) -> (u32, u32) {
    match level {
        DesktopNotificationLevel::Info => (0x58A6FF, 0x1A2433),
        DesktopNotificationLevel::Warning => (0xD29922, 0x2A230F),
        DesktopNotificationLevel::Error => (0xF85149, 0x32191D),
    }
}

fn selected_notification_id(
    current_selected_id: Option<u64>,
    preferred_selected_id: Option<u64>,
    notifications: &[DesktopNotification],
) -> Option<u64> {
    if let Some(preferred_id) = preferred_selected_id.filter(|id| {
        notifications
            .iter()
            .any(|notification| notification.id == *id)
    }) {
        return Some(preferred_id);
    }

    current_selected_id
        .filter(|id| {
            notifications
                .iter()
                .any(|notification| notification.id == *id)
        })
        .or_else(|| notifications.last().map(|notification| notification.id))
}

#[cfg(test)]
mod tests {
    use zom_runtime::state::{
        DesktopNotification, DesktopNotificationLevel, DesktopNotificationSource,
    };

    use super::selected_notification_id;

    fn notification(id: u64) -> DesktopNotification {
        DesktopNotification {
            id,
            level: DesktopNotificationLevel::Info,
            source: DesktopNotificationSource::System,
            message: format!("message-{id}"),
            created_at_ms: 1,
            updated_at_ms: 1,
            is_read: false,
            dedupe_key: None,
            occurrence_count: 1,
        }
    }

    #[test]
    fn selected_notification_defaults_to_latest_when_none_selected() {
        let notifications = vec![notification(1), notification(2), notification(3)];

        assert_eq!(
            selected_notification_id(None, None, &notifications),
            Some(3)
        );
    }

    #[test]
    fn selected_notification_keeps_current_when_still_present() {
        let notifications = vec![notification(1), notification(2), notification(3)];

        assert_eq!(
            selected_notification_id(Some(2), None, &notifications),
            Some(2)
        );
    }

    #[test]
    fn selected_notification_falls_back_to_latest_when_current_missing() {
        let notifications = vec![notification(11), notification(12)];

        assert_eq!(
            selected_notification_id(Some(10), None, &notifications),
            Some(12)
        );
    }

    #[test]
    fn selected_notification_prefers_runtime_target_when_present() {
        let notifications = vec![notification(21), notification(22), notification(23)];

        assert_eq!(
            selected_notification_id(Some(22), Some(23), &notifications),
            Some(23)
        );
    }
}
