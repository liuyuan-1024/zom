use std::collections::HashSet;
use std::path::PathBuf;

use zom_core::{
    BufferId, Command, FocusTarget, InputContext, InputResolution, Keystroke,
    command::{FileTreeCommand, TabCommand, WorkspaceCommand},
};
use zom_input::resolve_default;

use crate::{
    state::{FileTreeNodeKind, FileTreeState, PaneState, TabState, TitleBarState, ToolBarState},
    utils,
};

/// 需要在 UI 层执行的副作用动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopUiAction {
    /// 打开项目目录选择器。
    OpenProjectPicker,
    /// 打开设置入口。
    OpenSettings,
}

/// 桌面端根界面使用的应用状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAppState {
    /// 顶部标题栏状态。
    pub title_bar: TitleBarState,
    /// 底部工具栏信息。
    pub tool_bar: ToolBarState,
    /// 左侧文件树内容。
    pub file_tree: FileTreeState,
    /// 窗格
    pub pane: PaneState,
    /// 当前聚焦目标。
    pub focused_target: FocusTarget,
    /// 当前可见的工作台面板集合。
    pub visible_panels: HashSet<FocusTarget>,
    /// 当前打开项目的名称。
    pub project_name: String,
    /// 当前打开项目的根目录绝对路径。
    pub project_root: PathBuf,
    /// 下一帧需要应用的焦点请求（仅应用层内部可写）。
    pub(crate) pending_focus_target: Option<FocusTarget>,
    /// 下一帧需要由 UI 层执行的一次性动作。
    pub(crate) pending_ui_action: Option<DesktopUiAction>,
}

impl DesktopAppState {
    /// 切换当前工作区项目，并重建文件树。
    pub fn switch_project(&mut self, project_root: impl Into<PathBuf>) {
        let project_root = utils::normalize_workspace_root(project_root.into());
        self.project_name = utils::project_name_from_root(&project_root);
        self.project_root = project_root.clone();
        self.file_tree = FileTreeState::from_workspace_root(&project_root);

        // 旧项目打开的标签页路径不再可信，切换项目时统一清空。
        self.pane.tabs.clear();
        self.pane.active_tab_index = None;
        self.tool_bar.cursor = "1:1".into();
    }

    /// 确保文件树存在初始选中项（用于首次获取键盘焦点前）。
    pub fn ensure_file_tree_selection(&mut self) -> bool {
        self.file_tree.ensure_selection()
    }

    /// 处理一个键盘输入，解析成命令后统一交给应用层分发。
    pub fn handle_keystroke(&mut self, keystroke: &Keystroke) -> bool {
        let context = InputContext::new(self.focused_target);
        let resolution = resolve_default(keystroke, &context);
        let InputResolution::Command(command) = resolution else {
            return false;
        };
        self.handle_command(command);
        true
    }

    /// 返回指定面板当前是否可见。
    pub fn is_panel_visible(&self, target: FocusTarget) -> bool {
        if !target.is_visibility_managed_panel() {
            return true;
        }
        self.visible_panels.contains(&target)
    }

    /// 消费一次待处理的焦点目标（供 UI 层在下一帧应用）。
    pub fn take_pending_focus_target(&mut self) -> Option<FocusTarget> {
        self.pending_focus_target.take()
    }

    /// 消费一次待处理 UI 动作（供 UI 层在下一帧应用）。
    pub fn take_pending_ui_action(&mut self) -> Option<DesktopUiAction> {
        self.pending_ui_action.take()
    }

    /// 处理文件树节点激活，并同步工作区状态。
    pub fn handle_file_tree_node_activate(&mut self, relative_path: &str, kind: FileTreeNodeKind) {
        match kind {
            FileTreeNodeKind::Directory => self.file_tree.toggle_directory(relative_path),
            FileTreeNodeKind::File => {
                self.file_tree.activate_file(relative_path);
                self.open_file_in_pane(relative_path);
                self.focus_editor();
            }
        }
    }

    /// 统一处理顶层命令，并分发到对应领域。
    pub fn handle_command(&mut self, command: Command) {
        match command {
            Command::Workspace(command) => self.handle_workspace_command(command),
            Command::Editor(_command) => {
                // TODO: 编辑器命令分发接入后在此处理。
            }
        }
    }

    /// 处理工作台命令，并分发到细分子域。
    fn handle_workspace_command(&mut self, command: WorkspaceCommand) {
        match command {
            WorkspaceCommand::FocusPanel(target) => self.focus_panel(target),
            WorkspaceCommand::CloseFocused => self.close_focused(),
            WorkspaceCommand::OpenProjectPicker => {
                self.pending_ui_action = Some(DesktopUiAction::OpenProjectPicker);
            }
            WorkspaceCommand::OpenSettings => {
                self.pending_ui_action = Some(DesktopUiAction::OpenSettings);
            }
            WorkspaceCommand::OpenCodeActions => {
                // TODO: 代码动作入口接入后在这里打开。
            }
            WorkspaceCommand::StartDebugging => {
                // TODO: 调试入口接入后在这里触发。
            }
            WorkspaceCommand::FileTree(command) => self.handle_file_tree_command(command),
            WorkspaceCommand::Tab(command) => self.handle_tab_command(command),
        }
    }

    /// 处理文件树命令，并同步工作区状态。
    fn handle_file_tree_command(&mut self, command: FileTreeCommand) {
        match command {
            FileTreeCommand::SelectPrev => self.file_tree.select_prev_visible(),
            FileTreeCommand::SelectNext => self.file_tree.select_next_visible(),
            FileTreeCommand::ExpandOrDescend => self.file_tree.expand_or_descend_selected(),
            FileTreeCommand::CollapseOrAscend => self.file_tree.collapse_or_ascend_selected(),
            FileTreeCommand::ActivateSelection => {
                if let Some((relative_path, kind)) = self.file_tree.selected_node() {
                    self.handle_file_tree_node_activate(&relative_path, kind);
                } else {
                    self.file_tree.select_next_visible();
                }
            }
        }
    }

    /// 聚焦到指定面板：若面板当前隐藏，则先显示后聚焦。
    fn focus_panel(&mut self, target: FocusTarget) {
        self.set_panel_visible(target, true);
        self.focused_target = target;
        self.pending_focus_target = Some(target);
        self.prepare_panel_focus(target);
    }

    /// 关闭当前聚焦组件：优先关闭焦点面板，其次关闭当前标签页。
    fn close_focused(&mut self) {
        if self.focused_target.is_visibility_managed_panel()
            && self.is_panel_visible(self.focused_target)
        {
            self.set_panel_visible(self.focused_target, false);
            self.focus_editor();
            return;
        }

        if self.focused_target == FocusTarget::Editor {
            self.handle_tab_command(TabCommand::CloseActiveTab);
        }
    }

    /// 处理标签页命令。
    fn handle_tab_command(&mut self, command: TabCommand) {
        match command {
            TabCommand::CloseActiveTab => self.close_active_tab(),
            TabCommand::ActivatePrevTab => {
                // TODO: 标签页切换接入后在此处理。
            }
            TabCommand::ActivateNextTab => {
                // TODO: 标签页切换接入后在此处理。
            }
        }
    }

    fn close_active_tab(&mut self) {
        let Some(active_index) = self.pane.active_tab_index else {
            return;
        };
        if active_index >= self.pane.tabs.len() {
            self.pane.active_tab_index = None;
            return;
        }

        self.pane.tabs.remove(active_index);
        if self.pane.tabs.is_empty() {
            self.pane.active_tab_index = None;
            return;
        }

        let next_index = active_index.min(self.pane.tabs.len() - 1);
        self.pane.active_tab_index = Some(next_index);
    }

    fn focus_editor(&mut self) {
        self.focused_target = FocusTarget::Editor;
        self.pending_focus_target = Some(FocusTarget::Editor);
    }

    /// 在面板接收焦点前执行必要的准备动作。
    fn prepare_panel_focus(&mut self, target: FocusTarget) {
        if target == FocusTarget::FileTreePanel {
            self.ensure_file_tree_selection();
        }
    }

    fn set_panel_visible(&mut self, target: FocusTarget, visible: bool) {
        if !target.is_visibility_managed_panel() {
            return;
        }

        if visible {
            self.visible_panels.insert(target);
        } else {
            self.visible_panels.remove(&target);
        }
    }

    /// 在当前 Pane 打开文件：已打开则切换并刷新内容，未打开则新增标签页。
    fn open_file_in_pane(&mut self, relative_path: &str) {
        let absolute_path = utils::workspace_file_absolute_path(&self.project_root, relative_path);
        let (buffer_lines, line_ending, cursor) = utils::load_buffer_preview(&absolute_path);

        self.tool_bar.cursor = cursor;
        self.tool_bar.line_ending = line_ending;

        if let Some(tab_index) = self
            .pane
            .tabs
            .iter()
            .position(|tab| tab.relative_path == relative_path)
        {
            if let Some(existing_tab) = self.pane.tabs.get_mut(tab_index) {
                existing_tab.buffer_lines = buffer_lines;
            }
            self.pane.active_tab_index = Some(tab_index);
            return;
        }

        let next_buffer_id = self
            .pane
            .tabs
            .iter()
            .map(|tab| tab.buffer_id.value())
            .max()
            .unwrap_or(0)
            + 1;

        self.pane.tabs.push(TabState {
            buffer_id: BufferId::new(next_buffer_id),
            title: utils::file_name_from_path(relative_path),
            relative_path: relative_path.to_string(),
            buffer_lines,
        });
        self.pane.active_tab_index = Some(self.pane.tabs.len() - 1);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use zom_core::{
        Command, FocusTarget, Keystroke,
        command::{FileTreeCommand, WorkspaceCommand},
    };

    use super::{DesktopAppState, DesktopUiAction};
    use crate::state::FileTreeNodeKind;

    #[test]
    fn activating_file_tree_file_opens_tab_and_activates_it() {
        let mut state = DesktopAppState::from_current_workspace();
        let before_len = state.pane.tabs.len();

        state.handle_file_tree_node_activate("crates/zom-app/src/lib.rs", FileTreeNodeKind::File);

        assert_eq!(state.pane.tabs.len(), before_len + 1);
        let active_tab = state.pane.active_tab().expect("active tab should exist");
        assert_eq!(active_tab.relative_path, "crates/zom-app/src/lib.rs");
        assert!(!active_tab.buffer_lines.is_empty());
        assert_eq!(state.focused_target, FocusTarget::Editor);
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    fn keyboard_select_and_activate_opens_file_in_pane() {
        let workspace = create_temp_workspace("keyboard-open");
        fs::write(workspace.join("main.rs"), "fn main() {}").expect("write main.rs");

        let mut state = DesktopAppState::from_current_workspace();
        state.switch_project(workspace.clone());

        state.file_tree.select_next_visible();
        state.file_tree.select_next_visible();
        state.pane.tabs.clear();
        state.pane.active_tab_index = None;

        state.handle_command(Command::from(FileTreeCommand::ActivateSelection));

        assert_eq!(state.pane.tabs.len(), 1);
        let active_tab = state.pane.active_tab().expect("active tab should exist");
        assert_eq!(active_tab.relative_path, "main.rs");

        remove_temp_workspace(workspace);
    }

    #[test]
    fn focus_panel_shows_file_tree_and_requests_focus() {
        let mut state = DesktopAppState::from_current_workspace();
        state.visible_panels.remove(&FocusTarget::FileTreePanel);
        state.file_tree.roots[0].is_selected = false;

        state.handle_command(Command::from(WorkspaceCommand::FocusPanel(
            FocusTarget::FileTreePanel,
        )));

        assert!(state.is_panel_visible(FocusTarget::FileTreePanel));
        assert_eq!(state.focused_target, FocusTarget::FileTreePanel);
        assert_eq!(
            state.take_pending_focus_target(),
            Some(FocusTarget::FileTreePanel)
        );
        assert_eq!(
            state.file_tree.selected_node().map(|(path, _)| path),
            Some("".to_string())
        );
    }

    #[test]
    fn close_focused_hides_focused_file_tree_and_falls_back_to_editor() {
        let mut state = DesktopAppState::from_current_workspace();
        state.focused_target = FocusTarget::FileTreePanel;
        state.visible_panels.insert(FocusTarget::FileTreePanel);

        state.handle_command(Command::from(WorkspaceCommand::CloseFocused));

        assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
        assert_eq!(state.focused_target, FocusTarget::Editor);
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    fn close_focused_closes_active_tab_when_editor_is_focused() {
        let mut state = DesktopAppState::from_current_workspace();
        state.focused_target = FocusTarget::Editor;
        state.pane.tabs = vec![zom_app_test_tab("a.rs"), zom_app_test_tab("b.rs")];
        state.pane.active_tab_index = Some(1);

        state.handle_command(Command::from(WorkspaceCommand::CloseFocused));

        assert_eq!(state.pane.tabs.len(), 1);
        assert_eq!(state.pane.tabs[0].relative_path, "a.rs");
        assert_eq!(state.pane.active_tab_index, Some(0));
    }

    #[test]
    fn keyboard_shortcut_resolves_via_input_layer_and_dispatches_workspace_command() {
        let mut state = DesktopAppState::from_current_workspace();
        let keystroke = Keystroke::new(
            zom_core::KeyCode::Char('b'),
            zom_core::Modifiers::new(false, false, false, true),
        );

        let handled = state.handle_keystroke(&keystroke);

        assert!(handled);
        assert!(state.is_panel_visible(FocusTarget::FileTreePanel));
        assert_eq!(state.focused_target, FocusTarget::FileTreePanel);
        assert_eq!(
            state.take_pending_focus_target(),
            Some(FocusTarget::FileTreePanel)
        );
    }

    #[test]
    fn keyboard_shortcut_can_request_open_project_picker_ui_action() {
        let mut state = DesktopAppState::from_current_workspace();
        let keystroke = Keystroke::new(
            zom_core::KeyCode::Char('p'),
            zom_core::Modifiers::new(false, false, true, true),
        );

        let handled = state.handle_keystroke(&keystroke);

        assert!(handled);
        assert_eq!(
            state.take_pending_ui_action(),
            Some(DesktopUiAction::OpenProjectPicker)
        );
    }

    #[test]
    fn switch_project_reloads_real_file_tree_and_clears_tabs() {
        let workspace = create_temp_workspace("switch-project-tree");
        fs::create_dir_all(workspace.join("src")).expect("create src directory");
        fs::write(workspace.join("src/lib.rs"), "pub fn answer() -> u8 { 42 }")
            .expect("write lib.rs");

        let mut state = DesktopAppState::from_current_workspace();
        state.pane.tabs.push(zom_app_test_tab("old.rs"));
        state.pane.active_tab_index = Some(0);

        state.switch_project(workspace.clone());

        assert_eq!(
            state.project_root,
            crate::utils::normalize_workspace_root(workspace.clone())
        );
        assert_eq!(state.pane.tabs.len(), 0);
        assert!(state.pane.active_tab_index.is_none());
        assert_eq!(
            state.file_tree.roots[0]
                .children
                .iter()
                .map(|node| node.path.as_str())
                .collect::<Vec<_>>(),
            vec!["src"]
        );

        remove_temp_workspace(workspace);
    }

    #[test]
    fn open_file_reads_from_selected_project_root() {
        let workspace = create_temp_workspace("open-file-from-root");
        fs::create_dir_all(workspace.join("src")).expect("create src directory");
        fs::write(workspace.join("src/main.rs"), "fn main() {}").expect("write main.rs");

        let mut state = DesktopAppState::from_current_workspace();
        state.switch_project(workspace.clone());
        state.handle_file_tree_node_activate("src/main.rs", FileTreeNodeKind::File);

        let active_tab = state.pane.active_tab().expect("active tab should exist");
        assert_eq!(active_tab.relative_path, "src/main.rs");
        assert_eq!(active_tab.buffer_lines[0], "fn main() {}");

        remove_temp_workspace(workspace);
    }

    fn create_temp_workspace(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time should be after unix epoch")
            .as_nanos();
        let workspace = std::env::temp_dir().join(format!("zom-desktop-state-{name}-{timestamp}"));
        fs::create_dir_all(&workspace).expect("create temp workspace");
        workspace
    }

    fn remove_temp_workspace(path: PathBuf) {
        fs::remove_dir_all(path).expect("remove temp workspace");
    }

    fn zom_app_test_tab(relative_path: &str) -> crate::state::TabState {
        crate::state::TabState {
            buffer_id: zom_core::BufferId::new(999),
            title: "old".into(),
            relative_path: relative_path.into(),
            buffer_lines: vec!["old".into()],
        }
    }
}
