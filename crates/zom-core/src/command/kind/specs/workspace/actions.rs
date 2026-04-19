use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope,
    types::meta_shift_char,
};
use crate::{CommandInvocation, WorkspaceAction};

pub const SPECS: &[CommandKindSpec] = &[CommandKindSpec::new(
    CommandKind::WorkspaceOpenProjectPicker,
    "workspace.open_project_picker",
    "Open Project Picker",
    "Open the project folder picker.",
    Buildability::Static(|| CommandInvocation::from(WorkspaceAction::OpenProjectPicker)),
    &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('p')).with_priority(80)],
)];
