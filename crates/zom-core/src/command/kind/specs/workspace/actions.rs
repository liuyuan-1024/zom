use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope,
    types::{meta_char, meta_shift_char, plain},
};
use crate::{CommandInvocation, KeyCode, WorkspaceAction};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceOpenProjectPicker,
        "workspace.open_project_picker",
        "Open Project Picker",
        "Open the project folder picker.",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::OpenProjectPicker)),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('p')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceOpenSettings,
        "workspace.open_settings",
        "Open Settings",
        "Open application settings.",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::OpenSettings)),
        &[CommandShortcut::new(ShortcutScope::Global, meta_char(',')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceOpenCodeActions,
        "workspace.open_code_actions",
        "Open Code Actions",
        "Open the code actions menu.",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::OpenCodeActions)),
        &[CommandShortcut::new(ShortcutScope::Global, meta_char('.')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceStartDebugging,
        "workspace.start_debugging",
        "Start Debugging",
        "Start or continue debugging.",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::StartDebugging)),
        &[CommandShortcut::new(ShortcutScope::Global, plain(KeyCode::F(5))).with_priority(80)],
    ),
];
