//! 工作区主布局（左右停靠区、中央列、底部面板）渲染。

use gpui::{
    Context, Div, InteractiveElement, ParentElement, Stateful, StatefulInteractiveElement, Styled,
    Window, div, px, rgb,
};
use zom_protocol::{CommandInvocation, FocusTarget, NotificationAction};
use zom_runtime::{projection::shortcut_hint, state::PanelDock};

use super::super::{DEFAULT_BOTTOM_PANEL_HEIGHT, ZomRootView, dock_gap, splitter_hit_size};
use crate::{
    components::chip,
    theme::{color, size},
};

impl ZomRootView {
    /// 渲染工作区主行：左停靠区 + 中央列 + 右停靠区与分割线。
    pub(super) fn render_workspace_row(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let left_target = self.state.visible_panel_in_dock(PanelDock::Left);
        let right_target = self.state.visible_panel_in_dock(PanelDock::Right);
        let bottom_target = self.state.visible_panel_in_dock(PanelDock::Bottom);

        self.restore_dock_sizes_when_visible(left_target, right_target, bottom_target);

        let workspace_width: f32 = window.viewport_size().width.into();
        let viewport_height: f32 = window.viewport_size().height.into();
        let workspace_height = ZomRootView::workspace_height_from_viewport(viewport_height);
        self.normalize_dock_widths(
            workspace_width,
            left_target.is_some(),
            right_target.is_some(),
        );
        self.normalize_bottom_panel_height(workspace_height);

        let left_width = if left_target.is_some() {
            self.left_dock_width
        } else {
            0.0
        };
        let right_width = if right_target.is_some() {
            self.right_dock_width
        } else {
            0.0
        };
        let center_width = (workspace_width - left_width - right_width).max(0.0);
        let bottom_hidden_by_overlap = center_width <= dock_gap();
        let is_bottom_panel_visible = bottom_target.is_some() && !bottom_hidden_by_overlap;
        let splitter_size = splitter_hit_size();

        let mut workspace_row = div()
            .id("workspace-row")
            .relative()
            .flex()
            .flex_1()
            .overflow_hidden();

        if let Some(target) = left_target {
            workspace_row = workspace_row.child(self.render_left_dock(target, left_width));
        }

        workspace_row = workspace_row.child(self.render_center_column(
            bottom_target,
            is_bottom_panel_visible,
            splitter_size,
            cx,
        ));

        if let Some(target) = right_target {
            workspace_row = workspace_row.child(self.render_right_dock(target, right_width, cx));
        }

        if left_target.is_some() {
            workspace_row =
                workspace_row.child(self.render_left_splitter(left_width, splitter_size, cx));
        }

        if right_target.is_some() {
            workspace_row = workspace_row.child(self.render_right_splitter(
                workspace_width,
                right_width,
                splitter_size,
                cx,
            ));
        }

        workspace_row
    }

    fn restore_dock_sizes_when_visible(
        &mut self,
        left_target: Option<FocusTarget>,
        right_target: Option<FocusTarget>,
        bottom_target: Option<FocusTarget>,
    ) {
        if left_target.is_some() && self.left_dock_width <= 0.0 {
            self.left_dock_width = size::PANEL_WIDTH;
        }
        if right_target.is_some() && self.right_dock_width <= 0.0 {
            self.right_dock_width = size::PANEL_WIDTH;
        }
        if bottom_target.is_some() && self.bottom_panel_height <= 0.0 {
            self.bottom_panel_height = DEFAULT_BOTTOM_PANEL_HEIGHT;
        }
    }

    fn render_left_dock(&self, target: FocusTarget, left_width: f32) -> Stateful<Div> {
        let mut left_dock = div()
            .id("workspace-left-dock")
            .w(px(left_width))
            .h_full()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(rgb(color::COLOR_BORDER))
            .overflow_hidden();

        left_dock = match target {
            FocusTarget::FileTreePanel => left_dock.child(self.file_tree_panel.clone()),
            FocusTarget::GitPanel => left_dock.child(self.git_panel.clone()),
            FocusTarget::OutlinePanel => left_dock.child(self.outline_panel.clone()),
            FocusTarget::ProjectSearchPanel => left_dock.child(self.project_search_panel.clone()),
            FocusTarget::LanguageServersPanel => {
                left_dock.child(self.language_servers_panel.clone())
            }
            _ => left_dock,
        };

        left_dock
    }

    fn render_center_column(
        &mut self,
        bottom_target: Option<FocusTarget>,
        is_bottom_panel_visible: bool,
        splitter_size: f32,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut center_column = div()
            .id("workspace-center-column")
            .relative()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .overflow_hidden();

        if is_bottom_panel_visible {
            let bottom_height = self.bottom_panel_height.max(0.0);
            let mut bottom_dock = div()
                .id("workspace-bottom-dock")
                .w_full()
                .h(px(bottom_height))
                .flex_shrink_0()
                .border_t_1()
                .border_color(rgb(color::COLOR_BORDER))
                .overflow_hidden();
            if let Some(target) = bottom_target {
                bottom_dock = match target {
                    FocusTarget::TerminalPanel => bottom_dock.child(self.terminal_panel.clone()),
                    FocusTarget::DebugPanel => bottom_dock.child(self.debug_panel.clone()),
                    _ => bottom_dock,
                };
            }

            center_column = center_column
                .child(
                    div()
                        .id("workspace-editor-area")
                        .flex_1()
                        .flex()
                        .flex_col()
                        .overflow_hidden()
                        .child(self.pane_view.clone()),
                )
                .child(bottom_dock)
                .child(self.render_bottom_splitter(bottom_height, splitter_size, cx));
        } else {
            center_column = center_column.child(
                div()
                    .id("workspace-editor-area")
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(self.pane_view.clone()),
            );
        }

        center_column
    }

    fn render_right_dock(
        &mut self,
        target: FocusTarget,
        right_width: f32,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut right_dock = div()
            .id("workspace-right-dock")
            .w(px(right_width))
            .h_full()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(rgb(color::COLOR_BORDER))
            .overflow_hidden();

        if target == FocusTarget::NotificationPanel {
            right_dock = right_dock.child(self.render_notification_panel_with_toolbar(cx));
        }

        right_dock
    }

    fn render_notification_panel_with_toolbar(&mut self, cx: &mut Context<Self>) -> Stateful<Div> {
        let unread_error_count = self
            .state
            .notifications
            .iter()
            .filter(|item| {
                !item.is_read && item.level == zom_runtime::state::DesktopNotificationLevel::Error
            })
            .count();
        let read_count = self
            .state
            .notifications
            .iter()
            .filter(|item| item.is_read)
            .count();
        let total_count = self.state.notifications.len();
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
                                cx,
                            ))
                            .child(self.render_notification_action_chip(
                                "notification-chip-clear-read",
                                format!("清空已读 {read_count}"),
                                NotificationAction::ClearRead,
                                cx,
                            ))
                            .child(self.render_notification_action_chip(
                                "notification-chip-clear-all",
                                format!("清空全部 {total_count}"),
                                NotificationAction::ClearAll,
                                cx,
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

    fn render_notification_action_chip(
        &self,
        id: &'static str,
        label: String,
        action: NotificationAction,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let command = CommandInvocation::from(action);
        let label_for_tooltip = label.clone();
        let label_for_chip = label.clone();
        chip::interactive_chip(
            id,
            chip::TooltipSpec::new(label_for_tooltip, shortcut_hint(&command)),
        )
        .px(px(size::GAP_1))
        .py(px(size::GAP_1))
        .border_1()
        .border_color(rgb(color::COLOR_BORDER))
        .rounded_sm()
        .text_xs()
        .text_color(rgb(color::COLOR_FG_MUTED))
        .on_click(cx.listener(move |this, _event, _window, cx| {
            this.state.dispatch_command(CommandInvocation::from(action));
            this.sync_notification_panel(cx);
            cx.notify();
        }))
        .child(label_for_chip)
    }
}
