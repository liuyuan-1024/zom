use crate::command::catalog::{
    CommandKey, CommandShortcut, CommandSpec, ShortcutScope,
    types::{meta_char, meta_shift_char},
};
use crate::{Command, command::WorkspaceCommand};
use crate::{KeyCode, command::catalog::types::plain};

pub const SPECS: &[CommandSpec] = &[
    CommandSpec::new(
        CommandKey::WorkspaceOpenProjectPicker,
        "workspace.open_project_picker",
        "Open Project Picker",
        "Open the project folder picker.",
        || Some(Command::from(WorkspaceCommand::OpenProjectPicker)),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('p')).with_priority(80)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceOpenSettings,
        "workspace.open_settings",
        "Open Settings",
        "Open application settings.",
        || Some(Command::from(WorkspaceCommand::OpenSettings)),
        &[CommandShortcut::new(ShortcutScope::Global, meta_char(',')).with_priority(80)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceOpenCodeActions,
        "workspace.open_code_actions",
        "Open Code Actions",
        "Open the code actions menu.",
        || Some(Command::from(WorkspaceCommand::OpenCodeActions)),
        &[CommandShortcut::new(ShortcutScope::Global, meta_char('.')).with_priority(80)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceStartDebugging,
        "workspace.start_debugging",
        "Start Debugging",
        "Start or continue debugging.",
        || Some(Command::from(WorkspaceCommand::StartDebugging)),
        &[CommandShortcut::new(ShortcutScope::Global, plain(KeyCode::F(5))).with_priority(80)],
    ),
];
