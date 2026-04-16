//! 底部工具栏视图。

use gpui::{prelude::*, rgb};
use zom_app::state::{DesktopAppState, ToolBarEntry};

use super::icons;
use crate::chrome;
use crate::components::chip;
use crate::theme::{color, size};

/// 渲染底部工具栏。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    chrome::bar()
        .bg(rgb(color::COLOR_BG_APP))
        .border_t_1()
        .border_color(rgb(color::COLOR_BORDER))
        .text_xs()
        .text_color(rgb(color::COLOR_FG_MUTED))
        .child(
            chrome::group().children(
                state
                    .tool_bar
                    .left_tools
                    .iter()
                    .enumerate()
                    .map(|(index, item)| render_tool("tool-bar-left", index, item)),
            ),
        )
        .child(
            chrome::group()
                .child(render_value(&state.tool_bar.cursor))
                .child(render_value(&state.tool_bar.language))
                .child(render_value(&state.tool_bar.line_ending))
                .child(render_value(&state.tool_bar.encoding))
                .children(
                    state
                        .tool_bar
                        .right_tools
                        .iter()
                        .enumerate()
                        .map(|(index, item)| render_tool("tool-bar-right", index, item)),
                ),
        )
}

/// 渲染工具栏中的单个图标入口。
fn render_tool(group: &'static str, index: usize, item: &ToolBarEntry) -> impl IntoElement {
    let spec = icons::spec(item.icon);

    chip::interactive_icon_chip(
        (group, index),
        chip::TooltipSpec::new(spec.label, spec.shortcut),
    )
    .child(icons::render(
        item.icon,
        size::ICON_MD,
        rgb(color::COLOR_FG_MUTED),
    ))
}

/// 渲染工具栏中的纯文本值。
fn render_value(value: &str) -> impl IntoElement {
    chip::status_chip().child(value.to_string())
}
