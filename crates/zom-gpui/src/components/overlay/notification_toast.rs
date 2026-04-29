//! 通知悬浮提示渲染。

use gpui::{Div, InteractiveElement, ParentElement, Stateful, Styled, div, px, rgb};
use zom_runtime::state::{DesktopNotification, DesktopNotificationLevel};

use crate::theme::{color, size};

/// 渲染通知悬浮提示层。
pub(crate) fn layer(notification: &DesktopNotification) -> Stateful<Div> {
    let (border_color, background_color) = level_tone(notification.level);

    div()
        .id("notification-toast-layer")
        .absolute()
        .top(px(size::BAR_HEIGHT + size::GAP_2))
        .left(px(0.0))
        .w_full()
        .px(px(size::GAP_2))
        .child(
            div().w_full().flex().child(div().flex_1()).child(
                div()
                    .id("notification-toast-card")
                    .w(px(360.0))
                    .flex()
                    .flex_col()
                    .p(px(size::GAP_2))
                    .bg(rgb(background_color))
                    .border_1()
                    .border_color(rgb(border_color))
                    .rounded_sm()
                    .shadow_md()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(color::COLOR_FG_PRIMARY))
                            .child(notification.message.clone()),
                    ),
            ),
        )
}

fn level_tone(level: DesktopNotificationLevel) -> (u32, u32) {
    match level {
        DesktopNotificationLevel::Info => (color::COLOR_FG_MUTED, color::COLOR_BG_PANEL),
        DesktopNotificationLevel::Warning => (0xD29922, 0x2A230F),
        DesktopNotificationLevel::Error => (0xF85149, 0x32191D),
    }
}
