//! 文件树相关命令规范声明。

use crate::command::kind::{CommandKind, CommandKindSpec};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeSelectPrev,
        "workspace.file_tree.select_prev",
        "文件树选择上一项",
        "将文件树选择移动到上一个可见节点。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeSelectNext,
        "workspace.file_tree.select_next",
        "文件树选择下一项",
        "将文件树选择移动到下一个可见节点。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeExpandOrDescend,
        "workspace.file_tree.expand_or_descend",
        "文件树展开或下探",
        "展开选中的文件夹或进入其第一个子节点。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeCollapseOrAscend,
        "workspace.file_tree.collapse_or_ascend",
        "文件树折叠或上探",
        "折叠选中的文件夹或移动选择到父节点。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeActivateSelection,
        "workspace.file_tree.activate_selection",
        "文件树激活选中项",
        "激活选中的文件树节点。",
    ),
];
