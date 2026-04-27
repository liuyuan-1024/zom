//! 聚焦窗格的命令元信息。

use crate::FocusTarget;
use crate::command::kind::{CommandKind, CommandKindSpec};

pub const SPECS: &[CommandKindSpec] = &[CommandKindSpec::new(
    CommandKind::WorkspaceFocusPanel(FocusTarget::Editor),
    "workspace.focus_panel.editor",
    "聚焦编辑器",
    "聚焦编辑器窗格。",
)];
