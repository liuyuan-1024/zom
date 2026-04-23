//! 工作台通用动作命令规范声明。

use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope,
    types::{primary_char, primary_shift_char},
};
use crate::{CommandInvocation, WorkspaceAction};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceQuitApp,
        "workspace.quit_app",
        "退出应用",
        "退出当前应用。",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::QuitApp)),
        &[CommandShortcut::new(ShortcutScope::Global, primary_char('q')).with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceMinimizeWindow,
        "workspace.minimize_window",
        "最小化窗口",
        "将当前应用窗口最小化。",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::MinimizeWindow)),
        &[CommandShortcut::new(ShortcutScope::Global, primary_char('m')).with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceOpenProjectPicker,
        "workspace.open_project_picker",
        "打开项目选择器",
        "打开项目目录选择器。",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::OpenProjectPicker)),
        &[CommandShortcut::new(ShortcutScope::Global, primary_shift_char('p')).with_priority(80)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceCloseFocused,
        "workspace.close_focused",
        "关闭或隐藏",
        "关闭或隐藏当前聚焦组件",
        Buildability::Static(|| CommandInvocation::from(WorkspaceAction::CloseFocused)),
        &[CommandShortcut::new(ShortcutScope::Global, primary_char('w')).with_priority(120)],
    ),
];
