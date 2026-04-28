//! 设置悬浮层与遮罩交互渲染。

use gpui::{Context, Div, InteractiveElement, ParentElement, Stateful, Styled, div, px, rgb};

use super::super::ZomRootView;
use crate::{
    components::{notification_toast_overlay, settings_overlay},
    theme::color,
};

impl ZomRootView {
    /// 渲染设置浮层（含遮罩与中间卡片）。
    pub(super) fn render_settings_overlay(&self, _cx: &mut Context<Self>) -> Stateful<Div> {
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
                    .opacity(0.72),
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
                            .child(settings_overlay::panel()),
                    ),
            )
    }

    /// 渲染当前活跃通知的悬浮提示层。
    pub(super) fn render_notification_toast_layer(&self) -> Option<Stateful<Div>> {
        let notification = self.state.active_toast_notification.as_ref()?;
        Some(notification_toast_overlay::layer(notification))
    }
}
