//! 编辑器命令语义分发（历史、选区、剪贴板与文本事务）。

use zom_editor::{EditorState, apply_editor_invocation};
use zom_protocol::{BufferId, EditorAction, EditorInvocation};

use super::{DesktopAppState, DesktopUiAction};

impl DesktopAppState {
    /// 处理编辑器命令，并把结果写回当前活动标签页与工具栏状态。
    pub(super) fn dispatch_editor_invocation(&mut self, command: EditorInvocation) {
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

        let next_state = match &command {
            EditorInvocation::Action(EditorAction::Undo) => self
                .undo_editor_history(active_buffer_id, &current_state)
                .unwrap_or_else(|| current_state.clone()),
            EditorInvocation::Action(EditorAction::Redo) => self
                .redo_editor_history(active_buffer_id, &current_state)
                .unwrap_or_else(|| current_state.clone()),
            EditorInvocation::Action(EditorAction::Copy) => {
                if let Some(selected_text) = selected_text(&current_state) {
                    self.pending_ui_action = Some(DesktopUiAction::WriteClipboard(selected_text));
                }
                current_state.clone()
            }
            EditorInvocation::Action(EditorAction::Cut) => {
                if let Some(selected_text) = selected_text(&current_state) {
                    self.pending_ui_action = Some(DesktopUiAction::WriteClipboard(selected_text));
                    self.apply_editor_change_with_history(
                        active_buffer_id,
                        &current_state,
                        EditorInvocation::from(EditorAction::DeleteBackward),
                    )
                } else {
                    current_state.clone()
                }
            }
            EditorInvocation::Action(EditorAction::Paste) => {
                self.pending_ui_action = Some(DesktopUiAction::PasteFromClipboard);
                current_state.clone()
            }
            EditorInvocation::Action(EditorAction::SelectAll) => {
                self.select_all_in_editor(&current_state)
            }
            _ => self.apply_editor_change_with_history(active_buffer_id, &current_state, command),
        };
        let should_persist_draft = current_state.text() != next_state.text();
        if should_persist_draft {
            self.persist_editor_draft(active_buffer_id, &next_state);
        }
        self.replace_editor_state(active_buffer_id, next_state);
        self.sync_tool_bar_with_active_tab();
    }

    fn apply_editor_change_with_history(
        &mut self,
        buffer_id: BufferId,
        current_state: &EditorState,
        command: EditorInvocation,
    ) -> EditorState {
        let result = apply_editor_invocation(current_state, self.tool_bar.cursor, &command);
        if Self::should_record_history(&command) {
            self.record_editor_history(buffer_id, current_state, &result.state, &command);
        }
        result.state
    }
}

fn selected_text(state: &EditorState) -> Option<String> {
    let selection = state.selection();
    if selection.is_caret() {
        return None;
    }
    let from = state.position_to_offset(selection.start());
    let to = state.position_to_offset(selection.end());
    if from >= to {
        return None;
    }
    state.text().get(from..to).map(ToOwned::to_owned)
}
