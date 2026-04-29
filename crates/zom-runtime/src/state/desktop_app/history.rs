//! 编辑历史栈（Undo/Redo）管理。

use std::time::{Duration, Instant};

use zom_editor::EditorState;
use zom_protocol::{BufferId, EditorAction, EditorInvocation, FindReplaceAction};

use super::DesktopAppState;

const DEFAULT_HISTORY_LIMIT: usize = 200;
const MERGE_WINDOW: Duration = Duration::from_millis(800);

#[derive(Debug, Clone, PartialEq, Eq)]
/// 单个缓冲区的撤销/重做历史栈，支持时间窗口内的连续编辑合并。
pub(crate) struct EditorHistory {
    undo_stack: Vec<EditorState>,
    redo_stack: Vec<EditorState>,
    max_entries: usize,
    pending_group: Option<PendingGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PendingGroupKind {
    InsertText,
    DeleteBackward,
    DeleteForward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingGroup {
    kind: PendingGroupKind,
    last_after_state: EditorState,
    last_recorded_at: Instant,
}

impl Default for EditorHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_entries: DEFAULT_HISTORY_LIMIT,
            pending_group: None,
        }
    }
}

impl EditorHistory {
    /// 按命令类型把一次编辑变更写入历史栈。
    /// 在可合并窗口内会合并连续操作，并清空 redo 栈。
    fn record(
        &mut self,
        previous: &EditorState,
        current: &EditorState,
        command: &EditorInvocation,
    ) {
        if previous == current {
            return;
        }

        let now = Instant::now();
        let next_kind = merge_group_kind(command);
        let should_merge = self.should_merge(previous, next_kind.as_ref(), now);

        if !should_merge {
            self.undo_stack.push(previous.clone());
            if self.undo_stack.len() > self.max_entries {
                self.undo_stack.remove(0);
            }
        }
        self.redo_stack.clear();
        self.pending_group = next_kind.map(|kind| PendingGroup {
            kind,
            last_after_state: current.clone(),
            last_recorded_at: now,
        });
    }

    /// 回退到上一条历史状态，并把当前状态压入 redo 栈。
    fn undo(&mut self, current: &EditorState) -> Option<EditorState> {
        let previous = self.undo_stack.pop()?;
        self.redo_stack.push(current.clone());
        self.pending_group = None;
        Some(previous)
    }

    /// 从 redo 栈恢复下一条状态，并把当前状态压入 undo 栈。
    fn redo(&mut self, current: &EditorState) -> Option<EditorState> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current.clone());
        self.pending_group = None;
        Some(next)
    }

    /// 判断当前操作是否可并入上一条历史组。
    ///
    /// 需要同时满足：分组类型一致、时间窗口内、且“上一条 after 状态 == 当前 before 状态”。
    /// 最后一个条件用于防止跨来源修改后仍被错误合并。
    fn should_merge(
        &self,
        current_before_state: &EditorState,
        next_kind: Option<&PendingGroupKind>,
        now: Instant,
    ) -> bool {
        let Some(next_kind) = next_kind else {
            return false;
        };
        let Some(group) = &self.pending_group else {
            return false;
        };
        if &group.kind != next_kind {
            return false;
        }
        if now.duration_since(group.last_recorded_at) > MERGE_WINDOW {
            return false;
        }

        group.last_after_state == *current_before_state
    }
}

/// 将编辑命令归类为可合并历史组；返回 `None` 表示该命令不参与连续合并。
///
/// 查找/替换等批量动作不做连续合并，避免一次操作吞并多步可撤销边界。
fn merge_group_kind(command: &EditorInvocation) -> Option<PendingGroupKind> {
    match command {
        EditorInvocation::InsertText { text } if !text.is_empty() => {
            Some(PendingGroupKind::InsertText)
        }
        EditorInvocation::FindReplace { .. } => None,
        EditorInvocation::Action(EditorAction::DeleteBackward)
        | EditorInvocation::Action(EditorAction::DeleteWordBackward) => {
            Some(PendingGroupKind::DeleteBackward)
        }
        EditorInvocation::Action(EditorAction::DeleteForward)
        | EditorInvocation::Action(EditorAction::DeleteWordForward) => {
            Some(PendingGroupKind::DeleteForward)
        }
        _ => None,
    }
}

impl DesktopAppState {
    /// 判断某命令是否应写入历史栈。
    ///
    /// 纯导航/选区类命令不入历史，保证 Undo/Redo 聚焦“文档内容变更”。
    pub(super) fn should_record_history(command: &EditorInvocation) -> bool {
        match command {
            EditorInvocation::InsertText { text } => !text.is_empty(),
            EditorInvocation::FindReplace { request } => matches!(
                request.action,
                FindReplaceAction::ReplaceNext | FindReplaceAction::ReplaceAll
            ),
            EditorInvocation::Action(action) => matches!(
                action,
                EditorAction::InsertNewline
                    | EditorAction::InsertIndent
                    | EditorAction::Outdent
                    | EditorAction::DeleteBackward
                    | EditorAction::DeleteForward
                    | EditorAction::DeleteWordBackward
                    | EditorAction::DeleteWordForward
            ),
        }
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
