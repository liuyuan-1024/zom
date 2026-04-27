//! 悬浮层相关命令规范声明。

use crate::OverlayTarget;
use crate::command::kind::{CommandKind, CommandKindSpec};

pub const SPECS: &[CommandKindSpec] = &[CommandKindSpec::new(
    CommandKind::WorkspaceFocusOverlay(OverlayTarget::Settings),
    "workspace.focus_overlay.settings",
    "聚焦设置悬浮面板",
    "显示并聚焦设置悬浮面板。",
)];
