//！ 聚焦并显示指定面板的命令元信息

use crate::FocusTarget;
use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope,
    types::{meta_char, meta_shift_char},
};
use crate::{CommandInvocation, WorkspaceAction};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::Editor),
        "workspace.focus_panel.editor",
        "Focus Editor",
        "Move focus to the editor pane.",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::Editor))
        }),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::Palette),
        "workspace.focus_panel.palette",
        "聚焦命令调色板",
        "显示并聚焦命令调色板",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::Palette))
        }),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::FileTreePanel),
        "workspace.focus_panel.file_tree",
        "聚焦文件树面板",
        "显示并聚焦文件树面板",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::FileTreePanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('e')).with_priority(100)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::GitPanel),
        "workspace.focus_panel.git",
        "聚焦 Git 面板",
        "显示并聚焦 Git 面板",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::GitPanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('g')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::OutlinePanel),
        "workspace.focus_panel.outline",
        "聚焦大纲面板",
        "显示并聚焦大纲面板",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::OutlinePanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('o')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::ProjectSearchPanel),
        "workspace.focus_panel.project_search",
        "聚焦项目搜索面板",
        "显示并聚焦项目搜索面板",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::ProjectSearchPanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('f')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::LanguageServersPanel),
        "workspace.focus_panel.language_servers",
        "聚焦语言服务器面板",
        "显示并聚焦语言服务器面板",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(
                FocusTarget::LanguageServersPanel,
            ))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('l')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::TerminalPanel),
        "workspace.focus_panel.terminal",
        "聚焦终端面板",
        "显示并聚焦终端面板",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::TerminalPanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_char('.')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::DebugPanel),
        "workspace.focus_panel.debug",
        "聚焦调试面板",
        "显示并聚焦调试面板",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::DebugPanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('d')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::NotificationPanel),
        "workspace.focus_panel.notification",
        "聚焦通知面板",
        "显示并聚焦通知面板",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::NotificationPanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('n')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceCloseFocused,
        "workspace.close_focused",
        "关闭或隐藏",
        "关闭或隐藏当前聚焦组件",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::CloseFocused)),
        &[CommandShortcut::new(ShortcutScope::Global, meta_char('w')).with_priority(120)],
    ),
];
