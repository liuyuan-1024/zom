//! 命令协议层。
//! 这里表达的是“用户想做什么”，而不是“具体如何执行”。

mod catalog;
mod editor;
mod workspace;

pub use catalog::{
    CommandId, CommandKey, CommandMeta, CommandShortcut, CommandSpec, ShortcutScope,
    command_from_key, command_id, command_key, command_meta, command_spec, command_spec_by_id,
    command_spec_by_key, command_specs, default_shortcut_bindings, default_shortcuts,
};
pub use editor::EditorCommand;
pub use workspace::{FileTreeCommand, TabCommand, WorkspaceCommand};

/// 跨系统共享的顶层命令。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    /// 作用于文本编辑器的命令。
    Editor(EditorCommand),
    /// 作用于工作台或界面的命令。
    Workspace(WorkspaceCommand),
}

impl Command {
    /// 判断当前命令是否属于编辑器领域。
    pub fn is_editor(&self) -> bool {
        matches!(self, Self::Editor(_))
    }

    /// 判断当前命令是否属于工作台领域。
    pub fn is_workspace(&self) -> bool {
        matches!(self, Self::Workspace(_))
    }
}

impl From<EditorCommand> for Command {
    fn from(command: EditorCommand) -> Self {
        Self::Editor(command)
    }
}

impl From<WorkspaceCommand> for Command {
    fn from(command: WorkspaceCommand) -> Self {
        Self::Workspace(command)
    }
}

impl From<FileTreeCommand> for WorkspaceCommand {
    fn from(command: FileTreeCommand) -> Self {
        Self::FileTree(command)
    }
}

impl From<TabCommand> for WorkspaceCommand {
    fn from(command: TabCommand) -> Self {
        Self::Tab(command)
    }
}

impl From<FileTreeCommand> for Command {
    fn from(command: FileTreeCommand) -> Self {
        Self::Workspace(WorkspaceCommand::from(command))
    }
}

impl From<TabCommand> for Command {
    fn from(command: TabCommand) -> Self {
        Self::Workspace(WorkspaceCommand::from(command))
    }
}

#[cfg(test)]
mod tests {
    use crate::FocusTarget;

    use super::{Command, EditorCommand, FileTreeCommand, TabCommand, WorkspaceCommand};

    #[test]
    fn command_kind_helpers_match_the_payload() {
        let editor = Command::from(EditorCommand::Undo);
        let workspace = Command::from(WorkspaceCommand::FocusPanel(FocusTarget::Palette));

        assert!(editor.is_editor());
        assert!(!editor.is_workspace());
        assert!(workspace.is_workspace());
        assert!(!workspace.is_editor());
    }

    #[test]
    fn file_tree_and_tab_commands_are_promoted_to_top_level_command() {
        let file_tree = Command::from(FileTreeCommand::SelectNext);
        let tab = Command::from(TabCommand::CloseActiveTab);

        assert_eq!(
            file_tree,
            Command::Workspace(WorkspaceCommand::FileTree(FileTreeCommand::SelectNext))
        );
        assert_eq!(
            tab,
            Command::Workspace(WorkspaceCommand::Tab(TabCommand::CloseActiveTab))
        );
    }
}
