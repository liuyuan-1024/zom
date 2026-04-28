//! 聚焦并显示指定面板的命令元信息。

use crate::FocusTarget;
use crate::command::kind::{CommandKind, CommandKindSpec};

/// 可见性托管面板聚焦命令规范列表。
pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::Palette),
        "workspace.focus_panel.palette",
        "聚焦命令调色板",
        "显示并聚焦命令调色板",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::FileTreePanel),
        "workspace.focus_panel.file_tree",
        "聚焦文件树面板",
        "显示并聚焦文件树面板",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::GitPanel),
        "workspace.focus_panel.git",
        "聚焦 Git 面板",
        "显示并聚焦 Git 面板",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::OutlinePanel),
        "workspace.focus_panel.outline",
        "聚焦大纲面板",
        "显示并聚焦大纲面板",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::ProjectSearchPanel),
        "workspace.focus_panel.project_search",
        "聚焦项目搜索面板",
        "显示并聚焦项目搜索面板",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::LanguageServersPanel),
        "workspace.focus_panel.language_servers",
        "聚焦语言服务器面板",
        "显示并聚焦语言服务器面板",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::TerminalPanel),
        "workspace.focus_panel.terminal",
        "聚焦终端面板",
        "显示并聚焦终端面板",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::DebugPanel),
        "workspace.focus_panel.debug",
        "聚焦调试面板",
        "显示并聚焦调试面板",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::NotificationPanel),
        "workspace.focus_panel.notification",
        "聚焦通知面板",
        "显示并聚焦通知面板",
    ),
];
