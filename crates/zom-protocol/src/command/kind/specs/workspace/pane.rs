//！ 聚焦窗格的命令元信息

use crate::FocusTarget;
use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope, types::primary_char,
};
use crate::{CommandInvocation, WorkspaceAction};

pub const SPECS: &[CommandKindSpec] = &[CommandKindSpec::new(
    CommandKind::WorkspaceFocusPanel(FocusTarget::Editor),
    "workspace.focus_panel.editor",
    "聚焦编辑器",
    "聚焦编辑器窗格。",
    Buildability::Static(|| {
        CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::Editor))
    }),
    &[CommandShortcut::new(ShortcutScope::Global, primary_char('e')).with_priority(100)],
)];
