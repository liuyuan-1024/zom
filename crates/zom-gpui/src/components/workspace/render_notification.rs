//! `render_notification` 模块，负责 当前 域相关能力与数据组织。
use gpui::{Context, Div, InteractiveElement, ParentElement, Stateful, Styled, div, px, rgb};
use zom_input::shortcut_hint;
use zom_protocol::{CommandInvocation, NotificationAction};
use zom_runtime::state::DesktopNotificationLevel;

use super::WorkspaceView;
use crate::{
    components::chip::Chip,
    theme::{color, size},
};

impl WorkspaceView {
    /// 渲染通知面板工具栏并组装对应界面节点。
    pub(super) fn render_notification_panel_with_toolbar(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let (unread_error_count, read_count, total_count) = {
            let store = self.store.read(cx);
            let state = store.select_core();
            let unread_error_count = state
                .notifications
                .iter()
                .filter(|item| !item.is_read && item.level == DesktopNotificationLevel::Error)
                .count();
            let read_count = state
                .notifications
                .iter()
                .filter(|item| item.is_read)
                .count();
            let total_count = state.notifications.len();
            (unread_error_count, read_count, total_count)
        };
        div()
            .id("notification-panel-shell")
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("notification-panel-toolbar")
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap(px(size::GAP_1))
                    .px(px(size::GAP_1))
                    .py(px(size::GAP_1))
                    .border_b_1()
                    .border_color(rgb(color::COLOR_BORDER))
                    .bg(rgb(color::COLOR_BG_PANEL))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(size::GAP_1))
                            .child(self.render_notification_action_chip(
                                "notification-chip-focus-unread-error",
                                format!("未读错误 {unread_error_count}"),
                                NotificationAction::FocusUnreadError,
                            ))
                            .child(self.render_notification_action_chip(
                                "notification-chip-clear-read",
                                format!("清空已读 {read_count}"),
                                NotificationAction::ClearRead,
                            ))
                            .child(self.render_notification_action_chip(
                                "notification-chip-clear-all",
                                format!("清空全部 {total_count}"),
                                NotificationAction::ClearAll,
                            )),
                    ),
            )
            .child(
                div()
                    .id("notification-panel-content")
                    .flex_1()
                    .overflow_hidden()
                    .child(self.notification_panel.clone()),
            )
    }

    /// 渲染通知并组装对应界面节点。
    fn render_notification_action_chip(
        &self,
        id: &'static str,
        label: String,
        action: NotificationAction,
    ) -> impl gpui::IntoElement {
        let command = CommandInvocation::from(action);
        let label_for_tooltip = label.clone();
        let label_for_chip = label;
        Chip::new(id)
            .label(label_for_chip)
            .tooltip_hint(label_for_tooltip, shortcut_hint(&command))
    }
}
