//! 命令到快捷键文案的展示投影。

use zom_core::CommandInvocation;

/// 将命令语义投影为快捷键提示文案（供 UI 展示层消费）。
pub fn shortcut_hint(command: &CommandInvocation) -> Option<String> {
    zom_core::input::shortcut_hint(command)
}

#[cfg(test)]
mod tests {
    use zom_core::{CommandInvocation, WorkspaceAction};

    use super::shortcut_hint;

    #[test]
    fn shortcut_hint_projects_from_input_registry() {
        let command = CommandInvocation::from(WorkspaceAction::CloseFocused);
        assert_eq!(shortcut_hint(&command), Some("Cmd+W".to_string()));
    }
}
