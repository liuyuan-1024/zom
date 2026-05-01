use zom_protocol::Selection;
use zom_text::{TextBuffer, TextBufferError};

use super::state::{DocVersion, EditorState, Offset, clamp_selection_to_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChange {
    /// 变更前文档中的起始字节偏移（包含）。
    pub from: Offset,
    /// 变更前文档中的结束字节偏移（不包含）。
    pub to: Offset,
    /// 用于替换 `[from, to)` 的新文本。
    pub insert: String,
}

impl TextChange {
    pub fn new(from: Offset, to: Offset, insert: impl Into<String>) -> Self {
        Self {
            from,
            to,
            insert: insert.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionSource {
    /// 本地键盘输入或快捷键触发。
    Keyboard,
    /// 鼠标/指针行为触发（点击、拖拽等）。
    Mouse,
    /// runtime 同步通道发起的修改。
    Runtime,
    /// 撤销/重做历史回放。
    History,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionMeta {
    /// 变更来源标签，供历史策略和埋点归因。
    pub source: TransactionSource,
    /// 该事务是否应写入撤销历史。
    pub should_add_to_history: bool,
    /// 可选的人类可读标签（用于历史面板展示）。
    pub label: Option<String>,
}

impl TransactionMeta {
    pub fn from_source(source: TransactionSource) -> Self {
        Self {
            source,
            should_add_to_history: true,
            label: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionSpec {
    /// 基于“事务开始时文档版本”的字节区间编辑列表。
    pub changes: Vec<TextChange>,
    /// 显式指定下一选区；为 `None` 时按变更自动映射旧选区。
    pub selection: Option<Selection>,
    /// 事务元信息（来源、是否入历史等）。
    pub meta: TransactionMeta,
    /// 乐观并发校验版本；不匹配时在执行前直接拒绝。
    pub expected_version: Option<DocVersion>,
}

impl TransactionSpec {
    pub fn with_changes(changes: Vec<TextChange>, source: TransactionSource) -> Self {
        Self {
            changes,
            selection: None,
            meta: TransactionMeta::from_source(source),
            expected_version: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionResult {
    /// 事务执行后的完整编辑器状态。
    pub state: EditorState,
    /// 排序并校验后、最终真正应用的变更列表。
    pub applied_changes: Vec<TextChange>,
    /// 文本内容是否发生变化。
    pub is_document_changed: bool,
    /// 结果选区是否不同于事务前选区。
    pub is_selection_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyError {
    /// 请求携带的 `expected_version` 已过期。
    VersionMismatch { current_version: DocVersion },
    /// 按原始坐标排序后，两个变更区间发生重叠。
    OverlappingChanges {
        previous_to: Offset,
        current_from: Offset,
    },
    /// 变更区间不合法（越界或破坏字符边界）。
    InvalidChangeRange {
        index: usize,
        error: TextBufferError,
    },
}

/// 以原子方式执行标准化事务：
/// 1) 可选版本校验 2) 区间合法性校验 3) 文本替换
/// 4) 选区映射与钳制 5) 版本号更新策略。
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
    let previous_anchor = state.position_to_offset(previous_selection.anchor());
    let previous_active = state.position_to_offset(previous_selection.active());
    let mapped_selection = Selection::new(
        next_buffer
            .offset_to_position(map_offset(previous_anchor, &changes))
            .expect("mapped anchor offset should be valid"),
        next_buffer
            .offset_to_position(map_offset(previous_active, &changes))
            .expect("mapped active offset should be valid"),
    );
    let next_selection =
        clamp_selection_to_text(&next_buffer, spec.selection.unwrap_or(mapped_selection));

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

        if let Err(error) = buffer.validate_byte_range(change.from..change.to) {
            return Err(ApplyError::InvalidChangeRange { index, error });
        }
    }
    Ok(())
}

fn apply_sorted_changes(buffer: &mut TextBuffer, changes: &[TextChange]) -> Result<(), ApplyError> {
    // `delta` 表示“原始坐标”到“当前已编辑缓冲区坐标”的累计偏移差。
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
    // 光标若落在被替换区间内，吸附到替换后文本末尾，保证光标语义稳定。
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
    use zom_text::TextBufferError;

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

    #[test]
    fn apply_transaction_sorts_unsorted_changes_and_maps_selection() {
        let state = EditorState::from_text("abcdef");
        let spec = TransactionSpec {
            changes: vec![TextChange::new(4, 6, "YZ"), TextChange::new(0, 2, "WX")],
            selection: None,
            expected_version: Some(DocVersion::zero()),
            meta: super::TransactionMeta::from_source(TransactionSource::Runtime),
        };

        let result = apply_transaction(&state, spec).expect("transaction should apply");
        assert_eq!(result.state.text(), "WXcdYZ");
        assert_eq!(
            result.state.selection(),
            Selection::caret(Position::new(0, 2))
        );
        assert_eq!(result.state.version(), DocVersion::from(1));
    }

    #[test]
    fn apply_transaction_selection_only_change_increments_version() {
        let state = EditorState::from_text("abcdef");
        let spec = TransactionSpec {
            changes: vec![],
            selection: Some(Selection::new(Position::new(0, 1), Position::new(0, 4))),
            expected_version: Some(DocVersion::zero()),
            meta: super::TransactionMeta::from_source(TransactionSource::Keyboard),
        };

        let result =
            apply_transaction(&state, spec).expect("selection-only transaction should apply");
        assert_eq!(result.state.text(), "abcdef");
        assert_eq!(
            result.state.selection(),
            Selection::new(Position::new(0, 1), Position::new(0, 4))
        );
        assert!(result.is_selection_changed);
        assert!(!result.is_document_changed);
        assert_eq!(result.state.version(), DocVersion::from(1));
    }

    #[test]
    fn apply_transaction_rejects_change_with_split_multibyte_boundary() {
        let state = EditorState::from_text("a🙂b");
        let spec = TransactionSpec {
            changes: vec![TextChange::new(2, 5, "X")],
            selection: None,
            expected_version: None,
            meta: super::TransactionMeta::from_source(TransactionSource::Runtime),
        };

        let err = apply_transaction(&state, spec).expect_err("invalid change range expected");
        assert_eq!(
            err,
            ApplyError::InvalidChangeRange {
                index: 0,
                error: TextBufferError::NotCharBoundary { offset: 2 }
            }
        );
    }
}
