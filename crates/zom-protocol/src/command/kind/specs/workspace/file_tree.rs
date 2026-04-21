//! 文件树相关命令规范声明。

use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope, types::plain,
};
use crate::{CommandInvocation, FileTreeAction, FocusTarget, KeyCode};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeSelectPrev,
        "workspace.file_tree.select_prev",
        "文件树选择上一项",
        "将文件树选择移动到上一个可见节点。",
        Buildability::Static(|| CommandInvocation::from(FileTreeAction::SelectPrev)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Up),
        )
        .with_priority(110)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeSelectNext,
        "workspace.file_tree.select_next",
        "文件树选择下一项",
        "将文件树选择移动到下一个可见节点。",
        Buildability::Static(|| CommandInvocation::from(FileTreeAction::SelectNext)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Down),
        )
        .with_priority(110)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeExpandOrDescend,
        "workspace.file_tree.expand_or_descend",
        "文件树展开或下探",
        "展开选中的文件夹或进入其第一个子节点。",
        Buildability::Static(|| CommandInvocation::from(FileTreeAction::ExpandOrDescend)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Right),
        )
        .with_priority(110)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeCollapseOrAscend,
        "workspace.file_tree.collapse_or_ascend",
        "文件树折叠或上探",
        "折叠选中的文件夹或移动选择到父节点。",
        Buildability::Static(|| CommandInvocation::from(FileTreeAction::CollapseOrAscend)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Left),
        )
        .with_priority(110)],
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeActivateSelection,
        "workspace.file_tree.activate_selection",
        "文件树激活选中项",
        "激活选中的文件树节点。",
        Buildability::Static(|| CommandInvocation::from(FileTreeAction::ActivateSelection)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Enter),
        )
        .with_priority(110)],
    ),
];
