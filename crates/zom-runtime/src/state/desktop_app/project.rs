//！ 项目切换与文件打开逻辑

use std::path::PathBuf;

use zom_protocol::BufferId;

use crate::{
    buffer_preview,
    state::{FileTreeNodeKind, FileTreeState, TabState},
    workspace_paths,
};

use super::DesktopAppState;

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
        self.clear_editor_states();
        self.sync_tool_bar_with_active_tab();
    }

    /// 处理文件树节点激活，并同步工作区状态。
    pub fn handle_file_tree_node_activate(&mut self, relative_path: &str, kind: FileTreeNodeKind) {
        match kind {
            FileTreeNodeKind::Directory => self.file_tree.toggle_directory(relative_path),
            FileTreeNodeKind::File => {
                self.file_tree.activate_file(relative_path);
                if self.open_file_in_pane(relative_path) {
                    self.focus_editor();
                }
            }
        }
    }

    /// 在当前 Pane 打开文件：已打开则切换并刷新内容，未打开则新增标签页。
    pub(super) fn open_file_in_pane(&mut self, relative_path: &str) -> bool {
        let absolute_path =
            workspace_paths::workspace_file_absolute_path(&self.project_root, relative_path);
        let Ok(buffer_preview::BufferPreview {
            editor_state,
            line_ending,
            ..
        }) = buffer_preview::load_buffer_preview(&absolute_path)
        else {
            return false;
        };
        let language = workspace_paths::language_from_path(relative_path);

        if let Some(tab_index) = self
            .pane
            .tabs
            .iter()
            .position(|tab| tab.relative_path == relative_path)
        {
            if let Some(existing_tab) = self.pane.tabs.get_mut(tab_index) {
                let buffer_id = existing_tab.buffer_id;
                existing_tab.language = language;
                existing_tab.line_ending = line_ending;
                self.replace_editor_state(buffer_id, editor_state);
            }
            self.pane.active_tab_index = Some(tab_index);
            self.sync_file_tree_with_active_tab();
            self.sync_tool_bar_with_active_tab();
            return true;
        }

        let next_buffer_id = self
            .pane
            .tabs
            .iter()
            .map(|tab| tab.buffer_id.value())
            .max()
            .unwrap_or(0)
            + 1;

        let buffer_id = BufferId::new(next_buffer_id);
        self.replace_editor_state(buffer_id, editor_state);
        self.pane.tabs.push(TabState {
            buffer_id,
            title: workspace_paths::file_name_from_path(relative_path),
            relative_path: relative_path.to_string(),
            language,
            line_ending,
        });
        self.pane.active_tab_index = Some(self.pane.tabs.len() - 1);
        self.sync_file_tree_with_active_tab();
        self.sync_tool_bar_with_active_tab();
        true
    }
}
