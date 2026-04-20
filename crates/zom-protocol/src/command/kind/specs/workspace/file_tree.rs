//! 文件树相关命令规范声明。

use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope, types::plain,
};
use crate::{CommandInvocation, FileTreeAction, FocusTarget, KeyCode};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceFileTreeSelectPrev,
        "workspace.file_tree.select_prev",
        "File Tree Select Previous",
        "Move file-tree selection to the previous visible node.",
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
        "File Tree Select Next",
        "Move file-tree selection to the next visible node.",
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
        "File Tree Expand Or Descend",
        "Expand selected folder or descend into its first child.",
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
        "File Tree Collapse Or Ascend",
        "Collapse selected folder or move selection to parent node.",
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
        "File Tree Activate Selection",
        "Activate selected file-tree node.",
        Buildability::Static(|| CommandInvocation::from(FileTreeAction::ActivateSelection)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Enter),
        )
        .with_priority(110)],
    ),
];
