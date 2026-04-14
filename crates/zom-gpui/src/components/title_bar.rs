//! 顶部标题栏组件。

use gpui::{FontWeight, div, prelude::*, px, rgb};
use zom_app::DesktopAppState;

use crate::chrome;

/// 渲染顶栏，表达当前工作区和搜索入口位置。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    chrome::bar()
        .bg(rgb(0x161a22))
        .border_b_1()
        .border_color(rgb(0x262d3a))
        .child(
            chrome::group()
                .pl(px(chrome::title_bar_leading_inset()))
                .child(
                    chrome::chip()
                        .gap(px(chrome::gap()))
                        .child(div().text_xs().text_color(rgb(0x7aa2ff)).child("▦"))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(state.workspace_name.clone()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x8f9bb3))
                        .child(state.active_buffer.clone()),
                ),
        )
        .child(
            chrome::group()
                .child(
                    chrome::chip()
                        .w(px(chrome::TITLEBAR_SEARCH_WIDTH))
                        .text_xs()
                        .text_color(rgb(0x7f8aa3))
                        .child("Search files, symbols, commands"),
                )
                .child(render_settings_button()),
        )
}

/// 渲染标题栏右侧系统设置按钮。
fn render_settings_button() -> impl IntoElement {
    chrome::chip().child(
        div()
            .size(px(18.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(rgb(0xc9d4e6))
            .child("⚙"),
    )
}
