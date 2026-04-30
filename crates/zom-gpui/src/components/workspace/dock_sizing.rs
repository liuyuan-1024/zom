//! 工作区停靠区尺寸约束与拖拽更新逻辑。
use gpui::{Context, MouseMoveEvent};
use zom_runtime::state::PanelDock;

use super::splitters::{dock_gap, splitter_hit_size};
use super::{WorkspaceView, view::ActiveDockDrag};
use crate::theme::size;

const DEFAULT_BOTTOM_PANEL_HEIGHT: f32 = 240.0;

impl WorkspaceView {
    /// 当某个 dock 从隐藏切回可见时，恢复其默认尺寸，避免面板以 0 尺寸显示。
    pub(super) fn restore_dock_sizes_when_visible(
        &mut self,
        left_target: Option<zom_protocol::FocusTarget>,
        right_target: Option<zom_protocol::FocusTarget>,
        bottom_target: Option<zom_protocol::FocusTarget>,
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

    /// 根据窗口视口高度推导工作区可用高度（扣除标题栏和状态栏）。
    pub(super) fn workspace_height_from_viewport(viewport_height: f32) -> f32 {
        (viewport_height - size::BAR_HEIGHT * 2.0).max(0.0)
    }

    /// 计算单侧 dock 的理论最大宽度（不考虑对侧占用）。
    fn max_dock_width(workspace_width: f32) -> f32 {
        (workspace_width - dock_gap()).max(0.0)
    }

    /// 计算底部面板理论最大高度。
    fn max_bottom_panel_height(workspace_height: f32) -> f32 {
        (workspace_height - (dock_gap() + size::GAP_1)).max(0.0)
    }

    /// 规范化左右 dock 宽度，确保两侧同时可见时不发生重叠。
    pub(super) fn normalize_dock_widths(
        &mut self,
        workspace_width: f32,
        is_left_visible: bool,
        is_right_visible: bool,
    ) {
        self.left_dock_width = self.left_dock_width.max(0.0);
        self.right_dock_width = self.right_dock_width.max(0.0);

        let mut max_left = Self::max_dock_width(workspace_width);
        let mut max_right = Self::max_dock_width(workspace_width);
        if is_left_visible && is_right_visible {
            max_left = (workspace_width - self.right_dock_width - dock_gap()).max(0.0);
            max_right = (workspace_width - self.left_dock_width - dock_gap()).max(0.0);
        }
        self.left_dock_width = self.left_dock_width.min(max_left);
        self.right_dock_width = self.right_dock_width.min(max_right);
    }

    /// 规范化底部面板高度到合法区间。
    pub(super) fn normalize_bottom_panel_height(&mut self, workspace_height: f32) {
        self.bottom_panel_height = self
            .bottom_panel_height
            .clamp(0.0, Self::max_bottom_panel_height(workspace_height));
    }

    /// 根据鼠标 x 坐标更新左侧 dock 宽度。
    fn update_left_dock_width_from_cursor(
        &mut self,
        cursor_x: f32,
        workspace_width: f32,
        is_right_visible: bool,
    ) {
        let max_left = if is_right_visible {
            (workspace_width - self.right_dock_width - dock_gap()).max(0.0)
        } else {
            Self::max_dock_width(workspace_width)
        };
        self.left_dock_width = cursor_x.clamp(0.0, max_left);
    }

    /// 根据鼠标 x 坐标更新右侧 dock 宽度。
    fn update_right_dock_width_from_cursor(
        &mut self,
        cursor_x: f32,
        workspace_width: f32,
        is_left_visible: bool,
    ) {
        let max_right = if is_left_visible {
            (workspace_width - self.left_dock_width - dock_gap()).max(0.0)
        } else {
            Self::max_dock_width(workspace_width)
        };
        let raw_width = workspace_width - cursor_x;
        self.right_dock_width = raw_width.clamp(0.0, max_right);
    }

    /// 判断拖拽是否抵达“自动隐藏阈值”。
    ///
    /// 左右以贴边阈值判定，底部以拖拽后的预测高度判定。
    fn reached_hide_boundary(&self, event: &MouseMoveEvent, workspace_width: f32) -> bool {
        let edge = splitter_hit_size() * 0.5;
        match self.active_dock_drag {
            Some(ActiveDockDrag::Left) => {
                let cursor_x: f32 = event.position.x.into();
                cursor_x <= edge
            }
            Some(ActiveDockDrag::Right) => {
                let cursor_x: f32 = event.position.x.into();
                cursor_x >= (workspace_width - edge)
            }
            Some(ActiveDockDrag::Bottom {
                origin_y,
                origin_height,
            }) => {
                let cursor_y: f32 = event.position.y.into();
                let delta = cursor_y - origin_y;
                let next_height = (origin_height - delta).max(0.0);
                next_height <= edge
            }
            None => false,
        }
    }

    /// 隐藏当前正在拖拽的 dock，并同步 core 状态机。
    fn hide_active_dock_panel(&mut self, cx: &mut Context<Self>) {
        match self.active_dock_drag {
            Some(ActiveDockDrag::Left) => {
                self.left_dock_width = 0.0;
                self.store.update(cx, |store, cx| {
                    store.dispatch(crate::root_view::store::UiAction::HidePanelInDock(
                        PanelDock::Left,
                    ));
                    cx.notify();
                });
            }
            Some(ActiveDockDrag::Right) => {
                self.right_dock_width = 0.0;
                self.store.update(cx, |store, cx| {
                    store.dispatch(crate::root_view::store::UiAction::HidePanelInDock(
                        PanelDock::Right,
                    ));
                    cx.notify();
                });
            }
            Some(ActiveDockDrag::Bottom { .. }) => {
                self.bottom_panel_height = 0.0;
                self.store.update(cx, |store, cx| {
                    store.dispatch(crate::root_view::store::UiAction::HidePanelInDock(
                        PanelDock::Bottom,
                    ));
                    cx.notify();
                });
            }
            None => {}
        }
    }

    /// 响应分割条拖拽：按拖拽方向更新面板尺寸，触边时自动隐藏对应 dock 面板。
    pub(super) fn on_drag_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        workspace_width: f32,
        workspace_height: f32,
        cx: &mut Context<Self>,
    ) {
        if self.active_dock_drag.is_none() {
            return;
        }

        if self.reached_hide_boundary(event, workspace_width) {
            self.hide_active_dock_panel(cx);
            self.active_dock_drag = None;
            cx.notify();
            return;
        }

        if !event.dragging() {
            self.active_dock_drag = None;
            cx.notify();
            return;
        }

        let (is_left_visible, is_right_visible) = {
            let store = self.store.read(cx);
            (
                store
                    .select_visible_panel_in_dock(PanelDock::Left)
                    .is_some(),
                store
                    .select_visible_panel_in_dock(PanelDock::Right)
                    .is_some(),
            )
        };

        match self.active_dock_drag {
            Some(ActiveDockDrag::Left) => {
                let cursor_x: f32 = event.position.x.into();
                self.update_left_dock_width_from_cursor(
                    cursor_x,
                    workspace_width,
                    is_right_visible,
                );
            }
            Some(ActiveDockDrag::Right) => {
                let cursor_x: f32 = event.position.x.into();
                self.update_right_dock_width_from_cursor(
                    cursor_x,
                    workspace_width,
                    is_left_visible,
                );
            }
            Some(ActiveDockDrag::Bottom {
                origin_y,
                origin_height,
            }) => {
                let cursor_y: f32 = event.position.y.into();
                let delta = cursor_y - origin_y;
                let next_height = (origin_height - delta).max(0.0);
                self.bottom_panel_height =
                    next_height.min(Self::max_bottom_panel_height(workspace_height));
            }
            None => {}
        }
        cx.notify();
    }
}
