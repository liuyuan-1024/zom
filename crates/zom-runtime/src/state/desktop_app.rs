//! 桌面应用状态与命令分发主状态机。

use std::collections::HashSet;
use std::path::PathBuf;

use zom_editor::apply_editor_invocation;
use zom_protocol::input::resolve_default;
use zom_protocol::{
    BufferId, CommandInvocation, EditorInvocation, FocusTarget, InputContext, InputResolution,
    Keystroke, OverlayTarget, Position,
    command::{FileTreeAction, TabAction, WorkspaceAction},
};

use crate::{
    buffer_preview,
    state::{
        FileTreeNodeKind, FileTreeState, PaneState, PanelDock, TabState, TitleBarState,
        ToolBarState, dock_targets, panel_dock,
    },
    workspace_paths,
};

/// 需要在 UI 层执行的副作用动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopUiAction {
    /// 打开项目目录选择器。
    OpenProjectPicker,
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
    /// 当前可见的悬浮层。
    pub active_overlay: Option<OverlayTarget>,
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
        let project_root = workspace_paths::normalize_workspace_root(project_root.into());
        self.project_name = workspace_paths::project_name_from_root(&project_root);
        self.project_root = project_root.clone();
        self.file_tree = FileTreeState::from_workspace_root(&project_root);

        // 旧项目打开的标签页路径不再可信，切换项目时统一清空。
        self.pane.tabs.clear();
        self.pane.active_tab_index = None;
        self.tool_bar.cursor = Position::zero();
    }

    /// 确保文件树存在初始选中项（用于首次获取键盘焦点前）。
    pub fn ensure_file_tree_selection(&mut self) -> bool {
        self.file_tree.ensure_selection()
    }

    /// 处理一个键盘输入，解析成命令后统一交给应用层分发。
    pub fn handle_keystroke(&mut self, keystroke: &Keystroke) -> bool {
        let context = InputContext::new(self.focused_target);
        let resolution = resolve_default(keystroke, &context);
        match resolution {
            InputResolution::Command(command) => {
                self.handle_command(command);
                true
            }
            InputResolution::InsertText(text) => {
                self.handle_command(CommandInvocation::from(EditorInvocation::insert_text(text)));
                true
            }
            InputResolution::Noop => false,
        }
    }

    /// 返回指定面板当前是否可见。
    pub fn is_panel_visible(&self, target: FocusTarget) -> bool {
        if !target.is_visibility_managed_panel() {
            return true;
        }
        self.visible_panels.contains(&target)
    }

    /// 返回指定停靠区域当前可见的面板目标。
    pub fn visible_panel_in_dock(&self, dock: PanelDock) -> Option<FocusTarget> {
        dock_targets(dock)
            .iter()
            .copied()
            .find(|target| self.is_panel_visible(*target))
    }

    /// 隐藏指定停靠区域当前可见面板。
    /// 若该面板当前持有焦点，会自动回退焦点到编辑区。
    pub fn hide_visible_panel_in_dock(&mut self, dock: PanelDock) -> bool {
        let Some(target) = self.visible_panel_in_dock(dock) else {
            return false;
        };
        self.set_panel_visible(target, false);
        if self.focused_target == target {
            self.focus_editor();
        }
        true
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
    pub fn handle_command(&mut self, command: CommandInvocation) {
        match command {
            CommandInvocation::Workspace(command) => self.handle_workspace_command(command),
            CommandInvocation::Editor(command) => self.handle_editor_command(command),
        }
    }

    /// 处理工作台命令，并分发到细分子域。
    fn handle_workspace_command(&mut self, command: WorkspaceAction) {
        match command {
            WorkspaceAction::FocusPanel(target) => self.focus_panel(target),
            WorkspaceAction::FocusOverlay(target) => self.focus_overlay(target),
            WorkspaceAction::CloseFocused => self.close_focused(),
            WorkspaceAction::OpenProjectPicker => {
                self.pending_ui_action = Some(DesktopUiAction::OpenProjectPicker);
            }
            WorkspaceAction::FileTree(command) => self.handle_file_tree_command(command),
            WorkspaceAction::Tab(command) => self.handle_tab_command(command),
        }
    }

    /// 处理编辑器命令，并把结果写回当前活动标签页与工具栏状态。
    fn handle_editor_command(&mut self, command: EditorInvocation) {
        let Some(active_index) = self.pane.active_tab_index else {
            return;
        };
        let Some(active_tab) = self.pane.tabs.get_mut(active_index) else {
            self.pane.active_tab_index = None;
            return;
        };

        let result =
            apply_editor_invocation(&active_tab.editor_state, self.tool_bar.cursor, &command);
        active_tab.editor_state = result.state;
        self.tool_bar.cursor = result.cursor;
        self.tool_bar.line_ending = active_tab.line_ending();
    }

    /// 处理文件树命令，并同步工作区状态。
    fn handle_file_tree_command(&mut self, command: FileTreeAction) {
        match command {
            FileTreeAction::SelectPrev => self.file_tree.select_prev_visible(),
            FileTreeAction::SelectNext => self.file_tree.select_next_visible(),
            FileTreeAction::ExpandOrDescend => self.file_tree.expand_or_descend_selected(),
            FileTreeAction::CollapseOrAscend => self.file_tree.collapse_or_ascend_selected(),
            FileTreeAction::ActivateSelection => {
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
        self.hide_panels_in_same_dock(target);
        self.set_panel_visible(target, true);
        self.active_overlay = None;
        self.focused_target = target;
        self.pending_focus_target = Some(target);
        self.prepare_panel_focus(target);
    }

    /// 聚焦到指定悬浮层：显示并聚焦。
    fn focus_overlay(&mut self, target: OverlayTarget) {
        self.active_overlay = Some(target);
        self.focused_target = target.into();
        self.pending_focus_target = Some(self.focused_target);
    }

    /// 关闭当前聚焦组件：优先关闭焦点悬浮层，其次关闭焦点面板，最后关闭当前标签页。
    fn close_focused(&mut self) {
        if self.focused_target.is_overlay() && self.active_overlay.is_some() {
            self.active_overlay = None;
            self.focus_editor();
            return;
        }

        if self.focused_target.is_visibility_managed_panel()
            && self.is_panel_visible(self.focused_target)
        {
            self.set_panel_visible(self.focused_target, false);
            self.focus_editor();
            return;
        }

        if self.focused_target == FocusTarget::Editor {
            self.handle_tab_command(TabAction::CloseActiveTab);
        }
    }

    /// 处理标签页命令。
    fn handle_tab_command(&mut self, command: TabAction) {
        match command {
            TabAction::CloseActiveTab => self.close_active_tab(),
            TabAction::ActivatePrevTab => self.activate_prev_tab(),
            TabAction::ActivateNextTab => self.activate_next_tab(),
        }
    }

    fn close_active_tab(&mut self) {
        let Some(active_index) = self.pane.active_tab_index else {
            self.sync_tool_bar_with_active_tab();
            return;
        };
        if active_index >= self.pane.tabs.len() {
            self.pane.active_tab_index = None;
            self.sync_tool_bar_with_active_tab();
            return;
        }

        self.pane.tabs.remove(active_index);
        if self.pane.tabs.is_empty() {
            self.pane.active_tab_index = None;
            self.sync_tool_bar_with_active_tab();
            return;
        }

        let next_index = active_index.min(self.pane.tabs.len() - 1);
        self.pane.active_tab_index = Some(next_index);
        self.sync_tool_bar_with_active_tab();
    }

    fn activate_prev_tab(&mut self) {
        let tab_count = self.pane.tabs.len();
        if tab_count == 0 {
            self.pane.active_tab_index = None;
            self.sync_tool_bar_with_active_tab();
            return;
        }

        let current_index = self
            .pane
            .active_tab_index
            .unwrap_or(0)
            .min(tab_count.saturating_sub(1));
        let prev_index = if current_index == 0 {
            tab_count - 1
        } else {
            current_index - 1
        };
        self.pane.active_tab_index = Some(prev_index);
        self.sync_tool_bar_with_active_tab();
    }

    fn activate_next_tab(&mut self) {
        let tab_count = self.pane.tabs.len();
        if tab_count == 0 {
            self.pane.active_tab_index = None;
            self.sync_tool_bar_with_active_tab();
            return;
        }

        let current_index = self
            .pane
            .active_tab_index
            .unwrap_or(0)
            .min(tab_count.saturating_sub(1));
        let next_index = (current_index + 1) % tab_count;
        self.pane.active_tab_index = Some(next_index);
        self.sync_tool_bar_with_active_tab();
    }

    fn sync_tool_bar_with_active_tab(&mut self) {
        if let Some(active_tab) = self.pane.active_tab() {
            self.tool_bar.cursor = active_tab.editor_state.selection().active();
            self.tool_bar.line_ending = active_tab.line_ending();
        } else {
            self.tool_bar.cursor = Position::zero();
            self.tool_bar.line_ending = "LF".into();
        }
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

    fn hide_panels_in_same_dock(&mut self, target: FocusTarget) {
        let Some(dock) = panel_dock(target) else {
            return;
        };
        self.visible_panels
            .retain(|panel| panel_dock(*panel) != Some(dock));
    }

    /// 在当前 Pane 打开文件：已打开则切换并刷新内容，未打开则新增标签页。
    fn open_file_in_pane(&mut self, relative_path: &str) {
        let absolute_path =
            workspace_paths::workspace_file_absolute_path(&self.project_root, relative_path);
        let buffer_preview::BufferPreview {
            editor_state,
            line_ending,
            cursor,
        } = buffer_preview::load_buffer_preview(&absolute_path);

        self.tool_bar.cursor = cursor;
        self.tool_bar.line_ending = line_ending;

        if let Some(tab_index) = self
            .pane
            .tabs
            .iter()
            .position(|tab| tab.relative_path == relative_path)
        {
            if let Some(existing_tab) = self.pane.tabs.get_mut(tab_index) {
                existing_tab.editor_state = editor_state;
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
            title: workspace_paths::file_name_from_path(relative_path),
            relative_path: relative_path.to_string(),
            editor_state,
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

    use zom_protocol::{
        CommandInvocation, EditorAction, EditorInvocation, FocusTarget, KeyCode, Keystroke,
        Modifiers, OverlayTarget, Position,
        command::{FileTreeAction, TabAction, WorkspaceAction},
    };

    use super::{DesktopAppState, DesktopUiAction};
    use crate::state::{FileTreeNodeKind, PanelDock};

    fn shortcut_for(command: CommandInvocation) -> Keystroke {
        zom_protocol::input::default_shortcut_registry()
            .bindings()
            .iter()
            .find(|binding| binding.command == command)
            .map(|binding| binding.keystroke)
            .unwrap_or_else(|| panic!("default shortcut should exist for command: {command:?}"))
    }

    #[test]
    fn activating_file_tree_file_opens_tab_and_activates_it() {
        let mut state = DesktopAppState::from_current_workspace();
        let before_len = state.pane.tabs.len();

        state.handle_file_tree_node_activate(
            "crates/zom-runtime/src/lib.rs",
            FileTreeNodeKind::File,
        );

        assert_eq!(state.pane.tabs.len(), before_len + 1);
        let active_tab = state.pane.active_tab().expect("active tab should exist");
        assert_eq!(active_tab.relative_path, "crates/zom-runtime/src/lib.rs");
        assert!(!active_tab.buffer_lines().is_empty());
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

        state.handle_command(CommandInvocation::from(FileTreeAction::ActivateSelection));

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

        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
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

        state.handle_command(CommandInvocation::from(WorkspaceAction::CloseFocused));

        assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
        assert_eq!(state.focused_target, FocusTarget::Editor);
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    fn close_focused_closes_active_tab_when_editor_is_focused() {
        let mut state = DesktopAppState::from_current_workspace();
        state.focused_target = FocusTarget::Editor;
        state.pane.tabs = vec![zom_runtime_test_tab("a.rs"), zom_runtime_test_tab("b.rs")];
        state.pane.active_tab_index = Some(1);

        state.handle_command(CommandInvocation::from(WorkspaceAction::CloseFocused));

        assert_eq!(state.pane.tabs.len(), 1);
        assert_eq!(state.pane.tabs[0].relative_path, "a.rs");
        assert_eq!(state.pane.active_tab_index, Some(0));
    }

    #[test]
    fn tab_activation_commands_cycle_tabs_and_sync_toolbar_state() {
        let mut state = DesktopAppState::from_current_workspace();
        state.pane.tabs = vec![
            zom_runtime_test_tab_with_text_and_cursor("a.rs", "first\nline", 0),
            zom_runtime_test_tab_with_text_and_cursor("b.rs", "x\r\ny", 1),
            zom_runtime_test_tab_with_text_and_cursor("c.rs", "tail", 2),
        ];
        state.pane.active_tab_index = Some(0);
        state.tool_bar.cursor = Position::new(99, 99);
        state.tool_bar.line_ending = "LF".into();

        state.handle_command(CommandInvocation::from(TabAction::ActivateNextTab));

        assert_eq!(state.pane.active_tab_index, Some(1));
        assert_eq!(state.tool_bar.cursor, Position::new(0, 1));
        assert_eq!(state.tool_bar.line_ending, "CRLF");

        state.handle_command(CommandInvocation::from(TabAction::ActivatePrevTab));
        assert_eq!(state.pane.active_tab_index, Some(0));
        assert_eq!(state.tool_bar.cursor, Position::zero());
        assert_eq!(state.tool_bar.line_ending, "LF");

        state.handle_command(CommandInvocation::from(TabAction::ActivatePrevTab));
        assert_eq!(state.pane.active_tab_index, Some(2));
        assert_eq!(state.tool_bar.cursor, Position::new(0, 2));
    }

    #[test]
    fn keyboard_shortcut_can_activate_next_tab() {
        let mut state = DesktopAppState::from_current_workspace();
        state.focused_target = FocusTarget::FileTreePanel;
        state.pane.tabs = vec![
            zom_runtime_test_tab_with_text_and_cursor("a.rs", "a", 0),
            zom_runtime_test_tab_with_text_and_cursor("b.rs", "bc", 1),
        ];
        state.pane.active_tab_index = Some(0);

        let next_tab = shortcut_for(CommandInvocation::from(TabAction::ActivateNextTab));
        let handled = state.handle_keystroke(&next_tab);

        assert!(handled);
        assert_eq!(state.pane.active_tab_index, Some(1));
        assert_eq!(state.tool_bar.cursor, Position::new(0, 1));
    }

    #[test]
    fn keyboard_shortcut_resolves_via_input_layer_and_dispatches_workspace_command() {
        let mut state = DesktopAppState::from_current_workspace();
        let keystroke = shortcut_for(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::FileTreePanel,
        )));

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
    fn editor_command_updates_active_tab_buffer_and_cursor() {
        let mut state = DesktopAppState::from_current_workspace();
        state.pane.tabs = vec![crate::state::TabState {
            buffer_id: zom_protocol::BufferId::new(1),
            title: "demo.rs".into(),
            relative_path: "demo.rs".into(),
            editor_state: zom_editor::EditorState::from_text("ab"),
        }];
        state.pane.active_tab_index = Some(0);
        state.tool_bar.cursor = Position::new(0, 1);

        state.handle_command(CommandInvocation::from(EditorInvocation::insert_text("X")));

        let active_tab = state.pane.active_tab().expect("active tab should exist");
        assert_eq!(active_tab.text(), "aXb");
        assert_eq!(state.tool_bar.cursor, Position::new(0, 2));

        state.handle_command(CommandInvocation::from(EditorAction::DeleteBackward));
        let active_tab = state.pane.active_tab().expect("active tab should exist");
        assert_eq!(active_tab.text(), "ab");
        assert_eq!(state.tool_bar.cursor, Position::new(0, 1));
    }

    #[test]
    fn plain_character_keystroke_in_editor_focus_inserts_text() {
        let mut state = DesktopAppState::from_current_workspace();
        state.pane.tabs = vec![crate::state::TabState {
            buffer_id: zom_protocol::BufferId::new(1),
            title: "demo.rs".into(),
            relative_path: "demo.rs".into(),
            editor_state: zom_editor::EditorState::from_text("ab"),
        }];
        state.pane.active_tab_index = Some(0);
        state.focused_target = FocusTarget::Editor;
        state.tool_bar.cursor = Position::new(0, 1);

        let handled =
            state.handle_keystroke(&Keystroke::new(KeyCode::Char('x'), Modifiers::default()));

        assert!(handled);
        let active_tab = state.pane.active_tab().expect("active tab should exist");
        assert_eq!(active_tab.text(), "axb");
        assert_eq!(state.tool_bar.cursor, Position::new(0, 2));
    }

    #[test]
    fn editor_command_without_active_tab_is_noop() {
        let mut state = DesktopAppState::from_current_workspace();
        state.pane.tabs.clear();
        state.pane.active_tab_index = None;
        state.tool_bar.cursor = Position::new(3, 7);

        state.handle_command(CommandInvocation::from(EditorInvocation::insert_text("x")));

        assert_eq!(state.pane.tabs.len(), 0);
        assert_eq!(state.pane.active_tab_index, None);
        assert_eq!(state.tool_bar.cursor, Position::new(3, 7));
    }

    #[test]
    fn keyboard_shortcut_can_focus_and_close_git_panel() {
        let mut state = DesktopAppState::from_current_workspace();
        let focus_git = shortcut_for(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::GitPanel,
        )));

        let handled_focus = state.handle_keystroke(&focus_git);

        assert!(handled_focus);
        assert!(state.is_panel_visible(FocusTarget::GitPanel));
        assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
        assert_eq!(state.focused_target, FocusTarget::GitPanel);
        assert_eq!(
            state.take_pending_focus_target(),
            Some(FocusTarget::GitPanel)
        );

        let close = shortcut_for(CommandInvocation::from(WorkspaceAction::CloseFocused));
        let handled_close = state.handle_keystroke(&close);

        assert!(handled_close);
        assert!(!state.is_panel_visible(FocusTarget::GitPanel));
        assert_eq!(state.focused_target, FocusTarget::Editor);
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    fn focus_panel_replaces_existing_left_slot_panel() {
        let mut state = DesktopAppState::from_current_workspace();
        assert!(state.is_panel_visible(FocusTarget::FileTreePanel));

        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::OutlinePanel,
        )));

        assert!(!state.is_panel_visible(FocusTarget::FileTreePanel));
        assert!(state.is_panel_visible(FocusTarget::OutlinePanel));
        assert_eq!(state.focused_target, FocusTarget::OutlinePanel);
    }

    #[test]
    fn focus_panel_replaces_existing_bottom_dock_panel() {
        let mut state = DesktopAppState::from_current_workspace();
        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::TerminalPanel,
        )));
        assert!(state.is_panel_visible(FocusTarget::TerminalPanel));

        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::DebugPanel,
        )));

        assert!(!state.is_panel_visible(FocusTarget::TerminalPanel));
        assert!(state.is_panel_visible(FocusTarget::DebugPanel));
        assert_eq!(state.focused_target, FocusTarget::DebugPanel);
    }

    #[test]
    fn right_and_bottom_docks_can_stay_visible_together() {
        let mut state = DesktopAppState::from_current_workspace();
        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::NotificationPanel,
        )));
        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::TerminalPanel,
        )));

        assert!(state.is_panel_visible(FocusTarget::NotificationPanel));
        assert!(state.is_panel_visible(FocusTarget::TerminalPanel));
    }

    #[test]
    fn close_focused_bottom_panel_keeps_right_panel_visible() {
        let mut state = DesktopAppState::from_current_workspace();
        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::NotificationPanel,
        )));
        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::TerminalPanel,
        )));

        state.handle_command(CommandInvocation::from(WorkspaceAction::CloseFocused));

        assert!(!state.is_panel_visible(FocusTarget::TerminalPanel));
        assert!(state.is_panel_visible(FocusTarget::NotificationPanel));
        assert_eq!(state.focused_target, FocusTarget::Editor);
    }

    #[test]
    fn hide_visible_panel_in_dock_hides_target_and_falls_back_to_editor() {
        let mut state = DesktopAppState::from_current_workspace();
        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::GitPanel,
        )));
        assert_eq!(state.focused_target, FocusTarget::GitPanel);

        let hidden = state.hide_visible_panel_in_dock(PanelDock::Left);

        assert!(hidden);
        assert!(!state.is_panel_visible(FocusTarget::GitPanel));
        assert_eq!(state.focused_target, FocusTarget::Editor);
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    fn keyboard_shortcut_can_focus_and_close_notification_panel() {
        let mut state = DesktopAppState::from_current_workspace();
        let focus_notification = shortcut_for(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::NotificationPanel),
        ));

        let handled_focus = state.handle_keystroke(&focus_notification);

        assert!(handled_focus);
        assert!(state.is_panel_visible(FocusTarget::NotificationPanel));
        assert_eq!(state.focused_target, FocusTarget::NotificationPanel);
        assert_eq!(
            state.take_pending_focus_target(),
            Some(FocusTarget::NotificationPanel)
        );

        let close = shortcut_for(CommandInvocation::from(WorkspaceAction::CloseFocused));
        let handled_close = state.handle_keystroke(&close);

        assert!(handled_close);
        assert!(!state.is_panel_visible(FocusTarget::NotificationPanel));
        assert_eq!(state.focused_target, FocusTarget::Editor);
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    fn keyboard_shortcut_can_request_open_project_picker_ui_action() {
        let mut state = DesktopAppState::from_current_workspace();
        let keystroke = shortcut_for(CommandInvocation::from(WorkspaceAction::OpenProjectPicker));

        let handled = state.handle_keystroke(&keystroke);

        assert!(handled);
        assert_eq!(
            state.take_pending_ui_action(),
            Some(DesktopUiAction::OpenProjectPicker)
        );
    }

    #[test]
    fn keyboard_shortcut_can_focus_settings_overlay() {
        let mut state = DesktopAppState::from_current_workspace();
        let keystroke = shortcut_for(CommandInvocation::from(WorkspaceAction::FocusOverlay(
            OverlayTarget::Settings,
        )));

        let handled = state.handle_keystroke(&keystroke);

        assert!(handled);
        assert_eq!(state.active_overlay, Some(OverlayTarget::Settings));
        assert_eq!(state.focused_target, FocusTarget::SettingsOverlay);
        assert_eq!(
            state.take_pending_focus_target(),
            Some(FocusTarget::SettingsOverlay)
        );
        assert_eq!(state.take_pending_ui_action(), None);
    }

    #[test]
    fn close_focused_closes_active_settings_overlay_first() {
        let mut state = DesktopAppState::from_current_workspace();
        state.handle_command(CommandInvocation::from(WorkspaceAction::FocusOverlay(
            OverlayTarget::Settings,
        )));

        let close = shortcut_for(CommandInvocation::from(WorkspaceAction::CloseFocused));
        let handled = state.handle_keystroke(&close);

        assert!(handled);
        assert_eq!(state.active_overlay, None);
        assert_eq!(state.focused_target, FocusTarget::Editor);
        assert_eq!(state.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    fn switch_project_reloads_real_file_tree_and_clears_tabs() {
        let workspace = create_temp_workspace("switch-project-tree");
        fs::create_dir_all(workspace.join("src")).expect("create src directory");
        fs::write(workspace.join("src/lib.rs"), "pub fn answer() -> u8 { 42 }")
            .expect("write lib.rs");

        let mut state = DesktopAppState::from_current_workspace();
        state.pane.tabs.push(zom_runtime_test_tab("old.rs"));
        state.pane.active_tab_index = Some(0);

        state.switch_project(workspace.clone());

        assert_eq!(
            state.project_root,
            crate::workspace_paths::normalize_workspace_root(workspace.clone())
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
        assert_eq!(active_tab.buffer_lines()[0], "fn main() {}");
        assert_eq!(state.tool_bar.cursor, Position::zero());

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

    fn zom_runtime_test_tab(relative_path: &str) -> crate::state::TabState {
        crate::state::TabState {
            buffer_id: zom_protocol::BufferId::new(999),
            title: "old".into(),
            relative_path: relative_path.into(),
            editor_state: zom_editor::EditorState::from_text("old"),
        }
    }

    fn zom_runtime_test_tab_with_text_and_cursor(
        relative_path: &str,
        text: &str,
        cursor_column: u32,
    ) -> crate::state::TabState {
        let mut editor_state = zom_editor::EditorState::from_text(text);
        let mut cursor = Position::zero();
        for _ in 0..cursor_column {
            let result = zom_editor::apply_editor_invocation(
                &editor_state,
                cursor,
                &EditorInvocation::from(EditorAction::MoveRight),
            );
            editor_state = result.state;
            cursor = result.cursor;
        }
        crate::state::TabState {
            buffer_id: zom_protocol::BufferId::new((1000 + cursor_column).into()),
            title: relative_path.into(),
            relative_path: relative_path.into(),
            editor_state,
        }
    }
}
