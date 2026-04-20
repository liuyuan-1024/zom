//! 设置悬浮层与遮罩交互渲染。

use gpui::{
    Context, Div, InteractiveElement, MouseButton, MouseDownEvent, ParentElement, Stateful, Styled,
    div, px, rgb,
};
use zom_core::{CommandInvocation, WorkspaceAction};

use super::super::ZomRootView;
use crate::{components::settings_overlay, theme::color};

impl ZomRootView {
    /// 渲染设置浮层（含遮罩、点击空白关闭与中间卡片）。
    pub(super) fn render_settings_overlay(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        div()
            .id("settings-overlay-layer")
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .w_full()
            .h_full()
            .child(
                div()
                    .id("settings-overlay-mask")
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .w_full()
                    .h_full()
                    .bg(rgb(color::COLOR_BG_APP))
                    .opacity(0.72)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                            this.state.handle_command(CommandInvocation::from(
                                WorkspaceAction::CloseFocused,
                            ));
                            this.sync_child_views(cx);
                            cx.stop_propagation();
                            cx.notify();
                        }),
                    ),
            )
            .child(
                div()
                    .id("settings-overlay-center")
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .id("settings-overlay-card-container")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_this, _event: &MouseDownEvent, _window, cx| {
                                    cx.stop_propagation();
                                }),
                            )
                            .child(settings_overlay::panel()),
                    ),
            )
    }
}
