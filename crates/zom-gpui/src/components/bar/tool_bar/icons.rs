//! 工具栏图标规格与渲染。

use gpui::{Hsla, div, prelude::*, px, svg};
use zom_runtime::projection::shortcut_hint;
use zom_runtime::state::ToolBarEntry;
use zom_protocol::{CommandInvocation, FocusTarget, WorkspaceAction};

/// 底部工具栏图标的展示规格。
pub(super) struct ToolBarIconSpec {
    /// SVG 资源路径。
    pub path: &'static str,
    /// 悬停时显示的名称。
    pub label: &'static str,
    /// 悬停时显示的快捷键。
    pub shortcut: Option<String>,
}

/// 将动作语义映射为工具栏展示规格。
pub(super) fn spec(item: &ToolBarEntry) -> ToolBarIconSpec {
    let (path, label) = match &item.command {
        CommandInvocation::Workspace(WorkspaceAction::FocusPanel(FocusTarget::FileTreePanel)) => {
            ("icons/tool_bar/tool_file_tree.svg", "文件树")
        }
        CommandInvocation::Workspace(WorkspaceAction::FocusPanel(FocusTarget::GitPanel)) => {
            ("icons/tool_bar/tool_git_branch_alt.svg", "Git")
        }
        CommandInvocation::Workspace(WorkspaceAction::FocusPanel(FocusTarget::OutlinePanel)) => {
            ("icons/tool_bar/tool_list_tree.svg", "大纲")
        }
        CommandInvocation::Workspace(WorkspaceAction::FocusPanel(
            FocusTarget::ProjectSearchPanel,
        )) => ("icons/tool_bar/tool_search.svg", "项目搜索"),
        CommandInvocation::Workspace(WorkspaceAction::FocusPanel(
            FocusTarget::LanguageServersPanel,
        )) => ("icons/tool_bar/tool_bolt_outlined.svg", "语言服务器"),
        CommandInvocation::Workspace(WorkspaceAction::FocusPanel(FocusTarget::TerminalPanel)) => {
            ("icons/tool_bar/tool_terminal.svg", "终端")
        }
        CommandInvocation::Workspace(WorkspaceAction::FocusPanel(FocusTarget::DebugPanel)) => {
            ("icons/tool_bar/tool_debug.svg", "Debug")
        }
        CommandInvocation::Workspace(WorkspaceAction::FocusPanel(
            FocusTarget::NotificationPanel,
        )) => ("icons/tool_bar/tool_notification.svg", "通知"),
        _ => ("icons/keyboard.svg", "Action"),
    };

    ToolBarIconSpec {
        path,
        label,
        shortcut: shortcut_hint(&item.command),
    }
}

/// 渲染工具栏中的单色 SVG 图标。
pub(super) fn render(
    item: &ToolBarEntry,
    size: f32,
    color: impl Into<Hsla>,
) -> impl IntoElement {
    let color = color.into();
    let spec = spec(item);

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(spec.path).size(px(size)).text_color(color))
}
