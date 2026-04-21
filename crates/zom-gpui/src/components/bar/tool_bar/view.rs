//! 底部工具栏视图渲染。

use gpui::{div, prelude::*, px, rgb};
use zom_runtime::{
    projection::{command_dock, command_is_active, cursor_text},
    state::{DesktopAppState, PanelDock, ToolBarEntry},
};

use super::icons;
use crate::components::bar::{bar, group};
use crate::components::chip;
use crate::theme::{color, size};

/// 渲染底部工具栏。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    let mut bottom_dock_tools: Vec<(usize, &ToolBarEntry)> = Vec::new();
    let mut right_dock_tools: Vec<(usize, &ToolBarEntry)> = Vec::new();
    for (index, item) in state.tool_bar.right_tools.iter().enumerate() {
        match right_tool_dock(item) {
            Some(PanelDock::Bottom) => bottom_dock_tools.push((index, item)),
            Some(PanelDock::Right) => right_dock_tools.push((index, item)),
            _ => right_dock_tools.push((index, item)),
        }
    }

    let mut right_sections = group().child(render_active_file_status(state));
    if !bottom_dock_tools.is_empty() {
        right_sections = right_sections
            .child(render_section_divider("tb-divider-status-bottom"))
            .child(render_tool_group(
                state,
                "tool-bar-right-bottom",
                &bottom_dock_tools,
            ));
    }
    if !right_dock_tools.is_empty() {
        right_sections = right_sections
            .child(render_section_divider("tb-divider-bottom-right"))
            .child(render_tool_group(
                state,
                "tool-bar-right",
                &right_dock_tools,
            ));
    }

    bar()
        .border_t_1()
        .border_color(rgb(color::COLOR_BORDER))
        .text_xs()
        .text_color(rgb(color::COLOR_FG_MUTED))
        .child(
            group().child(
                div().flex().items_center().gap(px(size::GAP_1_5)).children(
                    state
                        .tool_bar
                        .left_tools
                        .iter()
                        .enumerate()
                        .map(|(index, item)| render_tool(state, "tool-bar-left", index, item)),
                ),
            ),
        )
        .child(right_sections)
}

/// 渲染工具栏中的单个图标入口。
fn render_tool(
    state: &DesktopAppState,
    group: &'static str,
    index: usize,
    item: &ToolBarEntry,
) -> impl IntoElement {
    let spec = icons::spec(item);
    let is_active = command_is_active(state, &item.command);
    let icon_color = if is_active {
        rgb(color::COLOR_FG_PRIMARY)
    } else {
        rgb(color::COLOR_FG_MUTED)
    };

    chip::interactive_icon_chip(
        (group, index),
        chip::TooltipSpec::new(spec.label, spec.shortcut),
    )
    .child(icons::render(item, size::ICON_MD, icon_color))
}

/// 渲染工具栏中的与当前活动文本有关的工具。
fn render_value(id: &'static str, value: &str, tooltip: &str) -> impl IntoElement {
    chip::interactive_chip(id, chip::TooltipSpec::new(tooltip, None::<&str>))
        .child(value.to_string())
}

fn render_active_file_status(state: &DesktopAppState) -> impl IntoElement {
    if state.pane.active_tab().is_none() {
        return group();
    }

    let cursor = cursor_text(state.tool_bar.cursor);

    group()
        .child(render_value("tb-cursor", &cursor, "跳转到行:列"))
        .child(render_value(
            "tb-language",
            &state.tool_bar.language,
            "选择语言类型",
        ))
}

fn render_tool_group(
    state: &DesktopAppState,
    group_id: &'static str,
    items: &[(usize, &ToolBarEntry)],
) -> impl IntoElement {
    div().flex().items_center().gap(px(size::GAP_1_5)).children(
        items
            .iter()
            .map(|(index, item)| render_tool(state, group_id, *index, item)),
    )
}

fn render_section_divider(id: &'static str) -> impl IntoElement {
    div()
        .id(id)
        .w(px(1.0))
        .h(px(size::ICON_MD))
        .bg(rgb(color::COLOR_BORDER))
}

fn right_tool_dock(item: &ToolBarEntry) -> Option<PanelDock> {
    command_dock(&item.command)
}
