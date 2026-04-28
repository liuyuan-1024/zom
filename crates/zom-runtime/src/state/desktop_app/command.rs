//! 命令解析与分发

use zom_input::resolve_default;
use zom_protocol::{
    CommandInvocation, EditorInvocation, InputContext, InputResolution, Keystroke,
    command::{FileTreeAction, NotificationAction, WorkspaceAction},
};

use super::{DesktopAppState, DesktopUiAction};

impl DesktopAppState {
    /// 处理一个键盘输入，解析成命令后统一交给应用层分发。
    pub fn dispatch_keystroke(&mut self, keystroke: &Keystroke) -> bool {
        let context = InputContext::new(self.focused_target);
        let resolution = resolve_default(keystroke, &context);
        match resolution {
            InputResolution::Command(command) => {
                self.dispatch_command(command);
                true
            }
            InputResolution::InsertText(text) => {
                self.dispatch_command(CommandInvocation::from(EditorInvocation::insert_text(text)));
                true
            }
            InputResolution::Noop => false,
        }
    }

    /// 统一处理顶层命令，并分发到对应领域。
    pub fn dispatch_command(&mut self, command: CommandInvocation) {
        match command {
            CommandInvocation::Workspace(command) => self.dispatch_workspace_action(command),
            CommandInvocation::Editor(command) => self.dispatch_editor_invocation(command),
        }
    }

    /// 处理工作台命令，并分发到细分子域。
    fn dispatch_workspace_action(&mut self, command: WorkspaceAction) {
        match command {
            WorkspaceAction::QuitApp => {
                self.pending_ui_action = Some(DesktopUiAction::QuitApp);
            }
            WorkspaceAction::MinimizeWindow => {
                self.pending_ui_action = Some(DesktopUiAction::MinimizeWindow);
            }
            WorkspaceAction::SaveActiveBuffer => self.save_active_editor_buffer(),
            WorkspaceAction::OpenProjectPicker => {
                self.pending_ui_action = Some(DesktopUiAction::OpenProjectPicker);
            }
            WorkspaceAction::OpenFindReplace => {
                self.pending_ui_action = Some(DesktopUiAction::OpenFindReplace);
            }
            WorkspaceAction::FocusPanel(target) => self.focus_panel(target),
            WorkspaceAction::FocusOverlay(target) => self.focus_overlay(target),
            WorkspaceAction::CloseFocused => self.close_focused(),
            WorkspaceAction::FileTree(command) => self.dispatch_file_tree_action(command),
            WorkspaceAction::Tab(command) => self.dispatch_tab_action(command),
            WorkspaceAction::Notification(command) => self.dispatch_notification_action(command),
        }
    }

    /// 处理文件树命令，并同步工作区状态。
    fn dispatch_file_tree_action(&mut self, command: FileTreeAction) {
        match command {
            FileTreeAction::SelectPrev => self.file_tree.select_prev_visible(),
            FileTreeAction::SelectNext => self.file_tree.select_next_visible(),
            FileTreeAction::ExpandOrDescend => self.file_tree.expand_or_descend_selected(),
            FileTreeAction::CollapseOrAscend => self.file_tree.collapse_or_ascend_selected(),
            FileTreeAction::ActivateSelection => {
                if let Some((relative_path, kind)) = self.file_tree.selected_node() {
                    self.activate_file_tree_node(&relative_path, kind);
                } else {
                    self.file_tree.select_next_visible();
                }
            }
        }
    }

    /// 处理通知中心命令。
    fn dispatch_notification_action(&mut self, command: NotificationAction) {
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
