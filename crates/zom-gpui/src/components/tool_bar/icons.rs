//! 底部工具栏专属的图标定义与渲染。

use gpui::{Hsla, div, prelude::*, px, svg};
use zom_app::projection::shortcut_hint;
use zom_app::state::ToolBarIcon;
use zom_core::{Command, FocusTarget, command::WorkspaceCommand};

/// 底部工具栏图标的展示规格。
pub(super) struct ToolBarIconSpec {
    /// SVG 资源路径。
    pub path: &'static str,
    /// 悬停时显示的名称。
    pub label: &'static str,
    /// 悬停时显示的快捷键。
    pub shortcut: Option<String>,
}

/// 将应用层语义映射为底部工具栏自身维护的展示规格。
pub(super) fn spec(icon: ToolBarIcon) -> ToolBarIconSpec {
    match icon {
        ToolBarIcon::FileTree => ToolBarIconSpec {
            path: "icons/tool_bar/tool_file_tree.svg",
            label: "文件树",
            shortcut: focus_panel_shortcut(FocusTarget::FileTreePanel),
        },
        ToolBarIcon::GitBranch => ToolBarIconSpec {
            path: "icons/tool_bar/tool_git_branch_alt.svg",
            label: "Git",
            shortcut: focus_panel_shortcut(FocusTarget::GitPanel),
        },
        ToolBarIcon::Outline => ToolBarIconSpec {
            path: "icons/tool_bar/tool_list_tree.svg",
            label: "Outline",
            shortcut: focus_panel_shortcut(FocusTarget::OutlinePanel),
        },
        ToolBarIcon::ProjectSearch => ToolBarIconSpec {
            path: "icons/tool_bar/tool_search.svg",
            label: "Search",
            shortcut: focus_panel_shortcut(FocusTarget::ProjectSearch),
        },
        ToolBarIcon::LSP => ToolBarIconSpec {
            path: "icons/tool_bar/tool_bolt_outlined.svg",
            label: "Code Actions",
            shortcut: workspace_shortcut(WorkspaceCommand::OpenCodeActions),
        },
        ToolBarIcon::Terminal => ToolBarIconSpec {
            path: "icons/tool_bar/tool_terminal.svg",
            label: "Terminal",
            shortcut: focus_panel_shortcut(FocusTarget::Terminal),
        },
        ToolBarIcon::Debug => ToolBarIconSpec {
            path: "icons/tool_bar/tool_debug.svg",
            label: "Debug",
            shortcut: workspace_shortcut(WorkspaceCommand::StartDebugging),
        },
        ToolBarIcon::Notification => ToolBarIconSpec {
            path: "icons/tool_bar/tool_notification.svg",
            label: "Notifications",
            shortcut: focus_panel_shortcut(FocusTarget::Notification),
        },
    }
}

fn focus_panel_shortcut(target: FocusTarget) -> Option<String> {
    workspace_shortcut(WorkspaceCommand::FocusPanel(target))
}

fn workspace_shortcut(command: WorkspaceCommand) -> Option<String> {
    shortcut_hint(&Command::from(command))
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
