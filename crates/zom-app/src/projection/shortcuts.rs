use zom_core::Command;

/// 将命令语义投影为快捷键提示文案（供 UI 展示层消费）。
pub fn shortcut_hint(command: &Command) -> Option<String> {
    zom_input::shortcut_hint(command)
}

#[cfg(test)]
mod tests {
    use zom_core::{Command, command::WorkspaceCommand};

    use super::shortcut_hint;

    #[test]
    fn shortcut_hint_projects_from_input_registry() {
        let command = Command::from(WorkspaceCommand::CloseFocused);
        assert_eq!(shortcut_hint(&command), Some("Cmd+W".to_string()));
    }
}
