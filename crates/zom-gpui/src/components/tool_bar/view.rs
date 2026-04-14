//! 底部工具栏视图。

use gpui::{CursorStyle, prelude::*, rgb};
use zom_app::state::{DesktopAppState, ToolBarItem};

use super::icons;
use crate::chrome;

/// 渲染底部工具栏。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    chrome::bar()
        .bg(rgb(0x0d1218))
        .border_t_1()
        .border_color(rgb(0x202938))
        .text_xs()
        .text_color(rgb(0xaeb8ca))
        .child(chrome::group().children(state.tool_bar.left_items.iter().map(render_item)))
        .child(
            chrome::group()
                .child(render_value(&state.tool_bar.cursor))
                .child(render_value(&state.tool_bar.language))
                .child(render_value(&state.tool_bar.line_ending))
                .child(render_value(&state.tool_bar.encoding))
                .children(state.tool_bar.right_items.iter().map(render_item)),
        )
}

/// 渲染工具栏中的单个图标入口。
fn render_item(item: &ToolBarItem) -> impl IntoElement {
    chrome::icon_chip()
        .cursor(CursorStyle::PointingHand)
        .child(icons::render(
            item.icon,
            chrome::tool_icon_size(),
            rgb(0xd7e0ef),
        ))
}

/// 渲染工具栏中的纯文本值。
fn render_value(value: &str) -> impl IntoElement {
    chrome::chip()
        .text_xs()
        .text_color(rgb(0xd7e0ef))
        .bg(rgb(0x121923))
        .border_1()
        .border_color(rgb(0x283243))
        .child(value.to_string())
}
