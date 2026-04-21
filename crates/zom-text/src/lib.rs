//! zom-text 的文本缓冲区抽象与基础操作。

use std::ops::Range;

use zom_protocol::Position;

/// 轻量文本缓冲区，提供基础插入/删除与位置映射能力。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBuffer {
    text: String,
}

/// 文本区间校验错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextBufferError {
    InvalidRange {
        start: usize,
        end: usize,
        len: usize,
    },
    NotCharBoundary {
        offset: usize,
    },
}

impl TextBuffer {
    /// 创建空文本缓冲区。
    pub fn new() -> Self {
        Self::default()
    }

    /// 用给定文本创建缓冲区。
    pub fn from_text(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// 读取底层完整文本切片。
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// 返回缓冲区字节长度。
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// 判断缓冲区是否为空。
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// 返回指定区间文本切片。
    pub fn slice(&self, range: Range<usize>) -> Result<&str, TextBufferError> {
        self.validate_range(range.clone())?;
        Ok(&self.text[range])
    }

    /// 用 `text` 替换指定区间。
    pub fn replace_range(
        &mut self,
        range: Range<usize>,
        text: &str,
    ) -> Result<(), TextBufferError> {
        self.validate_range(range.clone())?;
        self.text.replace_range(range, text);
        Ok(())
    }

    /// 将逻辑位置映射到字节偏移（越界时夹紧到文档边界）。
    pub fn position_to_offset(&self, position: Position) -> usize {
        position_to_offset(&self.text, position)
    }

    /// 将字节偏移映射到行列坐标，越界时返回 `None`。
    pub fn offset_to_position(&self, offset: usize) -> Option<Position> {
        if offset > self.text.len() {
            return None;
        }
        Some(offset_to_position(&self.text, offset))
    }

    fn validate_offset(&self, offset: usize) -> Result<(), TextBufferError> {
        if offset > self.text.len() {
            return Err(TextBufferError::InvalidRange {
                start: offset,
                end: offset,
                len: self.text.len(),
            });
        }
        if !self.text.is_char_boundary(offset) {
            return Err(TextBufferError::NotCharBoundary { offset });
        }
        Ok(())
    }

    fn validate_range(&self, range: Range<usize>) -> Result<(), TextBufferError> {
        if range.start > range.end || range.end > self.text.len() {
            return Err(TextBufferError::InvalidRange {
                start: range.start,
                end: range.end,
                len: self.text.len(),
            });
        }
        self.validate_offset(range.start)?;
        self.validate_offset(range.end)?;
        Ok(())
    }
}

/// 将逻辑位置映射到字节偏移（越界时夹紧到文档边界）。
pub fn position_to_offset(text: &str, position: Position) -> usize {
    let target = clamp_position_to_text(text, position);
    let mut line = 0u32;
    let mut column = 0u32;
    let mut iter = text.char_indices().peekable();

    while let Some((index, ch)) = iter.next() {
        if line == target.line && column == target.column {
            return index;
        }

        match ch {
            '\n' => {
                line += 1;
                column = 0;
            }
            '\r' => {}
            _ => {
                if line == target.line {
                    column += 1;
                }
            }
        }

        if iter.peek().is_none() && line == target.line && column == target.column {
            return text.len();
        }
    }

    text.len()
}

/// 将字节偏移映射到逻辑位置（越界时夹紧到文档边界）。
pub fn offset_to_position(text: &str, offset: usize) -> Position {
    let clamped_offset = offset.min(text.len());
    let mut line = 0u32;
    let mut column = 0u32;
    let mut current = 0usize;

    for ch in text.chars() {
        if current >= clamped_offset {
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

    clamp_position_to_text(text, Position::new(line, column))
}

/// 将逻辑位置夹紧到文档可达范围。
pub fn clamp_position_to_text(text: &str, position: Position) -> Position {
    let line = position.line.min(line_count(text).saturating_sub(1));
    let column = position.column.min(line_len(text, line));
    Position::new(line, column)
}

fn line_count(text: &str) -> u32 {
    let line_breaks = text.chars().filter(|ch| *ch == '\n').count();
    let line_count = line_breaks.saturating_add(1);
    u32::try_from(line_count).unwrap_or(u32::MAX)
}

fn line_len(text: &str, target_line: u32) -> u32 {
    let mut line = 0u32;
    let mut column = 0u32;

    for ch in text.chars() {
        if line != target_line {
            if ch == '\n' {
                line += 1;
            }
            continue;
        }

        match ch {
            '\n' => break,
            '\r' => {}
            _ => column += 1,
        }
    }

    if line == target_line { column } else { 0 }
}

/// 按编辑器视角拆分文本行，并保留空行。
pub fn split_lines(text: &str) -> Vec<String> {
    let mut lines = text
        .split('\n')
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect::<Vec<_>>();

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// 识别文本的换行风格。
pub fn detect_line_ending(text: &str) -> String {
    if text.contains("\r\n") {
        "CRLF".into()
    } else {
        "LF".into()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TextBuffer, TextBufferError, detect_line_ending, offset_to_position, position_to_offset,
        split_lines,
    };
    use zom_protocol::Position;

    #[test]
    fn replace_range_and_slice_work() {
        let mut buffer = TextBuffer::from_text("hello");
        buffer
            .replace_range(5..5, " world")
            .expect("replace should succeed");

        assert_eq!(buffer.as_str(), "hello world");
        assert_eq!(buffer.slice(0..5).expect("slice should succeed"), "hello");
    }

    #[test]
    fn replace_range_rejects_invalid_range() {
        let mut buffer = TextBuffer::from_text("abc");
        let err = buffer
            .replace_range(2..5, "x")
            .expect_err("out of range should fail");
        assert_eq!(
            err,
            TextBufferError::InvalidRange {
                start: 2,
                end: 5,
                len: 3
            }
        );
    }

    #[test]
    fn replace_range_rejects_non_char_boundary() {
        let mut buffer = TextBuffer::from_text("a中b");
        let err = buffer
            .replace_range(1..2, "x")
            .expect_err("non-char boundary should fail");
        assert_eq!(err, TextBufferError::NotCharBoundary { offset: 2 });
    }

    #[test]
    fn offset_to_position_works() {
        let buffer = TextBuffer::from_text("ab\ncd");
        assert_eq!(buffer.offset_to_position(0), Some(Position::new(0, 0)));
        assert_eq!(buffer.offset_to_position(2), Some(Position::new(0, 2)));
        assert_eq!(buffer.offset_to_position(3), Some(Position::new(1, 0)));
        assert_eq!(buffer.offset_to_position(5), Some(Position::new(1, 2)));
        assert_eq!(buffer.offset_to_position(6), None);
    }

    #[test]
    fn position_to_offset_clamps_out_of_range_line() {
        let text = "ab\ncd";
        let offset = position_to_offset(text, Position::new(99, 0));
        assert_eq!(offset_to_position(text, offset), Position::new(1, 0));
    }

    #[test]
    fn offset_to_position_ignores_cr_in_crlf_text() {
        let text = "a\r\nb";
        assert_eq!(offset_to_position(text, 1), Position::new(0, 1));
    }

    #[test]
    fn split_lines_preserves_blank_lines() {
        let lines = split_lines("a\n\nb\n");
        assert_eq!(lines, vec!["a", "", "b", ""]);
    }

    #[test]
    fn detect_line_ending_distinguishes_crlf_and_lf() {
        assert_eq!(detect_line_ending("a\r\nb\r\n"), "CRLF");
        assert_eq!(detect_line_ending("a\nb\n"), "LF");
    }
}
