//! 编辑历史栈（Undo/Redo）管理。

use std::time::{Duration, Instant};

use zom_editor::EditorState;
use zom_protocol::{BufferId, EditorAction, EditorInvocation};

use super::DesktopAppState;

const DEFAULT_HISTORY_LIMIT: usize = 200;
const MERGE_WINDOW: Duration = Duration::from_millis(800);

#[derive(Debug, Clone, PartialEq, Eq)]
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

    fn undo(&mut self, current: &EditorState) -> Option<EditorState> {
        let previous = self.undo_stack.pop()?;
        self.redo_stack.push(current.clone());
        self.pending_group = None;
        Some(previous)
    }

    fn redo(&mut self, current: &EditorState) -> Option<EditorState> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current.clone());
        self.pending_group = None;
        Some(next)
    }

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

fn merge_group_kind(command: &EditorInvocation) -> Option<PendingGroupKind> {
    match command {
        EditorInvocation::InsertText { text } if !text.is_empty() => {
            Some(PendingGroupKind::InsertText)
        }
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
    pub(super) fn should_record_history(command: &EditorInvocation) -> bool {
        match command {
            EditorInvocation::InsertText { text } => !text.is_empty(),
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

    pub(super) fn undo_editor_history(
        &mut self,
        buffer_id: BufferId,
        current: &EditorState,
    ) -> Option<EditorState> {
        self.editor_histories
            .get_mut(&buffer_id)
            .and_then(|history| history.undo(current))
    }

    pub(super) fn redo_editor_history(
        &mut self,
        buffer_id: BufferId,
        current: &EditorState,
    ) -> Option<EditorState> {
        self.editor_histories
            .get_mut(&buffer_id)
            .and_then(|history| history.redo(current))
    }

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
