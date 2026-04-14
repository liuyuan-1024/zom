//! 底部工具栏专属的图标定义与渲染。

use gpui::{Hsla, div, prelude::*, px, svg};
use zom_app::state::ToolBarIcon;

/// 底部工具栏图标的展示规格。
pub(super) struct ToolBarIconSpec {
    /// SVG 资源路径。
    pub path: &'static str,
    /// 悬停时显示的名称。
    pub label: &'static str,
    /// 悬停时显示的快捷键。
    pub shortcut: Option<&'static str>,
}

/// 将应用层语义映射为底部工具栏自身维护的展示规格。
pub(super) fn spec(icon: ToolBarIcon) -> ToolBarIconSpec {
    match icon {
        ToolBarIcon::Files => ToolBarIconSpec {
            path: "icons/tool_bar/tool_file_tree.svg",
            label: "Explorer",
            shortcut: Some("Cmd+Shift+E"),
        },
        ToolBarIcon::GitBranch => ToolBarIconSpec {
            path: "icons/tool_bar/tool_git_branch_alt.svg",
            label: "Git",
            shortcut: Some("Cmd+Shift+G"),
        },
        ToolBarIcon::Outline => ToolBarIconSpec {
            path: "icons/tool_bar/tool_list_tree.svg",
            label: "Outline",
            shortcut: Some("Cmd+Shift+O"),
        },
        ToolBarIcon::Search => ToolBarIconSpec {
            path: "icons/tool_bar/tool_search.svg",
            label: "Search",
            shortcut: Some("Cmd+Shift+F"),
        },
        ToolBarIcon::LanguageServer => ToolBarIconSpec {
            path: "icons/tool_bar/tool_bolt_outlined.svg",
            label: "Code Actions",
            shortcut: Some("Cmd+."),
        },
        ToolBarIcon::Terminal => ToolBarIconSpec {
            path: "icons/tool_bar/tool_terminal.svg",
            label: "Terminal",
            shortcut: Some("Ctrl+`"),
        },
        ToolBarIcon::Debug => ToolBarIconSpec {
            path: "icons/tool_bar/tool_debug.svg",
            label: "Debug",
            shortcut: Some("F5"),
        },
        ToolBarIcon::Notifications => ToolBarIconSpec {
            path: "icons/tool_bar/tool_notification.svg",
            label: "Notifications",
            shortcut: Some("Cmd+Shift+N"),
        },
    }
}

/// 渲染工具栏中的单色 SVG 图标
pub(super) fn render(icon: ToolBarIcon, size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    let color = color.into();
    let spec = spec(icon);

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(spec.path).size(px(size)).text_color(color))
}
