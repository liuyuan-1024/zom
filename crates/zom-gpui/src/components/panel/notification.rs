//! 通知面板视图。

use gpui::{
    AnyElement, App, Context, Entity, FocusHandle, Focusable, ParentElement, Render, ScrollHandle,
    Styled, Window, div, prelude::*, px, rgb,
};
use zom_runtime::state::{
    DesktopNotification, DesktopNotificationLevel, DesktopNotificationSource,
};

use crate::{
    components::panel::shell,
    root_view::store::AppStore,
    theme::{color, size},
};

/// 通知面板视图状态，维护选中项与滚动定位以支持键盘导航。
pub(crate) struct NotificationPanel {
    store: Entity<AppStore>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    selected_notification_id: Option<u64>,
    should_scroll_to_selection: bool,
}

impl NotificationPanel {
    /// 创建通知面板并订阅通知列表变化。
    /// 每次更新会重算选中项与滚动目标，保证键盘导航与可见性一致。
    pub(crate) fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        cx.observe(&store, |this, store, cx| {
            let notifications = store.read(cx).select_notifications();
            let preferred_selected_id = store.update(cx, |store, _cx| {
                store.take_pending_notification_selection_id()
            });
            let previous_selected = this.selected_notification_id;
            this.selected_notification_id =
                selected_notification_id(previous_selected, preferred_selected_id, &notifications);
            let is_logically_focused = store.read(cx).select_focused_target()
                == zom_protocol::FocusTarget::NotificationPanel;
            if is_logically_focused && previous_selected != this.selected_notification_id {
                this.should_scroll_to_selection = true;
            }
            cx.notify();
        })
        .detach();

        Self {
            store,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            selected_notification_id: None,
            should_scroll_to_selection: false,
        }
    }
}

impl Focusable for NotificationPanel {
    /// 返回当前组件的焦点句柄，用于键盘焦点路由。
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotificationPanel {
    /// 渲染通知列表并维护“选中项可见”滚动约束，空列表时回落到占位文案。
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let notifications = self.store.read(cx).select_notifications();
        let is_panel_focused = self.store.read(cx).select_focused_target()
            == zom_protocol::FocusTarget::NotificationPanel;

        let selected_row_index = notifications
            .iter()
            .rev()
            .position(|notification| self.selected_notification_id == Some(notification.id));
        if self.should_scroll_to_selection {
            if let Some(selected_row_index) = selected_row_index {
                self.scroll_handle.scroll_to_item(selected_row_index);
            }
            self.should_scroll_to_selection = false;
        }

        let body = if notifications.is_empty() {
            render_empty_placeholder().into_any_element()
        } else {
            let selected_notification_id = self.selected_notification_id;
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
                .children(notifications.iter().rev().map(|notification| {
                    let is_selected = selected_notification_id == Some(notification.id);
                    render_notification_item(notification, is_panel_focused && is_selected)
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

/// 渲染通知并组装对应界面节点。
fn render_notification_item(
    notification: &DesktopNotification,
    has_focus_emphasis: bool,
) -> AnyElement {
    let badge = level_badge(notification.level);
    let source = source_badge(notification.source);
    let item_id = gpui::SharedString::from(format!("notification-item-{}", notification.id));
    let (level_fg_color, unread_bg_color) = level_palette(notification.level);
    let is_unread = !notification.is_read;
    let border_color = if has_focus_emphasis {
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

/// 当通知来源进入 UI 时，转成紧凑标签用于元信息行展示。
fn source_badge(source: DesktopNotificationSource) -> &'static str {
    match source {
        DesktopNotificationSource::Workspace => "WORKSPACE",
        DesktopNotificationSource::System => "SYSTEM",
        DesktopNotificationSource::Debug => "DEBUG",
    }
}

/// 把通知等级映射为统一短标签，保证列表与 toast 文案一致。
fn level_badge(level: DesktopNotificationLevel) -> &'static str {
    match level {
        DesktopNotificationLevel::Info => "INFO",
        DesktopNotificationLevel::Warning => "WARN",
        DesktopNotificationLevel::Error => "ERROR",
    }
}

/// 返回等级主题色：前景强调色 + 未读背景色。
fn level_palette(level: DesktopNotificationLevel) -> (u32, u32) {
    match level {
        DesktopNotificationLevel::Info => (0x58A6FF, 0x1A2433),
        DesktopNotificationLevel::Warning => (0xD29922, 0x2A230F),
        DesktopNotificationLevel::Error => (0xF85149, 0x32191D),
    }
}

/// 解析当前应选中的通知 id。
///
/// 优先级：`preferred_selected_id`（若仍存在） > `current_selected_id` > 最新一条通知。
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

    if let Some(current_id) = current_selected_id
        && notifications
            .iter()
            .any(|notification| notification.id == current_id)
    {
        return Some(current_id);
    }

    notifications.last().map(|notification| notification.id)
}
