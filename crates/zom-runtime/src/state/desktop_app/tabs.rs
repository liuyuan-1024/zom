//! Tab 生命周期与同步逻辑

use zom_protocol::{Position, command::TabAction};

use super::DesktopAppState;

impl DesktopAppState {
    /// 处理标签页命令。
    pub(super) fn dispatch_tab_action(&mut self, command: TabAction) {
        match command {
            TabAction::CloseActiveTab => self.close_active_tab(),
            TabAction::ActivatePrevTab => self.activate_prev_tab(),
            TabAction::ActivateNextTab => self.activate_next_tab(),
        }
    }

    /// 关闭当前活动标签并维护活动索引与焦点状态。
    ///
    /// 同时清理关联编辑状态与历史，避免孤儿 `buffer_id` 残留。
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

        let closed_tab = self.pane.tabs.remove(active_index);
        self.remove_editor_state(closed_tab.buffer_id);
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

    /// 激活上一个标签页（循环切换）并同步关联视图。
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

    /// 激活下一个标签页（循环切换）并同步关联视图。
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

    /// 根据活动标签刷新工具栏光标和语言。
    ///
    /// 当活动标签不存在时重置为默认值，防止展示陈旧元信息。
    pub(super) fn sync_tool_bar_with_active_tab(&mut self) {
        if let Some(active_tab) = self.pane.active_tab()
            && let Some(editor_state) = self.editor_state(active_tab.buffer_id)
        {
            self.tool_bar.cursor = editor_state.selection().active();
            self.tool_bar.language = active_tab.language().to_string();
            return;
        }
        self.tool_bar.cursor = Position::zero();
        self.tool_bar.language.clear();
    }

    /// 让文件树高亮与当前活动标签保持一致。
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
