//! 命令语义到工作台状态的投影。

use zom_protocol::{CommandInvocation, FocusTarget, WorkspaceAction};

use crate::state::{DesktopAppState, PanelDock, panel_dock};

/// 从命令语义中提取面板目标（若存在）。
pub fn panel_target_for_command(command: &CommandInvocation) -> Option<FocusTarget> {
    match command {
        CommandInvocation::Workspace(WorkspaceAction::FocusPanel(target)) => Some(*target),
        _ => None,
    }
}

/// 将命令语义投影为面板停靠位置。
pub fn command_dock(command: &CommandInvocation) -> Option<PanelDock> {
    panel_target_for_command(command).and_then(panel_dock)
}

/// 判断某条命令在当前状态下是否处于激活态（用于高亮）。
pub fn command_is_active(state: &DesktopAppState, command: &CommandInvocation) -> bool {
    panel_target_for_command(command)
        .map(|target| state.is_panel_visible(target))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use zom_protocol::{CommandInvocation, FocusTarget, WorkspaceAction};

    use super::{command_dock, panel_target_for_command};
    use crate::state::PanelDock;

    #[test]
    fn focus_panel_command_projects_to_target_and_dock() {
        let command =
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::TerminalPanel));
        assert_eq!(
            panel_target_for_command(&command),
            Some(FocusTarget::TerminalPanel)
        );
        assert_eq!(command_dock(&command), Some(PanelDock::Bottom));
    }

    #[test]
    fn non_panel_command_has_no_target_projection() {
        let command = CommandInvocation::from(WorkspaceAction::OpenProjectPicker);
        assert_eq!(panel_target_for_command(&command), None);
        assert_eq!(command_dock(&command), None);
    }
}
