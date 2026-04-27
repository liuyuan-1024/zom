//! 命令到快捷键文案的展示投影。

use zom_input::shortcut_hint as input_shortcut_hint;
use zom_protocol::CommandInvocation;

/// 将命令语义投影为快捷键提示文案（供 UI 展示层消费）。
pub fn shortcut_hint(command: &CommandInvocation) -> Option<String> {
    input_shortcut_hint(command)
}

#[cfg(test)]
mod tests {
    use zom_protocol::{CommandInvocation, WorkspaceAction};

    use super::shortcut_hint;

    #[test]
    fn shortcut_hint_delegates_to_core_input_projection() {
        let command = CommandInvocation::from(WorkspaceAction::CloseFocused);
        assert_eq!(shortcut_hint(&command), zom_input::shortcut_hint(&command));
    }
}
