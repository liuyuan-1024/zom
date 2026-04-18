use crate::command::catalog::{CommandKey, CommandShortcut, CommandSpec, ShortcutScope};
use crate::{Command, command::FileTreeCommand};
use crate::{FocusTarget, KeyCode, command::catalog::types::plain};

pub const SPECS: &[CommandSpec] = &[
    CommandSpec::new(
        CommandKey::WorkspaceFileTreeSelectPrev,
        "workspace.file_tree.select_prev",
        "File Tree Select Previous",
        "Move file-tree selection to the previous visible node.",
        || Some(Command::from(FileTreeCommand::SelectPrev)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Up),
        )
        .with_priority(110)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFileTreeSelectNext,
        "workspace.file_tree.select_next",
        "File Tree Select Next",
        "Move file-tree selection to the next visible node.",
        || Some(Command::from(FileTreeCommand::SelectNext)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Down),
        )
        .with_priority(110)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFileTreeExpandOrDescend,
        "workspace.file_tree.expand_or_descend",
        "File Tree Expand Or Descend",
        "Expand selected folder or descend into its first child.",
        || Some(Command::from(FileTreeCommand::ExpandOrDescend)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Right),
        )
        .with_priority(110)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFileTreeCollapseOrAscend,
        "workspace.file_tree.collapse_or_ascend",
        "File Tree Collapse Or Ascend",
        "Collapse selected folder or move selection to parent node.",
        || Some(Command::from(FileTreeCommand::CollapseOrAscend)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Left),
        )
        .with_priority(110)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFileTreeActivateSelection,
        "workspace.file_tree.activate_selection",
        "File Tree Activate Selection",
        "Activate selected file-tree node.",
        || Some(Command::from(FileTreeCommand::ActivateSelection)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            plain(KeyCode::Enter),
        )
        .with_priority(110)],
    ),
];
