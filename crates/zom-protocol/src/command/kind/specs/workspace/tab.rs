//! 标签页相关命令规范声明。

use crate::command::kind::{CommandKind, CommandKindSpec};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceTabCloseActive,
        "workspace.tab.close_active",
        "关闭活动标签页",
        "关闭当前活动标签页。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceTabActivatePrev,
        "workspace.tab.activate_prev",
        "激活前一个标签页",
        "激活前一个标签页。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceTabActivateNext,
        "workspace.tab.activate_next",
        "激活下一个标签页",
        "激活下一个标签页。",
    ),
];
