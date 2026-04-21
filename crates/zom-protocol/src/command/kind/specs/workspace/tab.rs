//! 标签页相关命令规范声明。

use crate::command::kind::{Buildability, CommandKind, CommandKindSpec};
use crate::{CommandInvocation, TabAction};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceTabCloseActive,
        "workspace.tab.close_active",
        "关闭活动标签页",
        "关闭当前活动标签页。",
        Buildability::Static(|| CommandInvocation::from(TabAction::CloseActiveTab)),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceTabActivatePrev,
        "workspace.tab.activate_prev",
        "激活前一个标签页",
        "激活前一个标签页。",
        Buildability::Static(|| CommandInvocation::from(TabAction::ActivatePrevTab)),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceTabActivateNext,
        "workspace.tab.activate_next",
        "激活下一个标签页",
        "激活下一个标签页。",
        Buildability::Static(|| CommandInvocation::from(TabAction::ActivateNextTab)),
        &[],
    ),
];
