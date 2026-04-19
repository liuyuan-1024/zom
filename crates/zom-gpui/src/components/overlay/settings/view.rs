//! 设置悬浮层视图。

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
                .gap(px(size::GAP_2))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(color::COLOR_FG_PRIMARY))
                        .child("This is a minimal settings overlay used to validate overlay flow."),
                )
                .child(setting_row("Editor: Font Size", "16"))
                .child(setting_row("Theme", "Dark"))
                .child(setting_row("Line Ending", "LF"))
                .child(
                    div()
                        .pt(px(size::GAP_1))
                        .text_xs()
                        .text_color(rgb(color::COLOR_FG_MUTED))
                        .child("Use Cmd+W to close, or click outside the card."),
                ),
        )
}

fn setting_row(label: &'static str, value: &'static str) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .px(px(size::GAP_2))
        .py(px(size::GAP_1_5))
        .bg(rgb(color::COLOR_BG_ELEMENT))
        .rounded_sm()
        .child(
            div()
                .text_sm()
                .text_color(rgb(color::COLOR_FG_PRIMARY))
                .child(label),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(color::COLOR_FG_MUTED))
                .child(value),
        )
}
