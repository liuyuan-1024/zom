//! 设置面板视图渲染。

use gpui::{FontWeight, InteractiveElement, IntoElement, ParentElement, Styled, div, px, rgb};

use crate::theme::{color, size};

/// 渲染设置悬浮层卡片主体。
pub(crate) fn panel() -> impl IntoElement {
    div()
        .id("settings-overlay-card")
        .w(px(560.0))
        .flex()
        .flex_col()
        .bg(rgb(color::COLOR_BG_PANEL))
        .border_1()
        .border_color(rgb(color::COLOR_BORDER))
        .rounded_sm()
        .shadow_md()
        .overflow_hidden()
        .child(
            div()
                .px(px(size::GAP_3))
                .py(px(size::GAP_2))
                .border_b_1()
                .border_color(rgb(color::COLOR_BORDER))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(color::COLOR_FG_PRIMARY))
                        .child("Settings"),
                ),
        )
        .child(
            div()
                .px(px(size::GAP_3))
                .py(px(size::GAP_3))
                .flex()
                .flex_col()
                .gap(px(size::GAP_2)),
        )
}
