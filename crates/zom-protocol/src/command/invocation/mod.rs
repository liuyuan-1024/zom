//! 命令调用载荷的顶层聚合入口。

//! 命令调用层。
//! 这里表达的是“本次要执行什么”，可携带动态 payload。

mod editor;
mod workspace;

pub use editor::{EditorAction, EditorInvocation};
pub use workspace::{FileTreeAction, TabAction, WorkspaceAction};

/// 跨系统共享的顶层命令调用。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandInvocation {
    /// 作用于文本编辑器的调用。
    Editor(EditorInvocation),
    /// 作用于工作台或界面的调用。
    Workspace(WorkspaceAction),
}

impl CommandInvocation {
    /// 判断当前调用是否属于编辑器领域。
    pub fn is_editor(&self) -> bool {
        matches!(self, Self::Editor(_))
    }

    /// 判断当前调用是否属于工作台领域。
    pub fn is_workspace(&self) -> bool {
        matches!(self, Self::Workspace(_))
    }
}

impl From<EditorInvocation> for CommandInvocation {
    fn from(command: EditorInvocation) -> Self {
        Self::Editor(command)
    }
}

impl From<EditorAction> for CommandInvocation {
    fn from(action: EditorAction) -> Self {
        Self::Editor(EditorInvocation::from(action))
    }
}

impl From<WorkspaceAction> for CommandInvocation {
    fn from(command: WorkspaceAction) -> Self {
        Self::Workspace(command)
    }
}

impl From<FileTreeAction> for CommandInvocation {
    fn from(action: FileTreeAction) -> Self {
        Self::Workspace(WorkspaceAction::from(action))
    }
}

impl From<TabAction> for CommandInvocation {
    fn from(action: TabAction) -> Self {
        Self::Workspace(WorkspaceAction::from(action))
    }
}

#[cfg(test)]
mod tests {
    use crate::FocusTarget;

    use super::{
        CommandInvocation, EditorAction, EditorInvocation, FileTreeAction, TabAction,
        WorkspaceAction,
    };

    #[test]
    fn command_kind_helpers_match_the_payload() {
        let editor = CommandInvocation::from(EditorAction::Undo);
        let workspace = CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::Palette));

        assert!(editor.is_editor());
        assert!(!editor.is_workspace());
        assert!(workspace.is_workspace());
        assert!(!workspace.is_editor());
    }

    #[test]
    fn file_tree_and_tab_actions_are_promoted_to_top_level_command() {
        let file_tree = CommandInvocation::from(FileTreeAction::SelectNext);
        let tab = CommandInvocation::from(TabAction::CloseActiveTab);

        assert_eq!(
            file_tree,
            CommandInvocation::Workspace(WorkspaceAction::FileTree(FileTreeAction::SelectNext))
        );
        assert_eq!(
            tab,
            CommandInvocation::Workspace(WorkspaceAction::Tab(TabAction::CloseActiveTab))
        );
    }

    #[test]
    fn editor_insert_text_keeps_payload() {
        let invocation = CommandInvocation::from(EditorInvocation::insert_text("hello"));

        assert_eq!(
            invocation,
            CommandInvocation::Editor(EditorInvocation::InsertText {
                text: "hello".to_string()
            })
        );
    }
}
