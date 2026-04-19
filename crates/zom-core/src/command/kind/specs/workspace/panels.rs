use crate::FocusTarget;
use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope,
    types::{ctrl_char, meta_char, meta_shift_char},
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
        "Focus Command Palette",
        "Move focus to the command palette.",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::Palette))
        }),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::FileTreePanel),
        "workspace.focus_panel.file_tree",
        "Focus File Tree Panel",
        "Show file tree panel and move focus to it.",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::FileTreePanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_char('b')).with_priority(100)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::GitPanel),
        "workspace.focus_panel.git",
        "Focus Git Panel",
        "Show Git panel and move focus to it.",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::GitPanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('g')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::OutlinePanel),
        "workspace.focus_panel.outline",
        "Focus Outline Panel",
        "Show outline panel and move focus to it.",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::OutlinePanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('o')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::ProjectSearchPane),
        "workspace.focus_panel.project_search",
        "Focus Project Search Panel",
        "Show project search panel and move focus to it.",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::ProjectSearchPane))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('f')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::LanguageServers),
        "workspace.focus_panel.language_servers",
        "Focus Language Servers Panel",
        "Show language servers panel and move focus to it.",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::LanguageServers))
        }),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::TerminalPanel),
        "workspace.focus_panel.terminal",
        "Focus Terminal Panel",
        "Show terminal panel and move focus to it.",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::TerminalPanel))
        }),
        &[CommandShortcut::new(ShortcutScope::Global, ctrl_char('`')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::DebugPanel),
        "workspace.focus_panel.debug",
        "Focus Debug Panel",
        "Show debug panel and move focus to it.",
        Buildability::Static(|| {
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::DebugPanel))
        }),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFocusPanel(FocusTarget::NotificationPanel),
        "workspace.focus_panel.notification",
        "Focus Notification Panel",
        "Show notification panel and move focus to it.",
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
