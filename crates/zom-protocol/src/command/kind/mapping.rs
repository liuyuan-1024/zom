//! 命令语义族到运行时调用的映射。

use crate::{
    CommandInvocation, EditorAction, FileTreeAction, NotificationAction, TabAction,
    WorkspaceAction, command::kind::CommandKind,
};

/// 将可静态构造的 `CommandKind` 映射为 `CommandInvocation`。
///
/// 返回 `None` 表示该语义族需要动态载荷，无法在此静态构造。
pub(super) fn invocation_for_kind(kind: CommandKind) -> Option<CommandInvocation> {
    match kind {
        CommandKind::EditorInsertText => None,
        CommandKind::EditorInsertNewline => {
            Some(CommandInvocation::from(EditorAction::InsertNewline))
        }
        CommandKind::EditorInsertIndent => {
            Some(CommandInvocation::from(EditorAction::InsertIndent))
        }
        CommandKind::EditorOutdent => Some(CommandInvocation::from(EditorAction::Outdent)),
        CommandKind::EditorMoveLeft => Some(CommandInvocation::from(EditorAction::MoveLeft)),
        CommandKind::EditorMoveRight => Some(CommandInvocation::from(EditorAction::MoveRight)),
        CommandKind::EditorMoveUp => Some(CommandInvocation::from(EditorAction::MoveUp)),
        CommandKind::EditorMoveDown => Some(CommandInvocation::from(EditorAction::MoveDown)),
        CommandKind::EditorMoveToStart => Some(CommandInvocation::from(EditorAction::MoveToStart)),
        CommandKind::EditorMoveToEnd => Some(CommandInvocation::from(EditorAction::MoveToEnd)),
        CommandKind::EditorMovePageUp => Some(CommandInvocation::from(EditorAction::MovePageUp)),
        CommandKind::EditorMovePageDown => {
            Some(CommandInvocation::from(EditorAction::MovePageDown))
        }
        CommandKind::EditorSelectLeft => Some(CommandInvocation::from(EditorAction::SelectLeft)),
        CommandKind::EditorSelectRight => Some(CommandInvocation::from(EditorAction::SelectRight)),
        CommandKind::EditorSelectUp => Some(CommandInvocation::from(EditorAction::SelectUp)),
        CommandKind::EditorSelectDown => Some(CommandInvocation::from(EditorAction::SelectDown)),
        CommandKind::EditorSelectToStart => {
            Some(CommandInvocation::from(EditorAction::SelectToStart))
        }
        CommandKind::EditorSelectToEnd => Some(CommandInvocation::from(EditorAction::SelectToEnd)),
        CommandKind::EditorSelectPageUp => {
            Some(CommandInvocation::from(EditorAction::SelectPageUp))
        }
        CommandKind::EditorSelectPageDown => {
            Some(CommandInvocation::from(EditorAction::SelectPageDown))
        }
        CommandKind::EditorSelectAll => Some(CommandInvocation::from(EditorAction::SelectAll)),
        CommandKind::EditorDeleteBackward => {
            Some(CommandInvocation::from(EditorAction::DeleteBackward))
        }
        CommandKind::EditorDeleteForward => {
            Some(CommandInvocation::from(EditorAction::DeleteForward))
        }
        CommandKind::EditorDeleteWordBackward => {
            Some(CommandInvocation::from(EditorAction::DeleteWordBackward))
        }
        CommandKind::EditorDeleteWordForward => {
            Some(CommandInvocation::from(EditorAction::DeleteWordForward))
        }
        CommandKind::EditorCopy => Some(CommandInvocation::from(EditorAction::Copy)),
        CommandKind::EditorCut => Some(CommandInvocation::from(EditorAction::Cut)),
        CommandKind::EditorPaste => Some(CommandInvocation::from(EditorAction::Paste)),
        CommandKind::EditorUndo => Some(CommandInvocation::from(EditorAction::Undo)),
        CommandKind::EditorRedo => Some(CommandInvocation::from(EditorAction::Redo)),
        CommandKind::WorkspaceQuitApp => Some(CommandInvocation::from(WorkspaceAction::QuitApp)),
        CommandKind::WorkspaceMinimizeWindow => {
            Some(CommandInvocation::from(WorkspaceAction::MinimizeWindow))
        }
        CommandKind::WorkspaceOpenProjectPicker => {
            Some(CommandInvocation::from(WorkspaceAction::OpenProjectPicker))
        }
        CommandKind::WorkspaceSaveActiveBuffer => {
            Some(CommandInvocation::from(WorkspaceAction::SaveActiveBuffer))
        }
        CommandKind::WorkspaceFocusPanel(target) => {
            Some(CommandInvocation::from(WorkspaceAction::FocusPanel(target)))
        }
        CommandKind::WorkspaceFocusOverlay(target) => Some(CommandInvocation::from(
            WorkspaceAction::FocusOverlay(target),
        )),
        CommandKind::WorkspaceCloseFocused => {
            Some(CommandInvocation::from(WorkspaceAction::CloseFocused))
        }
        CommandKind::WorkspaceFileTreeSelectPrev => {
            Some(CommandInvocation::from(FileTreeAction::SelectPrev))
        }
        CommandKind::WorkspaceFileTreeSelectNext => {
            Some(CommandInvocation::from(FileTreeAction::SelectNext))
        }
        CommandKind::WorkspaceFileTreeExpandOrDescend => {
            Some(CommandInvocation::from(FileTreeAction::ExpandOrDescend))
        }
        CommandKind::WorkspaceFileTreeCollapseOrAscend => {
            Some(CommandInvocation::from(FileTreeAction::CollapseOrAscend))
        }
        CommandKind::WorkspaceFileTreeActivateSelection => {
            Some(CommandInvocation::from(FileTreeAction::ActivateSelection))
        }
        CommandKind::WorkspaceTabCloseActive => {
            Some(CommandInvocation::from(TabAction::CloseActiveTab))
        }
        CommandKind::WorkspaceTabActivatePrev => {
            Some(CommandInvocation::from(TabAction::ActivatePrevTab))
        }
        CommandKind::WorkspaceTabActivateNext => {
            Some(CommandInvocation::from(TabAction::ActivateNextTab))
        }
        CommandKind::WorkspaceNotificationMarkSelectedRead => Some(CommandInvocation::from(
            NotificationAction::MarkSelectedRead,
        )),
        CommandKind::WorkspaceNotificationMarkAllRead => {
            Some(CommandInvocation::from(NotificationAction::MarkAllRead))
        }
        CommandKind::WorkspaceNotificationClearAll => {
            Some(CommandInvocation::from(NotificationAction::ClearAll))
        }
        CommandKind::WorkspaceNotificationClearRead => {
            Some(CommandInvocation::from(NotificationAction::ClearRead))
        }
        CommandKind::WorkspaceNotificationFocusUnreadError => Some(CommandInvocation::from(
            NotificationAction::FocusUnreadError,
        )),
        CommandKind::WorkspaceNotificationSelectPrev => {
            Some(CommandInvocation::from(NotificationAction::SelectPrev))
        }
        CommandKind::WorkspaceNotificationSelectNext => {
            Some(CommandInvocation::from(NotificationAction::SelectNext))
        }
    }
}
