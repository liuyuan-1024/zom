
use regex::RegexBuilder;
use zom_protocol::{
    EditorAction, EditorInvocation, FindReplaceAction, FindReplaceRequest, Position, Selection,
};
use zom_text::TextBuffer;
use zom_text_tokens::{LF, TAB, TAB_CHAR};

use super::{
    state::{EditorState, clamp_selection_to_text},
    transaction::{
        TextChange, TransactionMeta, TransactionSource, TransactionSpec, apply_transaction,
    },
};

const OUTDENT_SPACES: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationResult {
    /// 执行命令后的状态快照，保证与最终选区/光标一致。
    pub state: EditorState,
    /// `state.selection().active()` 的便捷镜像，供 UI 直接消费。
    pub cursor: Position,
}

/// 执行高层编辑指令，并处理光标归一化与过期选区降级逻辑。
///
/// 外部传入光标会先按当前缓冲区钳制；若它与当前 active 不一致，
/// 则退化为单光标选区，避免命令误作用到过期 UI 选区。
pub fn apply_editor_invocation(
    state: &EditorState,
    cursor: Position,
    invocation: &EditorInvocation,
) -> InvocationResult {
    let cursor = clamp_position(state.buffer(), cursor);
    let selection = selection_for_invocation(state, cursor);

    match invocation {
        EditorInvocation::InsertText { text } => insert_text(state, selection, text),
        EditorInvocation::FindReplace { request } => {
            apply_find_replace_request(state, selection, cursor, request)
        }
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
        EditorAction::InsertNewline => insert_newline(state, selection),
        EditorAction::InsertIndent => insert_indent(state, selection),
        EditorAction::Outdent => outdent(state, selection),
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
        EditorAction::SelectAll => select_all(state, selection),
        EditorAction::OpenFindReplace
        | EditorAction::FindPrev
        | EditorAction::FindNext
        | EditorAction::ReplaceNext
        | EditorAction::ReplaceAll
        | EditorAction::ToggleFindCaseSensitive
        | EditorAction::ToggleFindWholeWord
        | EditorAction::ToggleFindRegex
        | EditorAction::Copy
        | EditorAction::Cut
        | EditorAction::Paste
        | EditorAction::Undo
        | EditorAction::Redo => InvocationResult {
            state: state.clone(),
            cursor: active,
        },
    }
}

fn selection_for_invocation(state: &EditorState, cursor: Position) -> Selection {
    // 只有“调用方光标 == 当前 active”时才复用选区，
    // 否则收缩为 caret，让命令以最新指针/输入法位置为准。
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

fn insert_newline(state: &EditorState, selection: Selection) -> InvocationResult {
    let indent = auto_indent_for_newline(state, selection.active());
    let mut text = String::from(LF);
    text.push_str(&indent);
    insert_text(state, selection, &text)
}

fn auto_indent_for_newline(state: &EditorState, cursor: Position) -> String {
    let line = cursor.line;
    let line_start = state.position_to_offset(Position::new(line, 0));
    let line_end = state.position_to_offset(Position::new(line, line_len(state.buffer(), line)));
    let Ok(line_text) = state.buffer().slice(line_start..line_end) else {
        return String::new();
    };
    leading_whitespace_prefix(&line_text).to_string()
}

fn insert_indent(state: &EditorState, selection: Selection) -> InvocationResult {
    if selection.is_caret() {
        return insert_text(state, selection, TAB);
    }

    let (start_line, end_line) = touched_line_range(selection);

    let changes = (start_line..=end_line)
        .map(|line| {
            let offset = state.position_to_offset(Position::new(line, 0));
            TextChange::new(offset, offset, TAB)
        })
        .collect::<Vec<_>>();

    apply_with_selection(
        state,
        selection,
        TransactionSpec {
            changes,
            selection: None,
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: None,
        },
    )
}

fn outdent(state: &EditorState, selection: Selection) -> InvocationResult {
    let (start_line, end_line) = if selection.is_caret() {
        let line = selection.active().line;
        (line, line)
    } else {
        touched_line_range(selection)
    };

    let mut current_state = state.clone();
    let mut current_selection = selection;
    let mut changed = false;

    for line in start_line..=end_line {
        let line_start = current_state.position_to_offset(Position::new(line, 0));
        let line_end = current_state
            .position_to_offset(Position::new(line, line_len(current_state.buffer(), line)));
        let Ok(line_text) = current_state.buffer().slice(line_start..line_end) else {
            continue;
        };

        let remove_len = removable_outdent_prefix_len(&line_text);
        if remove_len == 0 {
            continue;
        }

        let result = apply_with_selection(
            &current_state,
            current_selection,
            TransactionSpec {
                changes: vec![TextChange::new(line_start, line_start + remove_len, "")],
                selection: None,
                meta: TransactionMeta::from_source(TransactionSource::Keyboard),
                expected_version: None,
            },
        );
        current_selection = result.state.selection();
        current_state = result.state;
        changed = true;
    }

    if !changed {
        return InvocationResult {
            state: state.clone(),
            cursor: selection.active(),
        };
    }

    InvocationResult {
        cursor: current_selection.active(),
        state: current_state,
    }
}

fn touched_line_range(selection: Selection) -> (u32, u32) {
    if selection.is_caret() {
        let line = selection.active().line;
        return (line, line);
    }

    let start = selection.start();
    let end = selection.end();
    let mut end_line = end.line;
    // 选区若以“下一行 column=0”结束，按行操作时语义上不包含下一行。
    if end.column == 0 && end_line > start.line {
        end_line -= 1;
    }
    (start.line, end_line.max(start.line))
}

fn leading_whitespace_prefix(text: &str) -> &str {
    let end = text
        .char_indices()
        .find(|(_, ch)| !matches!(*ch, ' ' | TAB_CHAR))
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    &text[..end]
}

fn removable_outdent_prefix_len(text: &str) -> usize {
    if text.starts_with(TAB) {
        return TAB.len();
    }

    text.chars()
        .take(OUTDENT_SPACES)
        .take_while(|ch| *ch == ' ')
        .count()
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

fn select_all(state: &EditorState, selection: Selection) -> InvocationResult {
    let full_selection = Selection::new(Position::zero(), state.offset_to_position(state.len()));
    apply_with_selection(
        state,
        selection,
        TransactionSpec {
            changes: Vec::new(),
            selection: Some(full_selection),
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
    // 先把事务绑定到显式工作选区，再加版本保护；
    // 任意失败都降级为 no-op，保证动作处理函数总是可返回且不 panic。
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

fn apply_find_replace_request(
    state: &EditorState,
    selection: Selection,
    cursor: Position,
    request: &FindReplaceRequest,
) -> InvocationResult {
    if request.query.is_empty() {
        return InvocationResult {
            state: state.clone(),
            cursor,
        };
    }
    let text = state.text();
    let Some(regex) = build_find_regex(request) else {
        return InvocationResult {
            state: state.clone(),
            cursor,
        };
    };
    let matches = regex
        .find_iter(&text)
        .map(|m| (m.start(), m.end()))
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return InvocationResult {
            state: state.clone(),
            cursor,
        };
    }

    match request.action {
        FindReplaceAction::FindNext => {
            let offset = state.position_to_offset(cursor);
            let Some((from, to)) = next_match(&matches, offset) else {
                return InvocationResult {
                    state: state.clone(),
                    cursor,
                };
            };
            select_range_by_offsets(state, selection, from, to)
        }
        FindReplaceAction::FindPrev => {
            let offset = state.position_to_offset(cursor);
            let Some((from, to)) = prev_match(&matches, offset) else {
                return InvocationResult {
                    state: state.clone(),
                    cursor,
                };
            };
            select_range_by_offsets(state, selection, from, to)
        }
        FindReplaceAction::ReplaceNext => {
            let replacements =
                collect_replacement_items(&text, &regex, &request.replacement, request.use_regex);
            if replacements.is_empty() {
                return InvocationResult {
                    state: state.clone(),
                    cursor,
                };
            }
            let offset = state.position_to_offset(cursor);
            let Some((from, to, replacement)) = next_replacement(&replacements, offset) else {
                return InvocationResult {
                    state: state.clone(),
                    cursor,
                };
            };
            let caret = Selection::caret(state.offset_to_position(from));
            apply_with_selection(
                state,
                caret,
                TransactionSpec {
                    changes: vec![TextChange::new(from, to, replacement)],
                    selection: None,
                    meta: TransactionMeta::from_source(TransactionSource::Keyboard),
                    expected_version: None,
                },
            )
        }
        FindReplaceAction::ReplaceAll => {
            let replacements =
                collect_replacement_items(&text, &regex, &request.replacement, request.use_regex);
            let Some((first_from, _, _)) = replacements.first().cloned() else {
                return InvocationResult {
                    state: state.clone(),
                    cursor,
                };
            };
            let changes = replacements
                .iter()
                .map(|(from, to, replacement)| TextChange::new(*from, *to, replacement.clone()))
                .collect::<Vec<_>>();
            let caret = Selection::caret(state.offset_to_position(first_from));
            apply_with_selection(
                state,
                caret,
                TransactionSpec {
                    changes,
                    selection: None,
                    meta: TransactionMeta::from_source(TransactionSource::Keyboard),
                    expected_version: None,
                },
            )
        }
    }
}

fn build_find_regex(request: &FindReplaceRequest) -> Option<regex::Regex> {
    // 非正则模式先 escape，再复用同一 regex 管线处理大小写/整词匹配开关。
    let mut pattern = if request.use_regex {
        request.query.clone()
    } else {
        regex::escape(&request.query)
    };
    if request.whole_word {
        pattern = format!(r"\b(?:{})\b", pattern);
    }

    let mut builder = RegexBuilder::new(&pattern);
    builder.case_insensitive(!request.case_sensitive);
    builder.multi_line(true);
    builder.unicode(true);
    builder.build().ok()
}

fn next_match(matches: &[(usize, usize)], cursor_offset: usize) -> Option<(usize, usize)> {
    // 从光标开始向后找，未命中时回绕到第一项。
    matches
        .iter()
        .copied()
        .find(|(from, _)| *from >= cursor_offset)
        .or_else(|| matches.first().copied())
}

fn prev_match(matches: &[(usize, usize)], cursor_offset: usize) -> Option<(usize, usize)> {
    // 从光标开始向前找，未命中时回绕到最后一项。
    matches
        .iter()
        .rev()
        .copied()
        .find(|(_, to)| *to <= cursor_offset)
        .or_else(|| matches.last().copied())
}

fn select_range_by_offsets(
    state: &EditorState,
    selection: Selection,
    from: usize,
    to: usize,
) -> InvocationResult {
    let next_selection =
        Selection::new(state.offset_to_position(from), state.offset_to_position(to));
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

fn collect_replacement_items(
    text: &str,
    regex: &regex::Regex,
    replacement_template: &str,
    use_regex: bool,
) -> Vec<(usize, usize, String)> {
    if use_regex {
        // 正则替换需要先展开捕获组模板，避免后续逐条替换时重复解析模板。
        return regex
            .captures_iter(text)
            .filter_map(|captures| {
                let matched = captures.get(0)?;
                let mut replacement = String::new();
                captures.expand(replacement_template, &mut replacement);
                Some((matched.start(), matched.end(), replacement))
            })
            .collect();
    }

    regex
        .find_iter(text)
        .map(|matched| {
            (
                matched.start(),
                matched.end(),
                replacement_template.to_string(),
            )
        })
        .collect()
}

fn next_replacement(
    replacements: &[(usize, usize, String)],
    cursor_offset: usize,
) -> Option<(usize, usize, String)> {
    replacements
        .iter()
        .find(|(from, _, _)| *from >= cursor_offset)
        .cloned()
        .or_else(|| replacements.first().cloned())
}

#[cfg(test)]
mod tests {
    use zom_protocol::{EditorAction, EditorInvocation, FindReplaceAction, FindReplaceRequest};

    use super::{EditorState, Selection, apply_editor_invocation};

    fn state_with_selection(text: &str, selection: Selection) -> EditorState {
        let state = EditorState::from_text(text);
        EditorState::from_parts(state.buffer().clone(), selection, state.version())
    }

    #[test]
    fn insert_newline_applies_auto_indent_strategy() {
        let state = state_with_selection(
            "    foo",
            Selection::caret(zom_protocol::Position::new(0, 7)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 7),
            &EditorInvocation::from(EditorAction::InsertNewline),
        );

        assert_eq!(result.state.text(), "    foo\n    ");
        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(1, 4))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(1, 4));
    }

    #[test]
    fn insert_indent_indents_selected_lines() {
        let state = state_with_selection(
            "a\nb\nc",
            Selection::new(
                zom_protocol::Position::new(0, 0),
                zom_protocol::Position::new(2, 0),
            ),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(2, 0),
            &EditorInvocation::from(EditorAction::InsertIndent),
        );

        assert_eq!(result.state.text(), "\ta\n\tb\nc");
        assert_eq!(
            result.state.selection(),
            Selection::new(
                zom_protocol::Position::new(0, 1),
                zom_protocol::Position::new(2, 0)
            )
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(2, 0));
    }

    #[test]
    fn outdent_removes_indent_prefix_for_current_line() {
        let state =
            state_with_selection("\tfoo", Selection::caret(zom_protocol::Position::new(0, 1)));

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 1),
            &EditorInvocation::from(EditorAction::Outdent),
        );

        assert_eq!(result.state.text(), "foo");
        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 0))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 0));
    }

    #[test]
    fn outdent_removes_up_to_four_spaces_for_selected_lines() {
        let state = state_with_selection(
            "    a\n  b\nc",
            Selection::new(
                zom_protocol::Position::new(0, 0),
                zom_protocol::Position::new(2, 1),
            ),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(2, 1),
            &EditorInvocation::from(EditorAction::Outdent),
        );

        assert_eq!(result.state.text(), "a\nb\nc");
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
    fn select_all_selects_full_document() {
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

    #[test]
    fn move_right_steps_over_emoji_in_single_command() {
        let state =
            state_with_selection("a🙂b", Selection::caret(zom_protocol::Position::new(0, 1)));

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 1),
            &EditorInvocation::from(EditorAction::MoveRight),
        );

        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 2))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 2));
    }

    #[test]
    fn delete_backward_removes_full_emoji_scalar_value() {
        let state =
            state_with_selection("a🙂b", Selection::caret(zom_protocol::Position::new(0, 2)));

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 2),
            &EditorInvocation::from(EditorAction::DeleteBackward),
        );

        assert_eq!(result.state.text(), "ab");
        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 1))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 1));
    }

    #[test]
    fn delete_forward_removes_full_multibyte_cjk_scalar_value() {
        let state =
            state_with_selection("a中b", Selection::caret(zom_protocol::Position::new(0, 1)));

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 1),
            &EditorInvocation::from(EditorAction::DeleteForward),
        );

        assert_eq!(result.state.text(), "ab");
        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 1))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 1));
    }

    #[test]
    fn delete_backward_after_combining_mark_only_removes_last_scalar() {
        let state = state_with_selection(
            "e\u{301}x",
            Selection::caret(zom_protocol::Position::new(0, 2)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 2),
            &EditorInvocation::from(EditorAction::DeleteBackward),
        );

        assert_eq!(result.state.text(), "ex");
        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 1))
        );
        assert_eq!(result.cursor, zom_protocol::Position::new(0, 1));
    }

    #[test]
    fn find_next_selects_match_case_insensitive() {
        let state = state_with_selection(
            "foo Bar baz",
            Selection::caret(zom_protocol::Position::new(0, 0)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 0),
            &EditorInvocation::find_replace(FindReplaceRequest::new(
                "bar",
                "",
                FindReplaceAction::FindNext,
                false,
                false,
                false,
            )),
        );

        assert_eq!(
            result.state.selection(),
            Selection::new(
                zom_protocol::Position::new(0, 4),
                zom_protocol::Position::new(0, 7)
            )
        );
    }

    #[test]
    fn find_next_with_whole_word_skips_substring_match() {
        let state = state_with_selection(
            "foobar foo",
            Selection::caret(zom_protocol::Position::new(0, 0)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 0),
            &EditorInvocation::find_replace(FindReplaceRequest::new(
                "foo",
                "",
                FindReplaceAction::FindNext,
                true,
                true,
                false,
            )),
        );

        assert_eq!(
            result.state.selection(),
            Selection::new(
                zom_protocol::Position::new(0, 7),
                zom_protocol::Position::new(0, 10)
            )
        );
    }

    #[test]
    fn find_prev_wraps_to_last_match() {
        let state = state_with_selection(
            "one two one",
            Selection::caret(zom_protocol::Position::new(0, 0)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 0),
            &EditorInvocation::find_replace(FindReplaceRequest::new(
                "one",
                "",
                FindReplaceAction::FindPrev,
                true,
                false,
                false,
            )),
        );

        assert_eq!(
            result.state.selection(),
            Selection::new(
                zom_protocol::Position::new(0, 8),
                zom_protocol::Position::new(0, 11)
            )
        );
    }

    #[test]
    fn replace_next_replaces_single_match_and_moves_cursor() {
        let state = state_with_selection(
            "foo foo",
            Selection::caret(zom_protocol::Position::new(0, 0)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 0),
            &EditorInvocation::find_replace(FindReplaceRequest::new(
                "foo",
                "bar",
                FindReplaceAction::ReplaceNext,
                true,
                false,
                false,
            )),
        );

        assert_eq!(result.state.text(), "bar foo");
        assert_eq!(
            result.state.selection(),
            Selection::caret(zom_protocol::Position::new(0, 3))
        );
    }

    #[test]
    fn replace_all_supports_regex_groups() {
        let state = state_with_selection(
            "a1 b2 c3",
            Selection::caret(zom_protocol::Position::new(0, 0)),
        );

        let result = apply_editor_invocation(
            &state,
            zom_protocol::Position::new(0, 0),
            &EditorInvocation::find_replace(FindReplaceRequest::new(
                r"([a-z])(\d)",
                "$1-$2",
                FindReplaceAction::ReplaceAll,
                true,
                false,
                true,
            )),
        );

        assert_eq!(result.state.text(), "a-1 b-2 c-3");
    }
}
