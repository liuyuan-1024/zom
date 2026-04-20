//! 应用初始状态装配逻辑。

use std::collections::HashSet;

use zom_protocol::{FocusTarget, PaneId, Position};

use crate::{
    state::{
        DesktopAppState, FileTreeState, PaneState, TitleBarIcon, TitleBarState, ToolBarEntry,
        ToolBarIcon, ToolBarState,
    },
    utils,
};

impl DesktopAppState {
    /// 基于当前工作区构造应用初始状态。
    pub fn from_current_workspace() -> Self {
        let workspace_root = utils::normalize_workspace_root(utils::detect_workspace_root());
        let workspace_name = utils::project_name_from_root(&workspace_root);

        Self {
            title_bar: TitleBarState {
                right_icons: vec![TitleBarIcon::Settings],
            },
            tool_bar: ToolBarState {
                left_tools: vec![
                    ToolBarEntry {
                        icon: ToolBarIcon::FileTree,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::GitBranch,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::Outline,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::ProjectSearch,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::LanguageServers,
                    },
                ],
                cursor: Position::zero(),
                language: "Rust".into(),
                line_ending: "LF".into(),
                encoding: "UTF-8".into(),
                right_tools: vec![
                    ToolBarEntry {
                        icon: ToolBarIcon::Terminal,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::Debug,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::Notification,
                    },
                ],
            },
            file_tree: FileTreeState::from_workspace_root(&workspace_root),
            project_name: workspace_name.clone(),
            project_root: workspace_root,
            pane: PaneState {
                id: PaneId::new(1),
                tabs: Vec::new(),
                active_tab_index: None,
            },
            focused_target: FocusTarget::Editor,
            visible_panels: default_visible_panels(),
            active_overlay: None,
            pending_focus_target: Some(FocusTarget::Editor),
            pending_ui_action: None,
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

#[cfg(test)]
mod tests {
    use zom_text::{detect_line_ending, split_lines};

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
    }

    #[test]
    fn initial_state_requests_editor_focus() {
        let mut state = DesktopAppState::from_current_workspace();
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    fn split_lines_preserves_blank_lines() {
        let lines = split_lines("a\n\nb\n");

        assert_eq!(lines, vec!["a", "", "b", ""]);
    }

    #[test]
    fn detect_line_ending_distinguishes_crlf_and_lf() {
        assert_eq!(detect_line_ending("a\r\nb\r\n"), "CRLF");
        assert_eq!(detect_line_ending("a\nb\n"), "LF");
    }
}
