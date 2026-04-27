//! 编辑命令到事务的核心转换与执行。

use zom_protocol::{EditorAction, EditorInvocation, Position, Selection};
use zom_text::TextBuffer;

use super::{
    state::{EditorState, clamp_selection_to_text},
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
    let cursor = clamp_position(state.buffer(), cursor);
    let selection = selection_for_invocation(state, cursor);

    match invocation {
        EditorInvocation::InsertText { text } => insert_text(state, selection, text),
        EditorInvocation::Action(action) => apply_action(state, selection, *action),
    }
}

fn apply_action(
    state: &EditorState,
    selection: Selection,
    action: EditorAction,
) -> InvocationResult {
    let active = selection.active();
    match action {
        EditorAction::InsertNewline => insert_text(state, selection, "\n"),
        EditorAction::MoveLeft => move_left_without_selection(state, selection),
        EditorAction::MoveRight => move_right_without_selection(state, selection),
        EditorAction::MoveUp => apply_selection_move(
            state,
            selection,
            Selection::caret(move_vertical(state, active, -1)),
        ),
        EditorAction::MoveDown => apply_selection_move(
            state,
            selection,
            Selection::caret(move_vertical(state, active, 1)),
        ),
        EditorAction::MoveToStart => apply_selection_move(
            state,
            selection,
            Selection::caret(Position::new(active.line, 0)),
        ),
        EditorAction::MoveToEnd => apply_selection_move(
            state,
            selection,
            Selection::caret(Position::new(
                active.line,
                line_len(state.buffer(), active.line),
            )),
        ),
        EditorAction::MovePageUp => apply_selection_move(
            state,
            selection,
            Selection::caret(move_vertical(state, active, -20)),
        ),
        EditorAction::MovePageDown => apply_selection_move(
            state,
            selection,
            Selection::caret(move_vertical(state, active, 20)),
        ),
        EditorAction::SelectLeft => apply_selection_move(
            state,
            selection,
            Selection::new(selection.anchor(), move_left(state, active)),
        ),
        EditorAction::SelectRight => apply_selection_move(
            state,
            selection,
            Selection::new(selection.anchor(), move_right(state, active)),
        ),
        EditorAction::SelectUp => apply_selection_move(
            state,
            selection,
            Selection::new(selection.anchor(), move_vertical(state, active, -1)),
        ),
        EditorAction::SelectDown => apply_selection_move(
            state,
            selection,
            Selection::new(selection.anchor(), move_vertical(state, active, 1)),
        ),
        EditorAction::SelectToStart => apply_selection_move(
            state,
            selection,
            Selection::new(selection.anchor(), Position::new(active.line, 0)),
        ),
        EditorAction::SelectToEnd => apply_selection_move(
            state,
            selection,
            Selection::new(
                selection.anchor(),
                Position::new(active.line, line_len(state.buffer(), active.line)),
            ),
        ),
        EditorAction::SelectPageUp => apply_selection_move(
            state,
            selection,
            Selection::new(selection.anchor(), move_vertical(state, active, -20)),
        ),
        EditorAction::SelectPageDown => apply_selection_move(
            state,
            selection,
            Selection::new(selection.anchor(), move_vertical(state, active, 20)),
        ),
        EditorAction::DeleteBackward => delete_backward(state, selection),
        EditorAction::DeleteForward => delete_forward(state, selection),
        EditorAction::DeleteWordBackward => delete_word_backward(state, selection),
        EditorAction::DeleteWordForward => delete_word_forward(state, selection),
        // TODO: 历史与选择域后续由 editor state/history 模块承接。
        EditorAction::SelectAll => {
            apply_selection_move(state, selection, full_document_selection(state))
        }
        EditorAction::Undo | EditorAction::Redo => InvocationResult {
            state: state.clone(),
            cursor: active,
        },
    }
}

fn selection_for_invocation(state: &EditorState, cursor: Position) -> Selection {
    let selection = clamp_selection_to_text(state.buffer(), state.selection());
    if selection.active() == cursor {
        selection
    } else {
        Selection::caret(cursor)
    }
}

fn insert_text(state: &EditorState, selection: Selection, text: &str) -> InvocationResult {
    let range = selection.range();
    let from = state.position_to_offset(range.start());
    let to = state.position_to_offset(range.end());
    apply_with_selection(
        state,
        selection,
        TransactionSpec {
            changes: vec![TextChange::new(from, to, text)],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn move_left_without_selection(state: &EditorState, selection: Selection) -> InvocationResult {
    if !selection.is_caret() {
        return apply_selection_move(state, selection, Selection::caret(selection.start()));
    }
    let cursor = selection.active();
    apply_selection_move(state, selection, Selection::caret(move_left(state, cursor)))
}

fn move_right_without_selection(state: &EditorState, selection: Selection) -> InvocationResult {
    if !selection.is_caret() {
        return apply_selection_move(state, selection, Selection::caret(selection.end()));
    }
    let cursor = selection.active();
    apply_selection_move(
        state,
        selection,
        Selection::caret(move_right(state, cursor)),
    )
}

fn delete_selected_range(state: &EditorState, selection: Selection) -> InvocationResult {
    let range = selection.range();
    let from = state.position_to_offset(range.start());
    let to = state.position_to_offset(range.end());
    apply_with_selection(
        state,
        selection,
        TransactionSpec {
            changes: vec![TextChange::new(from, to, "")],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn delete_backward(state: &EditorState, selection: Selection) -> InvocationResult {
    if !selection.is_caret() {
        return delete_selected_range(state, selection);
    }
    let cursor = selection.active();
    let offset = state.position_to_offset(cursor);
    let Some(start) = prev_char_start(state.buffer(), offset) else {
        return InvocationResult {
            state: state.clone(),
            cursor,
        };
    };
    apply_with_selection(
        state,
        selection,
        TransactionSpec {
            changes: vec![TextChange::new(start, offset, "")],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn delete_forward(state: &EditorState, selection: Selection) -> InvocationResult {
    if !selection.is_caret() {
        return delete_selected_range(state, selection);
    }
    let cursor = selection.active();
    let offset = state.position_to_offset(cursor);
    let Some(end) = next_char_end(state.buffer(), offset) else {
        return InvocationResult {
            state: state.clone(),
            cursor,
        };
    };
    apply_with_selection(
        state,
        selection,
        TransactionSpec {
            changes: vec![TextChange::new(offset, end, "")],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn delete_word_backward(state: &EditorState, selection: Selection) -> InvocationResult {
    if !selection.is_caret() {
        return delete_selected_range(state, selection);
    }
    let cursor = selection.active();
    let offset = state.position_to_offset(cursor);
    let mut start = offset;

    while let Some(prev) = prev_char_start(state.buffer(), start) {
        let Some(ch) = char_at(state.buffer(), prev) else {
            break;
        };
        if !is_word_boundary_char(ch) {
            break;
        }
        start = prev;
    }

    while let Some(prev) = prev_char_start(state.buffer(), start) {
        let Some(ch) = char_at(state.buffer(), prev) else {
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

    apply_with_selection(
        state,
        selection,
        TransactionSpec {
            changes: vec![TextChange::new(start, offset, "")],
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn delete_word_forward(state: &EditorState, selection: Selection) -> InvocationResult {
    if !selection.is_caret() {
        return delete_selected_range(state, selection);
    }
    let cursor = selection.active();
    let offset = state.position_to_offset(cursor);
    let mut end = offset;

    while let Some(next_end) = next_char_end(state.buffer(), end) {
        let Some(ch) = char_at(state.buffer(), end) else {
            break;
        };
        if !is_word_boundary_char(ch) {
            break;
        }
        end = next_end;
    }

    while let Some(next_end) = next_char_end(state.buffer(), end) {
        let Some(ch) = char_at(state.buffer(), end) else {
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

    apply_with_selection(
        state,
        selection,
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
    prev_char_start(state.buffer(), offset)
        .map(|start| state.offset_to_position(start))
        .unwrap_or(cursor)
}

fn move_right(state: &EditorState, cursor: Position) -> Position {
    let offset = state.position_to_offset(cursor);
    next_char_end(state.buffer(), offset)
        .map(|end| state.offset_to_position(end))
        .unwrap_or(cursor)
}

fn move_vertical(state: &EditorState, cursor: Position, delta: i32) -> Position {
    let line_count = line_count(state.buffer()) as i32;
    let next_line = (cursor.line as i32 + delta).clamp(0, line_count.saturating_sub(1));
    let next_line = u32::try_from(next_line).unwrap_or(0);
    Position::new(
        next_line,
        cursor.column.min(line_len(state.buffer(), next_line)),
    )
}

fn full_document_selection(state: &EditorState) -> Selection {
    Selection::new(Position::zero(), state.offset_to_position(state.len()))
}

fn apply_selection_move(
    state: &EditorState,
    selection: Selection,
    next_selection: Selection,
) -> InvocationResult {
    apply_with_selection(
        state,
        selection,
        TransactionSpec {
            changes: Vec::new(),
            selection: Some(next_selection),
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn apply_with_selection(
    state: &EditorState,
    selection: Selection,
    mut spec: TransactionSpec,
) -> InvocationResult {
    let working_state = state_with_selection(state, selection);
    spec.expected_version = Some(working_state.version());

    match apply_transaction(&working_state, spec) {
        Ok(result) => InvocationResult {
            cursor: result.state.selection().active(),
            state: result.state,
        },
        Err(_) => InvocationResult {
            state: state.clone(),
            cursor: selection.active(),
        },
    }
}

fn state_with_selection(state: &EditorState, selection: Selection) -> EditorState {
    if state.selection() == selection {
        return state.clone();
    }
    EditorState::from_parts(state.buffer().clone(), selection, state.version())
}

fn prev_char_start(buffer: &TextBuffer, offset: usize) -> Option<usize> {
    buffer.prev_char_start(offset)
}

fn next_char_end(buffer: &TextBuffer, offset: usize) -> Option<usize> {
    buffer.next_char_end(offset)
}

fn char_at(buffer: &TextBuffer, offset: usize) -> Option<char> {
    buffer.char_at(offset)
}

fn line_count(buffer: &TextBuffer) -> usize {
    usize::try_from(buffer.line_count())
        .unwrap_or(usize::MAX)
        .max(1)
}

fn line_len(buffer: &TextBuffer, line: u32) -> u32 {
    buffer.line_len(line)
}

fn clamp_position(buffer: &TextBuffer, cursor: Position) -> Position {
    buffer.clamp_position(cursor)
}

fn is_word_boundary_char(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            '_' | '-' | '/' | '\\' | '.' | ',' | ';' | ':' | '(' | ')'
        )
}

#[cfg(test)]
mod tests {
    use zom_protocol::{EditorAction, EditorInvocation};

    use super::{EditorState, Selection, apply_editor_invocation};

    fn state_with_selection(text: &str, selection: Selection) -> EditorState {
        let state = EditorState::from_text(text);
        EditorState::from_parts(state.buffer().clone(), selection, state.version())
    }

    #[test]
    fn shift_right_expands_selection_from_caret() {
        let state = state_with_selection("ab", Selection::caret(zom_protocol::Position::new(0, 0)));

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 0),
            &EditorInvocation::from(EditorAction::SelectRight),
        );

        assert_eq!(
            result.state.selection(),
            Selection::new(
                zom_protocol::Position::new(0, 0),
                zom_protocol::Position::new(0, 1)
            )
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 1));
    }

    #[test]
    fn insert_text_replaces_current_selection() {
        let state = state_with_selection(
            "abcd",
            Selection::new(
                zom_protocol::Position::new(0, 1),
                zom_protocol::Position::new(0, 3),
            ),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 3),
            &EditorInvocation::insert_text("X"),
        );

        assert_eq!(result.state.text(), "aXd");
        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 2))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 2));
    }

    #[test]
    fn delete_backward_deletes_selected_range_first() {
        let state = state_with_selection(
            "abcd",
            Selection::new(
                zom_protocol::Position::new(0, 1),
                zom_protocol::Position::new(0, 3),
            ),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 3),
            &EditorInvocation::from(EditorAction::DeleteBackward),
        );

        assert_eq!(result.state.text(), "ad");
        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 1))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 1));
    }

    #[test]
    fn move_left_collapses_selection_to_start_without_shift() {
        let state = state_with_selection(
            "abcd",
            Selection::new(
                zom_protocol::Position::new(0, 1),
                zom_protocol::Position::new(0, 3),
            ),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 3),
            &EditorInvocation::from(EditorAction::MoveLeft),
        );

        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 1))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 1));
    }

    #[test]
    fn move_right_collapses_selection_to_end_without_shift() {
        let state = state_with_selection(
            "abcd",
            Selection::new(
                zom_protocol::Position::new(0, 3),
                zom_protocol::Position::new(0, 1),
            ),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 1),
            &EditorInvocation::from(EditorAction::MoveRight),
        );

        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 3))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 3));
    }

    #[test]
    fn shift_up_and_down_extend_selection_while_preserving_anchor() {
        let state = state_with_selection(
            "ab\ncd\nef",
            Selection::caret(zom_protocol::Position::new(1, 1)),
        );

        let up = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(1, 1),
            &EditorInvocation::from(EditorAction::SelectUp),
        );
        assert_eq!(
            up.state.selection(),
            Selection::new(
                zom_protocol::Position::new(1, 1),
                zom_protocol::Position::new(0, 1)
            )
        );
        assert_eq!(up.cursor, zom_protocol::Position::new(0, 1));

        let down = apply_editor_invocation(
            &up.state,
            up.cursor,
            &EditorInvocation::from(EditorAction::SelectDown),
        );
        assert_eq!(
            down.state.selection(),
            Selection::caret(zom_protocol::Position::new(1, 1))
        );
        assert_eq!(down.cursor, zom_protocol::Position::new(1, 1));
    }

    #[test]
    fn shift_home_extends_selection_to_line_start() {
        let state = state_with_selection(
            "ab\ncdef\ngh",
            Selection::caret(zom_protocol::Position::new(1, 3)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(1, 3),
            &EditorInvocation::from(EditorAction::SelectToStart),
        );

        assert_eq!(
            result.state.selection(),
            Selection::new(
                zom_protocol::Position::new(1, 3),
                zom_protocol::Position::new(1, 0)
            )
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(1, 0));
    }

    #[test]
    fn shift_end_extends_selection_to_line_end() {
        let state = state_with_selection(
            "ab\ncdef\ngh",
            Selection::caret(zom_protocol::Position::new(1, 1)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(1, 1),
            &EditorInvocation::from(EditorAction::SelectToEnd),
        );

        assert_eq!(
            result.state.selection(),
            Selection::new(
                zom_protocol::Position::new(1, 1),
                zom_protocol::Position::new(1, 4)
            )
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(1, 4));
    }

    #[test]
    fn shift_page_up_and_down_extend_selection_while_preserving_anchor() {
        let text = (0..30)
            .map(|index| format!("line-{index:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        let state =
            state_with_selection(&text, Selection::caret(zom_protocol::Position::new(25, 4)));

        let page_up = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(25, 4),
            &EditorInvocation::from(EditorAction::SelectPageUp),
        );
        assert_eq!(
            page_up.state.selection(),
            Selection::new(
                zom_protocol::Position::new(25, 4),
                zom_protocol::Position::new(5, 4)
            )
        );
        assert_eq!(page_up.cursor, zom_protocol::Position::new(5, 4));

        let page_down = apply_editor_invocation(
            &page_up.state,
            page_up.cursor,
            &EditorInvocation::from(EditorAction::SelectPageDown),
        );
        assert_eq!(
            page_down.state.selection(),
            Selection::caret(zom_protocol::Position::new(25, 4))
        );
        assert_eq!(page_down.cursor, zom_protocol::Position::new(25, 4));
    }

    #[test]
    fn select_all_selects_entire_document() {
        let state = state_with_selection(
            "ab\ncd",
            Selection::caret(zom_protocol::Position::new(1, 1)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(1, 1),
            &EditorInvocation::from(EditorAction::SelectAll),
        );

        assert_eq!(
            result.state.selection(),
            Selection::new(
                zom_protocol::Position::new(0, 0),
                zom_protocol::Position::new(1, 2)
            )
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(1, 2));
    }
}
