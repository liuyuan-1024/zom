//! 编辑历史栈（Undo/Redo）管理。

use zom_editor::{EditorState, should_record_history};
use zom_protocol::{BufferId, EditorInvocation};

use super::DesktopAppState;

pub(crate) use zom_editor::EditorHistory;

impl DesktopAppState {
    /// 判断某命令是否应写入历史栈。
    ///
    /// 纯导航/选区类命令不入历史，保证 Undo/Redo 聚焦“文档内容变更”。
    pub(super) fn should_record_history(command: &EditorInvocation) -> bool {
        should_record_history(command)
    }

    /// 对指定缓冲区执行撤销；若不存在历史记录则返回 `None`。
    pub(super) fn undo_editor_history(
        &mut self,
        buffer_id: BufferId,
        current: &EditorState,
    ) -> Option<EditorState> {
        self.editor_histories
            .get_mut(&buffer_id)
            .and_then(|history| history.undo(current))
    }

    /// 对指定缓冲区执行重做；若不存在可重做记录则返回 `None`。
    pub(super) fn redo_editor_history(
        &mut self,
        buffer_id: BufferId,
        current: &EditorState,
    ) -> Option<EditorState> {
        self.editor_histories
            .get_mut(&buffer_id)
            .and_then(|history| history.redo(current))
    }

    /// 把指定缓冲区的一次编辑前后状态写入历史记录器。
    pub(super) fn record_editor_history(
        &mut self,
        buffer_id: BufferId,
        previous: &EditorState,
        current: &EditorState,
        command: &EditorInvocation,
    ) {
        self.editor_histories
            .entry(buffer_id)
            .or_default()
            .record(previous, current, command);
    }
}
