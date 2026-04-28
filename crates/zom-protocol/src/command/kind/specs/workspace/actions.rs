//! 工作台通用动作命令规范声明。

use crate::command::kind::{CommandKind, CommandKindSpec};

/// 工作台通用动作命令规范列表。
pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceQuitApp,
        "workspace.quit_app",
        "退出应用",
        "退出当前应用。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceMinimizeWindow,
        "workspace.minimize_window",
        "最小化窗口",
        "将当前应用窗口最小化。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceOpenProjectPicker,
        "workspace.open_project_picker",
        "打开项目选择器",
        "打开项目目录选择器。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceCloseFocused,
        "workspace.close_focused",
        "关闭或隐藏",
        "关闭或隐藏当前聚焦组件",
    ),
];
