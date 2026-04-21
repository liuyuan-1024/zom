//! 编辑命令到事务的核心转换与执行。

use zom_protocol::{EditorAction, EditorInvocation, Position, Selection};
use zom_text::{offset_to_position, split_lines};

use super::{
    state::EditorState,
    transaction::{
        TextChange, TransactionMeta, TransactionSource, TransactionSpec, apply_transaction,
    },
};

/// 编辑命令执行结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationResult {
    pub state: EditorState,
    pub cursor: Position,
}

/// 对给定编辑状态执行一次编辑器调用。
pub fn apply_editor_invocation(
    state: &EditorState,
    cursor: Position,
    invocation: &EditorInvocation,
) -> InvocationResult {
    let cursor = clamp_position(state.text(), cursor);

    match invocation {
        EditorInvocation::InsertText { text } => insert_text(state, cursor, text),
        EditorInvocation::Action(action) => apply_action(state, cursor, *action),
    }
}

fn apply_action(state: &EditorState, cursor: Position, action: EditorAction) -> InvocationResult {
    match action {
        EditorAction::InsertNewline => insert_text(state, cursor, "\n"),
        EditorAction::MoveLeft => apply_cursor_move(state, cursor, move_left(state, cursor)),
        EditorAction::MoveRight => apply_cursor_move(state, cursor, move_right(state, cursor)),
        EditorAction::MoveUp => apply_cursor_move(state, cursor, move_vertical(state, cursor, -1)),
        EditorAction::MoveDown => apply_cursor_move(state, cursor, move_vertical(state, cursor, 1)),
        EditorAction::MoveToStart => {
            apply_cursor_move(state, cursor, Position::new(cursor.line, 0))
        }
        EditorAction::MoveToEnd => apply_cursor_move(
            state,
            cursor,
            Position::new(cursor.line, line_len(state.text(), cursor.line)),
        ),
        EditorAction::MovePageUp => {
            apply_cursor_move(state, cursor, move_vertical(state, cursor, -20))
        }
        EditorAction::MovePageDown => {
            apply_cursor_move(state, cursor, move_vertical(state, cursor, 20))
        }
        EditorAction::DeleteBackward => delete_backward(state, cursor),
        EditorAction::DeleteForward => delete_forward(state, cursor),
        EditorAction::DeleteWordBackward => delete_word_backward(state, cursor),
        EditorAction::DeleteWordForward => delete_word_forward(state, cursor),
        // TODO: 历史与选择域后续由 editor state/history 模块承接。
        EditorAction::Undo | EditorAction::Redo | EditorAction::SelectAll => InvocationResult {
            state: state.clone(),
            cursor,
        },
    }
}

fn insert_text(state: &EditorState, cursor: Position, text: &str) -> InvocationResult {
    let offset = state.position_to_offset(cursor);
    apply_with_cursor(
        state,
        cursor,
        TransactionSpec {
            changes: vec![TextChange::new(offset, offset, text)],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn delete_backward(state: &EditorState, cursor: Position) -> InvocationResult {
    let offset = state.position_to_offset(cursor);
    let Some(start) = prev_char_start(state.text(), offset) else {
        return InvocationResult {
            state: state.clone(),
            cursor,
        };
    };
    apply_with_cursor(
        state,
        cursor,
        TransactionSpec {
            changes: vec![TextChange::new(start, offset, "")],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn delete_forward(state: &EditorState, cursor: Position) -> InvocationResult {
    let offset = state.position_to_offset(cursor);
    let Some(end) = next_char_end(state.text(), offset) else {
        return InvocationResult {
            state: state.clone(),
            cursor,
        };
    };
    apply_with_cursor(
        state,
        cursor,
        TransactionSpec {
            changes: vec![TextChange::new(offset, end, "")],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn delete_word_backward(state: &EditorState, cursor: Position) -> InvocationResult {
    let offset = state.position_to_offset(cursor);
    let mut start = offset;

    while let Some(prev) = prev_char_start(state.text(), start) {
        let Some(ch) = char_at(state.text(), prev) else {
            break;
        };
        if !is_word_boundary_char(ch) {
            break;
        }
        start = prev;
    }

    while let Some(prev) = prev_char_start(state.text(), start) {
        let Some(ch) = char_at(state.text(), prev) else {
            break;
        };
        if is_word_boundary_char(ch) {
            break;
        }
        start = prev;
    }

    if start == offset {
        return InvocationResult {
            state: state.clone(),
            cursor,
        };
    }

    apply_with_cursor(
        state,
        cursor,
        TransactionSpec {
            changes: vec![TextChange::new(start, offset, "")],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn delete_word_forward(state: &EditorState, cursor: Position) -> InvocationResult {
    let offset = state.position_to_offset(cursor);
    let mut end = offset;

    while let Some(next_end) = next_char_end(state.text(), end) {
        let Some(ch) = char_at(state.text(), end) else {
            break;
        };
        if !is_word_boundary_char(ch) {
            break;
        }
        end = next_end;
    }

    while let Some(next_end) = next_char_end(state.text(), end) {
        let Some(ch) = char_at(state.text(), end) else {
            break;
        };
        if is_word_boundary_char(ch) {
            break;
        }
        end = next_end;
    }

    if end == offset {
        return InvocationResult {
            state: state.clone(),
            cursor,
        };
    }

    apply_with_cursor(
        state,
        cursor,
        TransactionSpec {
            changes: vec![TextChange::new(offset, end, "")],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn move_left(state: &EditorState, cursor: Position) -> Position {
    let offset = state.position_to_offset(cursor);
    prev_char_start(state.text(), offset)
        .map(|start| offset_to_position(state.text(), start))
        .unwrap_or(cursor)
}

fn move_right(state: &EditorState, cursor: Position) -> Position {
    let offset = state.position_to_offset(cursor);
    next_char_end(state.text(), offset)
        .map(|end| offset_to_position(state.text(), end))
        .unwrap_or(cursor)
}

fn move_vertical(state: &EditorState, cursor: Position, delta: i32) -> Position {
    let line_count = line_count(state.text()) as i32;
    let next_line = (cursor.line as i32 + delta).clamp(0, line_count.saturating_sub(1));
    let next_line = u32::try_from(next_line).unwrap_or(0);
    Position::new(
        next_line,
        cursor.column.min(line_len(state.text(), next_line)),
    )
}

fn apply_cursor_move(
    state: &EditorState,
    cursor: Position,
    next_cursor: Position,
) -> InvocationResult {
    apply_with_cursor(
        state,
        cursor,
        TransactionSpec {
            changes: Vec::new(),
            selection: Some(Selection::caret(next_cursor)),
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn apply_with_cursor(
    state: &EditorState,
    cursor: Position,
    mut spec: TransactionSpec,
) -> InvocationResult {
    let working_state = state_with_cursor(state, cursor);
    spec.expected_version = Some(working_state.version());

    match apply_transaction(&working_state, spec) {
        Ok(result) => InvocationResult {
            cursor: result.state.selection().active(),
            state: result.state,
        },
        Err(_) => InvocationResult {
            state: state.clone(),
            cursor,
        },
    }
}

fn state_with_cursor(state: &EditorState, cursor: Position) -> EditorState {
    let selection = Selection::caret(cursor);
    if state.selection() == selection {
        return state.clone();
    }
    EditorState::from_parts(state.buffer().clone(), selection, state.version())
}

fn prev_char_start(text: &str, offset: usize) -> Option<usize> {
    if offset == 0 {
        return None;
    }
    text[..offset].char_indices().last().map(|(start, _)| start)
}

fn next_char_end(text: &str, offset: usize) -> Option<usize> {
    if offset >= text.len() {
        return None;
    }
    text[offset..]
        .chars()
        .next()
        .map(|ch| offset + ch.len_utf8())
}

fn char_at(text: &str, offset: usize) -> Option<char> {
    text[offset..].chars().next()
}

fn line_count(text: &str) -> usize {
    split_lines(text).len().max(1)
}

fn line_len(text: &str, line: u32) -> u32 {
    split_lines(text)
        .get(line as usize)
        .map(|line_text| line_text.chars().count() as u32)
        .unwrap_or(0)
}

fn clamp_position(text: &str, cursor: Position) -> Position {
    let line_count = u32::try_from(line_count(text)).unwrap_or(u32::MAX);
    let line = cursor.line.min(line_count.saturating_sub(1));
    Position::new(line, cursor.column.min(line_len(text, line)))
}

fn is_word_boundary_char(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            '_' | '-' | '/' | '\\' | '.' | ',' | ';' | ':' | '(' | ')'
        )
}
