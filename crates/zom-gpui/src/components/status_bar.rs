//! 底部状态栏组件。

use gpui::{div, prelude::*, px, rgb};
use zom_app::{DesktopAppState, StatusBarItem};

use crate::chrome;

/// 渲染底部状态栏。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    chrome::bar()
        .bg(rgb(0x0d1218))
        .border_t_1()
        .border_color(rgb(0x202938))
        .text_xs()
        .text_color(rgb(0xaeb8ca))
        .child(chrome::group().children(state.status_bar.left_items.iter().map(render_item)))
        .child(
            chrome::group()
                .child(render_value(&state.status_bar.cursor))
                .child(render_value(&state.status_bar.language))
                .children(state.status_bar.right_items.iter().map(render_item)),
        )
}

/// 渲染状态栏中的单个图标入口。
fn render_item(item: &StatusBarItem) -> impl IntoElement {
    chrome::chip()
        .gap(px(chrome::gap()))
        .text_xs()
        .bg(rgb(0x121923))
        .border_1()
        .border_color(rgb(0x283243))
        .child(div().text_color(rgb(0xd7e0ef)).child(item.icon.clone()))
        .child(div().text_color(rgb(0x8f9bb2)).child(item.label.clone()))
}

/// 渲染状态栏中的纯文本值。
fn render_value(value: &str) -> impl IntoElement {
    chrome::chip()
        .text_xs()
        .text_color(rgb(0xd7e0ef))
        .bg(rgb(0x121923))
        .border_1()
        .border_color(rgb(0x283243))
        .child(value.to_string())
}
