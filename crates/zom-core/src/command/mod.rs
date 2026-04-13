//! 命令协议层。
//! 这里表达的是“用户想做什么”，而不是“具体如何执行”。

mod editor;
mod workspace;

pub use editor::EditorCommand;
pub use workspace::WorkspaceCommand;

/// 跨系统共享的顶层命令。
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::{Command, EditorCommand, WorkspaceCommand};

    #[test]
    fn command_kind_helpers_match_the_payload() {
        let editor = Command::from(EditorCommand::Undo);
        let workspace = Command::from(WorkspaceCommand::OpenCommandPalette);

        assert!(editor.is_editor());
        assert!(!editor.is_workspace());
        assert!(workspace.is_workspace());
        assert!(!workspace.is_editor());
    }
}
