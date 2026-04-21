//! 悬浮层相关命令规范声明。

use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope, types::meta_char,
};
use crate::{CommandInvocation, OverlayTarget, WorkspaceAction};

pub const SPECS: &[CommandKindSpec] = &[CommandKindSpec::new(
    CommandKind::WorkspaceFocusOverlay(OverlayTarget::Settings),
    "workspace.focus_overlay.settings",
    "聚焦设置悬浮面板",
    "显示并聚焦设置悬浮面板。",
    Buildability::Static(|| {
        CommandInvocation::from(WorkspaceAction::FocusOverlay(OverlayTarget::Settings))
    }),
    &[CommandShortcut::new(ShortcutScope::Global, meta_char(',')).with_priority(80)],
)];
