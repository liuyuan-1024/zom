//! 事务定义与状态推进逻辑。

use zom_protocol::Selection;
use zom_text::{TextBuffer, TextBufferError, offset_to_position, position_to_offset};

use super::state::{DocVersion, EditorState, Offset, clamp_selection_to_text};

/// 单次文本替换操作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChange {
    /// 起始偏移（包含）。
    pub from: Offset,
    /// 结束偏移（不包含）。
    pub to: Offset,
    /// 替换文本。
    pub insert: String,
}

impl TextChange {
    /// 创建一个文本变更。
    pub fn new(from: Offset, to: Offset, insert: impl Into<String>) -> Self {
        Self {
            from,
            to,
            insert: insert.into(),
        }
    }
}

/// 事务来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionSource {
    Keyboard,
    Mouse,
    Runtime,
    History,
}

/// 事务元信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionMeta {
    pub source: TransactionSource,
    pub should_add_to_history: bool,
    pub label: Option<String>,
}

impl TransactionMeta {
    /// 基于来源构造默认元信息。
    pub fn from_source(source: TransactionSource) -> Self {
        Self {
            source,
            should_add_to_history: true,
            label: None,
        }
    }
}

/// 事务输入规格。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionSpec {
    pub changes: Vec<TextChange>,
    pub selection: Option<Selection>,
    pub meta: TransactionMeta,
    pub expected_version: Option<DocVersion>,
}

impl TransactionSpec {
    /// 仅携带文本变更的事务。
    pub fn with_changes(changes: Vec<TextChange>, source: TransactionSource) -> Self {
        Self {
            changes,
            selection: None,
            meta: TransactionMeta::from_source(source),
            expected_version: None,
        }
    }
}

/// 应用事务后的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionResult {
    pub state: EditorState,
    pub applied_changes: Vec<TextChange>,
    pub is_document_changed: bool,
    pub is_selection_changed: bool,
}

/// 事务失败原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyError {
    VersionMismatch {
        current_version: DocVersion,
    },
    OverlappingChanges {
        previous_to: Offset,
        current_from: Offset,
    },
    InvalidChangeRange {
        index: usize,
        error: TextBufferError,
    },
}

/// 将一笔事务应用到状态并返回新状态。
pub fn apply_transaction(
    state: &EditorState,
    spec: TransactionSpec,
) -> Result<TransactionResult, ApplyError> {
    if let Some(expected) = spec.expected_version
        && expected != state.version()
    {
        return Err(ApplyError::VersionMismatch {
            current_version: state.version(),
        });
    }

    let mut changes = spec.changes;
    changes.sort_by_key(|change| (change.from, change.to));

    validate_changes(state.buffer(), &changes)?;

    let mut next_buffer = state.buffer().clone();
    apply_sorted_changes(&mut next_buffer, &changes)?;

    let previous_selection = state.selection();
    let mapped_selection = Selection::new(
        offset_to_position(
            next_buffer.as_str(),
            map_offset(
                position_to_offset(state.text(), previous_selection.anchor()),
                &changes,
            ),
        ),
        offset_to_position(
            next_buffer.as_str(),
            map_offset(
                position_to_offset(state.text(), previous_selection.active()),
                &changes,
            ),
        ),
    );
    let next_selection = clamp_selection_to_text(
        next_buffer.as_str(),
        spec.selection.unwrap_or(mapped_selection),
    );

    let is_document_changed = !changes.is_empty();
    let is_selection_changed = next_selection != previous_selection;
    let next_version = if is_document_changed || is_selection_changed {
        state.version().next()
    } else {
        state.version()
    };

    Ok(TransactionResult {
        state: EditorState::from_parts(next_buffer, next_selection, next_version),
        applied_changes: changes,
        is_document_changed,
        is_selection_changed,
    })
}

fn validate_changes(buffer: &TextBuffer, changes: &[TextChange]) -> Result<(), ApplyError> {
    let mut previous_to = 0usize;
    for (index, change) in changes.iter().enumerate() {
        if index > 0 && change.from < previous_to {
            return Err(ApplyError::OverlappingChanges {
                previous_to,
                current_from: change.from,
            });
        }
        previous_to = change.to;

        if let Err(error) = buffer.slice(change.from..change.to) {
            return Err(ApplyError::InvalidChangeRange { index, error });
        }
    }
    Ok(())
}

fn apply_sorted_changes(buffer: &mut TextBuffer, changes: &[TextChange]) -> Result<(), ApplyError> {
    let mut delta = 0isize;
    for (index, change) in changes.iter().enumerate() {
        let removed = change.to - change.from;
        let from = shift_offset(change.from, delta);
        let to = from + removed;
        if let Err(error) = buffer.replace_range(from..to, &change.insert) {
            return Err(ApplyError::InvalidChangeRange { index, error });
        }
        delta += change.insert.len() as isize - removed as isize;
    }
    Ok(())
}

fn map_offset(offset: Offset, changes: &[TextChange]) -> Offset {
    let mut mapped = offset;
    for change in changes {
        let removed = change.to - change.from;
        let added = change.insert.len();
        if mapped < change.from {
            continue;
        }
        if mapped >= change.to {
            let delta = added as isize - removed as isize;
            mapped = shift_offset(mapped, delta);
            continue;
        }
        mapped = change.from + added;
    }
    mapped
}

fn shift_offset(offset: usize, delta: isize) -> usize {
    if delta >= 0 {
        offset + delta as usize
    } else {
        offset.saturating_sub(delta.unsigned_abs())
    }
}

#[cfg(test)]
mod tests {
    use zom_protocol::{Position, Selection};

    use crate::features::editing::state::{DocVersion, EditorState};

    use super::{ApplyError, TextChange, TransactionSource, TransactionSpec, apply_transaction};

    #[test]
    fn apply_transaction_inserts_text_and_maps_selection() {
        let state = EditorState::from_text("ab");
        let spec = TransactionSpec {
            changes: vec![TextChange::new(1, 1, "X")],
            selection: None,
            expected_version: Some(DocVersion::zero()),
            meta: super::TransactionMeta::from_source(TransactionSource::Keyboard),
        };

        let result = apply_transaction(&state, spec).expect("transaction should apply");
        assert_eq!(result.state.text(), "aXb");
        assert_eq!(result.state.selection(), Selection::caret(Position::zero()));
        assert_eq!(result.state.version(), DocVersion::from(1));
        assert!(result.is_document_changed);
    }

    #[test]
    fn apply_transaction_with_explicit_selection_clamps_to_text_end() {
        let state = EditorState::from_text("abc");
        let spec = TransactionSpec {
            changes: vec![TextChange::new(3, 3, "d")],
            selection: Some(Selection::caret(Position::new(0, 99))),
            expected_version: None,
            meta: super::TransactionMeta::from_source(TransactionSource::Runtime),
        };

        let result = apply_transaction(&state, spec).expect("transaction should apply");
        assert_eq!(
            result.state.selection(),
            Selection::caret(Position::new(0, 4))
        );
    }

    #[test]
    fn apply_transaction_rejects_version_mismatch() {
        let state = EditorState::from_text("abc");
        let spec = TransactionSpec {
            changes: vec![TextChange::new(0, 0, "X")],
            selection: None,
            expected_version: Some(DocVersion::from(7)),
            meta: super::TransactionMeta::from_source(TransactionSource::Runtime),
        };

        let err = apply_transaction(&state, spec).expect_err("version mismatch expected");
        assert_eq!(
            err,
            ApplyError::VersionMismatch {
                current_version: DocVersion::zero()
            }
        );
    }

    #[test]
    fn apply_transaction_rejects_overlapping_changes() {
        let state = EditorState::from_text("abcdef");
        let spec = TransactionSpec {
            changes: vec![TextChange::new(2, 4, "X"), TextChange::new(3, 5, "Y")],
            selection: None,
            expected_version: None,
            meta: super::TransactionMeta::from_source(TransactionSource::Keyboard),
        };

        let err = apply_transaction(&state, spec).expect_err("overlap should fail");
        assert_eq!(
            err,
            ApplyError::OverlappingChanges {
                previous_to: 4,
                current_from: 3
            }
        );
    }
}
