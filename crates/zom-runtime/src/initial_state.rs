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
    ///
    /// 该函数集中定义“冷启动默认布局”：标题栏、工具栏、面板显隐与焦点。
    /// 后续恢复会话或项目切换都应在此约定之上做增量覆盖。
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
            editor_histories: HashMap::new(),
            focused_target: FocusTarget::Editor,
            visible_panels: default_visible_panels(),
            active_overlay: None,
            active_toast: None,
            pending_focus_target: Some(FocusTarget::Editor),
            pending_ui_action: None,
            next_toast_id: 1,
        }
    }
}

/// 构造工作台默认可见面板集合。
///
/// 可见性来源于 `FocusTarget` 目录，不在 runtime 重复硬编码，避免双份配置漂移。
fn default_visible_panels() -> HashSet<FocusTarget> {
    FocusTarget::VISIBILITY_MANAGED_PANELS
        .into_iter()
        .filter(|target| target.is_visible_by_default())
        .collect()
}

/// 生成工具栏某一侧的面板切换入口。
///
/// 入口顺序继承 `VISIBILITY_MANAGED_PANELS` 的声明顺序，保证 UI 排列稳定。
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
    /// 计算状态文件树结果。
    fn initial_state_has_buffers_and_file_tree_content() {
        let state = DesktopAppState::from_current_workspace();

        assert!(!state.file_tree.roots.is_empty());
        assert!(state.pane.tabs.is_empty());
    }

    #[test]
    /// 计算状态标签页结果。
    fn initial_state_starts_without_active_tab() {
        let state = DesktopAppState::from_current_workspace();
        assert!(state.pane.active_tab().is_none());
        assert_eq!(state.tool_bar.language, "");
    }

    #[test]
    /// 计算状态编辑器焦点结果。
    fn initial_state_requests_editor_focus() {
        let mut state = DesktopAppState::from_current_workspace();
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }
}
