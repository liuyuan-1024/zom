//! 工作台通用动作命令规范声明。

use crate::command::kind::types::meta_char;
use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope,
    types::meta_shift_char,
};
use crate::{CommandInvocation, WorkspaceAction};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceOpenProjectPicker,
        "workspace.open_project_picker",
        "打开项目选择器",
        "打开项目目录选择器。",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::OpenProjectPicker)),
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('p')).with_priority(80)],
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
