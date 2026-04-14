//! 底部工具栏专属的图标定义与渲染。

use gpui::{Hsla, div, prelude::*, px, svg};
use zom_app::state::ToolBarIcon;

fn icon_path(icon: ToolBarIcon) -> &'static str {
    match icon {
        ToolBarIcon::Files => "icons/tool_bar/tool_file_tree.svg",
        ToolBarIcon::GitBranch => "icons/tool_bar/tool_git_branch_alt.svg",
        ToolBarIcon::Outline => "icons/tool_bar/tool_list_tree.svg",
        ToolBarIcon::Search => "icons/tool_bar/tool_search.svg",
        ToolBarIcon::LanguageServer => "icons/tool_bar/tool_bolt_outlined.svg",
        ToolBarIcon::Terminal => "icons/tool_bar/tool_terminal.svg",
        ToolBarIcon::Debug => "icons/tool_bar/tool_debug.svg",
        ToolBarIcon::Notifications => "icons/tool_bar/tool_notification.svg",
    }
}

/// 渲染工具栏中的单色 SVG 图标
pub(super) fn render(icon: ToolBarIcon, size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    let color = color.into();

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(icon_path(icon)).size(px(size)).text_color(color))
}
