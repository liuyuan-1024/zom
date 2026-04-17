//! 底部工具栏视图。

use gpui::{div, prelude::*, px, rgb};
use zom_app::state::{DesktopAppState, ToolBarEntry};

use super::icons;
use crate::chrome;
use crate::components::chip;
use crate::theme::{color, size};

/// 渲染底部工具栏。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    chrome::bar()
        .border_t_1()
        .border_color(rgb(color::COLOR_BORDER))
        .text_xs()
        .text_color(rgb(color::COLOR_FG_MUTED))
        .child(
            chrome::group().child(
                div().flex().items_center().gap(px(size::GAP_1_5)).children(
                    state
                        .tool_bar
                        .left_tools
                        .iter()
                        .enumerate()
                        .map(|(index, item)| render_tool("tool-bar-left", index, item)),
                ),
            ),
        )
        .child(
            chrome::group()
                .child(render_value(
                    "tb-cursor",
                    &state.tool_bar.cursor,
                    "Go to Line/Col",
                ))
                .child(render_value(
                    "tb-language",
                    &state.tool_bar.language,
                    "Select Language",
                ))
                .child(render_value(
                    "tb-line-ending",
                    &state.tool_bar.line_ending,
                    "Select End of Line Sequence",
                ))
                .child(render_value(
                    "tb-encoding",
                    &state.tool_bar.encoding,
                    "Select Encoding",
                ))
                .child(
                    div().flex().items_center().gap(px(size::GAP_1_5)).children(
                        state
                            .tool_bar
                            .right_tools
                            .iter()
                            .enumerate()
                            .map(|(index, item)| render_tool("tool-bar-right", index, item)),
                    ),
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

/// 渲染工具栏中的与当前活动文本有关的工具。
fn render_value(id: &'static str, value: &str, tooltip: &str) -> impl IntoElement {
    chip::interactive_chip(id, chip::TooltipSpec::new(tooltip, None::<&str>))
        .child(value.to_string())
}
