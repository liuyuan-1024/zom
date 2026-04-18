use std::collections::HashSet;

use zom_core::{
    BufferId, Command, FocusTarget, InputContext, InputResolution, Keystroke,
    command::{FileTreeCommand, WorkspaceCommand},
};
use zom_input::resolve_default;

use crate::{
    state::{FileTreeNodeKind, FileTreeState, PaneState, TabState, TitleBarState, ToolBarState},
    utils,
};

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
    /// 下一帧需要应用的焦点请求（仅应用层内部可写）。
    pub(crate) pending_focus_target: Option<FocusTarget>,
}

impl DesktopAppState {
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

    /// 处理文件树节点激活，并同步工作区状态。
    pub fn handle_file_tree_node_activate(&mut self, relative_path: &str, kind: FileTreeNodeKind) {
        match kind {
            FileTreeNodeKind::Directory => self.file_tree.toggle_directory(relative_path),
            FileTreeNodeKind::File => {
                self.file_tree.activate_file(relative_path);
                self.open_file_in_pane(relative_path);
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
            WorkspaceCommand::TogglePanel(target) => self.toggle_panel(target),
            WorkspaceCommand::FileTree(command) => self.handle_file_tree_command(command),
            WorkspaceCommand::Tab(_) => {
                // TODO: 工作台聚焦与标签页命令接入后在此处理。
            }
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

    /// 切换指定面板显示状态，并维护焦点回退规则。
    fn toggle_panel(&mut self, target: FocusTarget) {
        let is_visible = self.is_panel_visible(target);
        self.set_panel_visible(target, !is_visible);

        if is_visible {
            if self.focused_target == target {
                self.focused_target = FocusTarget::Editor;
                self.pending_focus_target = Some(FocusTarget::Editor);
            }
            return;
        }

        self.focused_target = target;
        self.pending_focus_target = Some(target);
        self.prepare_panel_focus(target);
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
        let absolute_path = utils::workspace_file_absolute_path(relative_path);
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
    use zom_core::{
        Command, FocusTarget, Keystroke,
        command::{FileTreeCommand, WorkspaceCommand},
    };

    use super::DesktopAppState;
    use crate::state::FileTreeNodeKind;

    #[test]
    fn activating_file_tree_file_opens_tab_and_activates_it() {
        let mut state = DesktopAppState::sample();
        let before_len = state.pane.tabs.len();

        state.handle_file_tree_node_activate("crates/zom-app/src/lib.rs", FileTreeNodeKind::File);

        assert_eq!(state.pane.tabs.len(), before_len + 1);
        let active_tab = state.pane.active_tab().expect("active tab should exist");
        assert_eq!(active_tab.relative_path, "crates/zom-app/src/lib.rs");
        assert!(!active_tab.buffer_lines.is_empty());
    }

    #[test]
    fn keyboard_select_and_activate_opens_file_in_pane() {
        let mut state = DesktopAppState::sample();

        state.file_tree.activate_file("crates/zom-app/src/lib.rs");
        state.pane.tabs.clear();
        state.pane.active_tab_index = None;

        state.handle_command(Command::from(FileTreeCommand::ActivateSelection));

        assert_eq!(state.pane.tabs.len(), 1);
        let active_tab = state.pane.active_tab().expect("active tab should exist");
        assert_eq!(active_tab.relative_path, "crates/zom-app/src/lib.rs");
    }

    #[test]
    fn focus_panel_shows_file_tree_and_requests_focus() {
        let mut state = DesktopAppState::sample();
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
    fn toggle_panel_hides_focused_file_tree_and_falls_back_to_editor() {
        let mut state = DesktopAppState::sample();
        state.focused_target = FocusTarget::FileTreePanel;
        state.visible_panels.insert(FocusTarget::FileTreePanel);

        state.handle_command(Command::from(WorkspaceCommand::TogglePanel(
            FocusTarget::FileTreePanel,
        )));

        assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
        assert_eq!(state.focused_target, FocusTarget::Editor);
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    fn keyboard_shortcut_resolves_via_input_layer_and_dispatches_workspace_command() {
        let mut state = DesktopAppState::sample();
        let keystroke = Keystroke::new(
            zom_core::KeyCode::Char('b'),
            zom_core::Modifiers::new(false, false, false, true),
        );

        let handled = state.handle_keystroke(&keystroke);

        assert!(handled);
        assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
    }
}
