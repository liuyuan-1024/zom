//！ Tab 生命周期与同步逻辑

use zom_protocol::{Position, command::TabAction};

use super::DesktopAppState;

impl DesktopAppState {
    /// 处理标签页命令。
    pub(super) fn handle_tab_command(&mut self, command: TabAction) {
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
        self.sync_file_tree_with_active_tab();
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
        self.sync_file_tree_with_active_tab();
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
        self.sync_file_tree_with_active_tab();
        self.sync_tool_bar_with_active_tab();
    }

    pub(super) fn sync_tool_bar_with_active_tab(&mut self) {
        if let Some(active_tab) = self.pane.active_tab() {
            self.tool_bar.cursor = active_tab.editor_state.selection().active();
            self.tool_bar.language = active_tab.language().to_string();
        } else {
            self.tool_bar.cursor = Position::zero();
            self.tool_bar.language.clear();
        }
    }

    pub(super) fn sync_file_tree_with_active_tab(&mut self) {
        let Some(relative_path) = self
            .pane
            .active_tab()
            .map(|active_tab| active_tab.relative_path.clone())
        else {
            return;
        };
        self.file_tree.activate_file(&relative_path);
    }
}
