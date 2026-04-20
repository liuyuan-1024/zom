//! 编辑器文本缓冲区领域模型。

use zom_protocol::{EditorAction, EditorInvocation, Position};
use zom_text::{detect_line_ending, split_lines};

/// 编辑器缓冲区。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorBuffer {
    text: String,
}

impl EditorBuffer {
    /// 用给定文本创建缓冲区。
    pub fn from_text(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// 返回缓冲区完整文本。
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// 返回按编辑器视角拆分后的文本行。
    pub fn lines(&self) -> Vec<String> {
        split_lines(self.as_str())
    }

    /// 返回缓冲区换行风格。
    pub fn line_ending(&self) -> String {
        detect_line_ending(self.as_str())
    }

    /// 返回逻辑行数（至少为 1）。
    pub fn line_count(&self) -> usize {
        self.lines().len().max(1)
    }

    /// 执行一次编辑器调用，并返回更新后的光标位置。
    pub fn apply_invocation(
        &mut self,
        cursor: Position,
        invocation: &EditorInvocation,
    ) -> Position {
        let cursor = self.clamp_position(cursor);

        match invocation {
            EditorInvocation::InsertText { text } => self.insert_text(cursor, text),
            EditorInvocation::Action(action) => self.apply_action(cursor, *action),
        }
    }

    fn apply_action(&mut self, cursor: Position, action: EditorAction) -> Position {
        match action {
            EditorAction::InsertNewline => self.insert_text(cursor, "\n"),
            EditorAction::MoveLeft => self.move_left(cursor),
            EditorAction::MoveRight => self.move_right(cursor),
            EditorAction::MoveUp => self.move_vertical(cursor, -1),
            EditorAction::MoveDown => self.move_vertical(cursor, 1),
            EditorAction::MoveToStart => Position::new(cursor.line, 0),
            EditorAction::MoveToEnd => Position::new(cursor.line, self.line_len(cursor.line)),
            EditorAction::MovePageUp => self.move_vertical(cursor, -20),
            EditorAction::MovePageDown => self.move_vertical(cursor, 20),
            EditorAction::DeleteBackward => self.delete_backward(cursor),
            EditorAction::DeleteForward => self.delete_forward(cursor),
            EditorAction::DeleteWordBackward => self.delete_word_backward(cursor),
            EditorAction::DeleteWordForward => self.delete_word_forward(cursor),
            // TODO: 历史与选择域后续由 editor state/history 模块承接。
            EditorAction::Undo | EditorAction::Redo | EditorAction::SelectAll => cursor,
        }
    }

    fn insert_text(&mut self, cursor: Position, text: &str) -> Position {
        let offset = self.position_to_offset(cursor);
        self.text.insert_str(offset, text);
        self.offset_to_position(offset + text.len())
    }

    fn delete_backward(&mut self, cursor: Position) -> Position {
        let offset = self.position_to_offset(cursor);
        let Some(start) = self.prev_char_start(offset) else {
            return cursor;
        };
        self.text.replace_range(start..offset, "");
        self.offset_to_position(start)
    }

    fn delete_forward(&mut self, cursor: Position) -> Position {
        let offset = self.position_to_offset(cursor);
        let Some(end) = self.next_char_end(offset) else {
            return cursor;
        };
        self.text.replace_range(offset..end, "");
        self.offset_to_position(offset)
    }

    fn delete_word_backward(&mut self, cursor: Position) -> Position {
        let offset = self.position_to_offset(cursor);
        let mut start = offset;

        while let Some(prev) = self.prev_char_start(start) {
            let ch = self.char_at(prev);
            if !is_word_boundary_char(ch) {
                break;
            }
            start = prev;
        }

        while let Some(prev) = self.prev_char_start(start) {
            let ch = self.char_at(prev);
            if is_word_boundary_char(ch) {
                break;
            }
            start = prev;
        }

        if start == offset {
            return cursor;
        }

        self.text.replace_range(start..offset, "");
        self.offset_to_position(start)
    }

    fn delete_word_forward(&mut self, cursor: Position) -> Position {
        let offset = self.position_to_offset(cursor);
        let mut end = offset;

        while let Some(next_end) = self.next_char_end(end) {
            let ch = self.char_at(end);
            if !is_word_boundary_char(ch) {
                break;
            }
            end = next_end;
        }

        while let Some(next_end) = self.next_char_end(end) {
            let ch = self.char_at(end);
            if is_word_boundary_char(ch) {
                break;
            }
            end = next_end;
        }

        if end == offset {
            return cursor;
        }

        self.text.replace_range(offset..end, "");
        self.offset_to_position(offset)
    }

    fn move_left(&self, cursor: Position) -> Position {
        let offset = self.position_to_offset(cursor);
        self.prev_char_start(offset)
            .map(|start| self.offset_to_position(start))
            .unwrap_or(cursor)
    }

    fn move_right(&self, cursor: Position) -> Position {
        let offset = self.position_to_offset(cursor);
        self.next_char_end(offset)
            .map(|end| self.offset_to_position(end))
            .unwrap_or(cursor)
    }

    fn move_vertical(&self, cursor: Position, delta: i32) -> Position {
        let line_count = self.line_count() as i32;
        let next_line = (cursor.line as i32 + delta).clamp(0, line_count.saturating_sub(1));
        let next_line = u32::try_from(next_line).unwrap_or(0);
        let next_col = cursor.column.min(self.line_len(next_line));
        Position::new(next_line, next_col)
    }

    fn clamp_position(&self, cursor: Position) -> Position {
        let line_count = self.line_count() as u32;
        let line = cursor.line.min(line_count.saturating_sub(1));
        let column = cursor.column.min(self.line_len(line));
        Position::new(line, column)
    }

    fn line_len(&self, line: u32) -> u32 {
        self.lines()
            .get(line as usize)
            .map(|text| text.chars().count() as u32)
            .unwrap_or(0)
    }

    fn position_to_offset(&self, position: Position) -> usize {
        let target = self.clamp_position(position);
        let mut line = 0u32;
        let mut col = 0u32;
        let mut iter = self.text.char_indices().peekable();

        while let Some((idx, ch)) = iter.next() {
            if line == target.line && col == target.column {
                return idx;
            }

            match ch {
                '\n' => {
                    line += 1;
                    col = 0;
                }
                '\r' => {}
                _ => {
                    if line == target.line {
                        col += 1;
                    }
                }
            }

            if iter.peek().is_none() && line == target.line && col == target.column {
                return self.text.len();
            }
        }

        self.text.len()
    }

    fn offset_to_position(&self, offset: usize) -> Position {
        let offset = offset.min(self.text.len());
        let mut line = 0u32;
        let mut column = 0u32;
        let mut current = 0usize;

        for ch in self.text.chars() {
            if current >= offset {
                break;
            }
            current += ch.len_utf8();
            match ch {
                '\n' => {
                    line += 1;
                    column = 0;
                }
                '\r' => {}
                _ => column += 1,
            }
        }

        self.clamp_position(Position::new(line, column))
    }

    fn prev_char_start(&self, offset: usize) -> Option<usize> {
        if offset == 0 {
            return None;
        }
        self.text[..offset]
            .char_indices()
            .last()
            .map(|(start, _)| start)
    }

    fn next_char_end(&self, offset: usize) -> Option<usize> {
        if offset >= self.text.len() {
            return None;
        }
        self.text[offset..]
            .chars()
            .next()
            .map(|ch| offset + ch.len_utf8())
    }

    fn char_at(&self, offset: usize) -> char {
        self.text[offset..]
            .chars()
            .next()
            .expect("offset should point to a valid char boundary")
    }
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
    use zom_protocol::{EditorAction, EditorInvocation, Position};

    use super::EditorBuffer;

    #[test]
    fn lines_and_line_ending_are_projected_from_text() {
        let buffer = EditorBuffer::from_text("a\r\n\r\nb\r\n");
        assert_eq!(buffer.line_ending(), "CRLF");
        assert_eq!(buffer.lines(), vec!["a", "", "b", ""]);
        assert_eq!(buffer.line_count(), 4);
    }

    #[test]
    fn insert_text_and_move_right_updates_cursor_and_content() {
        let mut buffer = EditorBuffer::from_text("ab");
        let mut cursor = Position::new(0, 1);

        cursor = buffer.apply_invocation(cursor, &EditorInvocation::insert_text("X"));
        assert_eq!(buffer.as_str(), "aXb");
        assert_eq!(cursor, Position::new(0, 2));

        cursor =
            buffer.apply_invocation(cursor, &EditorInvocation::Action(EditorAction::MoveRight));
        assert_eq!(cursor, Position::new(0, 3));
    }

    #[test]
    fn delete_backward_and_delete_word_forward_work() {
        let mut buffer = EditorBuffer::from_text("hello world");
        let mut cursor = Position::new(0, 6);

        cursor = buffer.apply_invocation(
            cursor,
            &EditorInvocation::Action(EditorAction::DeleteBackward),
        );
        assert_eq!(buffer.as_str(), "helloworld");
        assert_eq!(cursor, Position::new(0, 5));

        cursor = buffer.apply_invocation(
            cursor,
            &EditorInvocation::Action(EditorAction::DeleteWordForward),
        );
        assert_eq!(buffer.as_str(), "hello");
        assert_eq!(cursor, Position::new(0, 5));
    }
}
