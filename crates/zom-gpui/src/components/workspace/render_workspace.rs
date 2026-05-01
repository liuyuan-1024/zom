//! `render_workspace` 模块，负责 当前 域相关能力与数据组织。
use gpui::{
    Context, CursorStyle, Div, InteractiveElement, ParentElement, Stateful, Styled, Window, div,
    px, rgb,
};
use zom_protocol::{FocusTarget, OverlayTarget};

use super::splitters::dock_gap;
use super::{WorkspaceView, splitters::splitter_hit_size, view::ActiveDockDrag};
use crate::theme::color;

impl WorkspaceView {
    /// 渲染工作区并组装对应界面节点。
    pub(super) fn render_workspace(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let (left_target, right_target, bottom_target, active_overlay) = {
            let store = self.store.read(cx);
            (
                store.select_visible_panel_in_dock(zom_runtime::state::PanelDock::Left),
                store.select_visible_panel_in_dock(zom_runtime::state::PanelDock::Right),
                store.select_visible_panel_in_dock(zom_runtime::state::PanelDock::Bottom),
                store.select_active_overlay(),
            )
        };

        self.restore_dock_sizes_when_visible(left_target, right_target, bottom_target);

        let workspace_width: f32 = window.viewport_size().width.into();
        let viewport_height: f32 = window.viewport_size().height.into();
        let workspace_height = Self::workspace_height_from_viewport(viewport_height);
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
            .overflow_hidden()
            .cursor(match self.active_dock_drag {
                Some(ActiveDockDrag::Bottom { .. }) => CursorStyle::ResizeUpDown,
                Some(ActiveDockDrag::Left | ActiveDockDrag::Right) => CursorStyle::ResizeLeftRight,
                None => CursorStyle::Arrow,
            })
            .capture_any_mouse_up(cx.listener(|this, _event, _window, cx| {
                if this.active_dock_drag.is_some() {
                    this.active_dock_drag = None;
                    cx.notify();
                }
            }))
            .on_mouse_move(cx.listener(|this, event, window, cx| {
                let workspace_width: f32 = window.viewport_size().width.into();
                let viewport_height: f32 = window.viewport_size().height.into();
                let workspace_height = Self::workspace_height_from_viewport(viewport_height);
                this.on_drag_mouse_move(event, workspace_width, workspace_height, cx);
            }));

        if let Some(target) = left_target {
            workspace_row = workspace_row.child(self.render_left_dock(target, left_width));
        }

        workspace_row = workspace_row.child(self.render_center_column(
            bottom_target,
            is_bottom_panel_visible,
            splitter_size,
            active_overlay,
            cx,
        ));

        if let Some(target) = right_target {
            workspace_row = workspace_row.child(self.render_right_dock(target, right_width));
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

    /// 渲染停靠区并组装对应界面节点。
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

    /// 渲染列并组装对应界面节点。
    fn render_center_column(
        &mut self,
        bottom_target: Option<FocusTarget>,
        is_bottom_panel_visible: bool,
        splitter_size: f32,
        active_overlay: Option<OverlayTarget>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut editor_area = div()
            .id("workspace-editor-area")
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden();
        if active_overlay == Some(OverlayTarget::FindReplace) {
            editor_area = editor_area.child(self.render_find_replace_bar(cx));
        }
        editor_area = editor_area.child(self.pane_view.clone());

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
                .child(editor_area)
                .child(bottom_dock)
                .child(self.render_bottom_splitter(bottom_height, splitter_size, cx));
        } else {
            center_column = center_column.child(editor_area);
        }

        center_column
    }

    /// 渲染停靠区并组装对应界面节点。
    fn render_right_dock(&self, target: FocusTarget, right_width: f32) -> Stateful<Div> {
        let mut right_dock = div()
            .id("workspace-right-dock")
            .w(px(right_width))
            .h_full()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(rgb(color::COLOR_BORDER))
            .overflow_hidden();

        right_dock = match target {
            FocusTarget::ShortcutPanel => right_dock.child(self.shortcut_panel.clone()),
            _ => right_dock,
        };

        right_dock
    }
}
