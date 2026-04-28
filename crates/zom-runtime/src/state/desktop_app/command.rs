//！ 命令解析与分发

use zom_editor::apply_editor_invocation;
use zom_input::resolve_default;
use zom_protocol::{
    CommandInvocation, EditorInvocation, InputContext, InputResolution, Keystroke,
    command::{FileTreeAction, NotificationAction, WorkspaceAction},
};

use super::{DesktopAppState, DesktopUiAction};

impl DesktopAppState {
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
            WorkspaceAction::QuitApp => {
                self.pending_ui_action = Some(DesktopUiAction::QuitApp);
            }
            WorkspaceAction::MinimizeWindow => {
                self.pending_ui_action = Some(DesktopUiAction::MinimizeWindow);
            }
            WorkspaceAction::OpenProjectPicker => {
                self.pending_ui_action = Some(DesktopUiAction::OpenProjectPicker);
            }
            WorkspaceAction::FocusPanel(target) => self.focus_panel(target),
            WorkspaceAction::FocusOverlay(target) => self.focus_overlay(target),
            WorkspaceAction::CloseFocused => self.close_focused(),
            WorkspaceAction::FileTree(command) => self.handle_file_tree_command(command),
            WorkspaceAction::Tab(command) => self.handle_tab_command(command),
            WorkspaceAction::Notification(command) => self.handle_notification_command(command),
        }
    }

    /// 处理编辑器命令，并把结果写回当前活动标签页与工具栏状态。
    fn handle_editor_command(&mut self, command: EditorInvocation) {
        let Some(active_buffer_id) = self.active_buffer_id() else {
            if self.pane.active_tab_index.is_some() {
                self.pane.active_tab_index = None;
                self.sync_tool_bar_with_active_tab();
            }
            return;
        };
        let Some(current_state) = self.take_editor_state(active_buffer_id) else {
            self.pane.active_tab_index = None;
            self.sync_tool_bar_with_active_tab();
            return;
        };

        let result = apply_editor_invocation(&current_state, self.tool_bar.cursor, &command);
        self.replace_editor_state(active_buffer_id, result.state);
        self.sync_tool_bar_with_active_tab();
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

    /// 处理通知中心命令。
    fn handle_notification_command(&mut self, command: NotificationAction) {
        match command {
            NotificationAction::MarkSelectedRead => self.mark_selected_notification_read(),
            NotificationAction::MarkAllRead => self.mark_all_notifications_read(),
            NotificationAction::ClearAll => self.clear_notifications(),
            NotificationAction::ClearRead => self.clear_read_notifications(),
            NotificationAction::FocusUnreadError => self.focus_unread_error_notification(),
            NotificationAction::SelectPrev => self.select_prev_notification(),
            NotificationAction::SelectNext => self.select_next_notification(),
        }
    }
}
