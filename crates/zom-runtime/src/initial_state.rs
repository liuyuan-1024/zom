//! 应用初始状态装配逻辑。

use std::collections::{HashMap, HashSet};

use zom_protocol::{
    FocusTarget, PaneId, Position, ToolBarSide, WorkspaceAction, command::CommandInvocation,
};

use crate::{
    state::{
        DesktopAppState, FileTreeState, PaneState, TitleBarAction, TitleBarState, ToolBarEntry,
        ToolBarState,
    },
    workspace_paths,
};

impl DesktopAppState {
    /// 基于当前工作区构造应用初始状态。
    pub fn from_current_workspace() -> Self {
        let workspace_root =
            workspace_paths::normalize_workspace_root(workspace_paths::detect_workspace_root());
        let workspace_name = workspace_paths::project_name_from_root(&workspace_root);

        Self {
            title_bar: TitleBarState {
                right_actions: vec![TitleBarAction {
                    command: CommandInvocation::from(WorkspaceAction::FocusOverlay(
                        zom_protocol::OverlayTarget::Settings,
                    )),
                }],
            },
            tool_bar: ToolBarState {
                left_tools: panel_toolbar_entries(ToolBarSide::Left),
                cursor: Position::zero(),
                language: String::new(),
                right_tools: panel_toolbar_entries(ToolBarSide::Right),
            },
            file_tree: FileTreeState::from_workspace_root(&workspace_root),
            project_name: workspace_name.clone(),
            project_root: workspace_root,
            pane: PaneState {
                id: PaneId::new(1),
                tabs: Vec::new(),
                active_tab_index: None,
            },
            editor_states: HashMap::new(),
            focused_target: FocusTarget::Editor,
            visible_panels: default_visible_panels(),
            active_overlay: None,
            notifications: Vec::new(),
            active_toast_notification: None,
            active_status_notification: None,
            unread_notification_count: 0,
            selected_notification_id: None,
            pending_notification_selection_id: None,
            pending_focus_target: Some(FocusTarget::Editor),
            pending_ui_action: None,
            next_notification_id: 1,
        }
    }
}

/// 构造工作台默认可见面板集合。
fn default_visible_panels() -> HashSet<FocusTarget> {
    FocusTarget::VISIBILITY_MANAGED_PANELS
        .into_iter()
        .filter(|target| target.is_visible_by_default())
        .collect()
}

fn panel_toolbar_entries(side: ToolBarSide) -> Vec<ToolBarEntry> {
    FocusTarget::VISIBILITY_MANAGED_PANELS
        .into_iter()
        .filter(|target| target.tool_bar_side() == Some(side))
        .map(|target| ToolBarEntry {
            command: CommandInvocation::from(WorkspaceAction::FocusPanel(target)),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use zom_protocol::FocusTarget;

    use crate::state::DesktopAppState;

    #[test]
    fn initial_state_has_buffers_and_file_tree_content() {
        let state = DesktopAppState::from_current_workspace();

        assert!(!state.file_tree.roots.is_empty());
        assert!(state.pane.tabs.is_empty());
    }

    #[test]
    fn initial_state_starts_without_active_tab() {
        let state = DesktopAppState::from_current_workspace();
        assert!(state.pane.active_tab().is_none());
        assert_eq!(state.tool_bar.language, "");
    }

    #[test]
    fn initial_state_requests_editor_focus() {
        let mut state = DesktopAppState::from_current_workspace();
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }
}
