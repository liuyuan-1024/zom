use zom_core::{
    BufferId, Command,
    command::{FileTreeCommand, WorkspaceCommand},
};

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
    /// 当前打开项目的名称。
    pub project_name: String,
}

impl DesktopAppState {
    /// 确保文件树存在初始选中项（用于首次获取键盘焦点前）。
    pub fn ensure_file_tree_selection(&mut self) -> bool {
        self.file_tree.ensure_selection()
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
            WorkspaceCommand::FileTree(command) => self.handle_file_tree_command(command),
            WorkspaceCommand::FocusFileTree
            | WorkspaceCommand::FocusCommandPalette
            | WorkspaceCommand::Tab(_) => {
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
    use zom_core::{Command, command::FileTreeCommand};

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
}
