//! 工作区分割条渲染与拖拽起止事件绑定。
use gpui::{
    Context, CursorStyle, Div, InteractiveElement, MouseButton, MouseDownEvent, MouseUpEvent,
    Stateful, Styled, div, px,
};

use super::{WorkspaceView, view::ActiveDockDrag};
use crate::theme::size;

impl WorkspaceView {
    /// 渲染底部分割条并在按下时记录拖拽起点。
    pub(super) fn render_bottom_splitter(
        &self,
        bottom_height: f32,
        splitter_size: f32,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let origin_height = bottom_height;
        div()
            .id("workspace-bottom-splitter")
            .absolute()
            .left(px(0.0))
            .bottom(px(bottom_height - splitter_size * 0.5))
            .w_full()
            .h(px(splitter_size))
            .cursor(CursorStyle::ResizeUpDown)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                    let origin_y: f32 = event.position.y.into();
                    this.active_dock_drag = Some(ActiveDockDrag::Bottom {
                        origin_y,
                        origin_height,
                    });
                    cx.stop_propagation();
                    cx.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    this.active_dock_drag = None;
                    cx.notify();
                }),
            )
    }

    /// 渲染左侧分割条并切换为左侧拖拽模式。
    pub(super) fn render_left_splitter(
        &self,
        left_width: f32,
        splitter_size: f32,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let left_splitter_x = left_width - splitter_size * 0.5;
        div()
            .id("workspace-left-splitter")
            .absolute()
            .left(px(left_splitter_x))
            .top(px(0.0))
            .w(px(splitter_size))
            .h_full()
            .cursor(CursorStyle::ResizeLeftRight)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                    this.active_dock_drag = Some(ActiveDockDrag::Left);
                    cx.stop_propagation();
                    cx.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    this.active_dock_drag = None;
                    cx.notify();
                }),
            )
    }

    /// 渲染右侧分割条并切换为右侧拖拽模式。
    pub(super) fn render_right_splitter(
        &self,
        workspace_width: f32,
        right_width: f32,
        splitter_size: f32,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let right_splitter_x = workspace_width - right_width - splitter_size * 0.5;
        div()
            .id("workspace-right-splitter")
            .absolute()
            .left(px(right_splitter_x))
            .top(px(0.0))
            .w(px(splitter_size))
            .h_full()
            .cursor(CursorStyle::ResizeLeftRight)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                    this.active_dock_drag = Some(ActiveDockDrag::Right);
                    cx.stop_propagation();
                    cx.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    this.active_dock_drag = None;
                    cx.notify();
                }),
            )
    }
}

pub(super) fn dock_gap() -> f32 {
    size::GAP_1
}

/// 分割条命中宽度（用于点击/拖拽手柄尺寸与贴边隐藏阈值）。
pub(super) fn splitter_hit_size() -> f32 {
    size::GAP_1
}
