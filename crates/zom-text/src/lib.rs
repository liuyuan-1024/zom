//! zom-text 的文本缓冲区抽象与基础操作。

mod line_index;

use std::{fmt, ops::Range};

use line_index::LineIndex;
use ropey::Rope;
use zom_protocol::Position;
use zom_text_tokens::{LineEnding, CR_BYTE, CR_CHAR, LF_BYTE, LF_CHAR};

/// 轻量文本缓冲区，提供基础插入/删除与位置映射能力。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBuffer {
    /// 底层 Rope 存储；以字符索引为主能力，外层负责桥接字节偏移协议。
    rope: Rope,
}

/// 文本区间校验错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextBufferError {
    /// 字节区间越界或 `start > end`。
    InvalidRange {
        start: usize,
        end: usize,
        len: usize,
    },
    /// 偏移不在 UTF-8 字符边界，无法安全切片/替换。
    NotCharBoundary { offset: usize },
}

impl TextBuffer {
    /// 创建空文本缓冲区。
    pub fn new() -> Self {
        Self::default()
    }

    /// 用给定文本创建缓冲区。
    pub fn from_text(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            rope: Rope::from_str(&text),
        }
    }

    /// 返回底层 rope 只读视图。
    ///
    /// 主要用于只读高级能力（例如按行/按字符遍历），避免复制文本。
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// 返回缓冲区字节长度。
    pub fn len(&self) -> usize {
        self.rope.len_bytes()
    }

    /// 判断缓冲区是否为空。
    pub fn is_empty(&self) -> bool {
        self.rope.len_bytes() == 0
    }

    /// 返回指定字节区间文本切片。
    ///
    /// 输入必须满足 UTF-8 边界约束，否则返回 `NotCharBoundary`。
    pub fn slice(&self, range: Range<usize>) -> Result<String, TextBufferError> {
        self.validate_byte_range(range.clone())?;
        let start_char = self.rope.byte_to_char(range.start);
        let end_char = self.rope.byte_to_char(range.end);
        Ok(self.rope.slice(start_char..end_char).to_string())
    }

    /// 用 `text` 替换指定字节区间。
    ///
    /// 先校验边界，再按字符索引执行 `remove + insert`，保证多字节字符不被截断。
    pub fn replace_range(
        &mut self,
        range: Range<usize>,
        text: &str,
    ) -> Result<(), TextBufferError> {
        self.validate_byte_range(range.clone())?;
        let start_char = self.rope.byte_to_char(range.start);
        let end_char = self.rope.byte_to_char(range.end);
        self.rope.remove(start_char..end_char);
        self.rope.insert(start_char, text);
        Ok(())
    }

    /// 将逻辑位置映射到字节偏移（越界时夹紧到文档边界）。
    ///
    /// 列语义按“可视列”计算：`\r` 不计列宽，`\n` 终止当前行。
    pub fn position_to_offset(&self, position: Position) -> usize {
        LineIndex::new(&self.rope).position_to_offset(position)
    }

    /// 将字节偏移映射到行列坐标，越界时返回 `None`。
    ///
    /// 仅当偏移不超过文本末尾才可映射；列统计同样忽略 `\r`。
    pub fn offset_to_position(&self, offset: usize) -> Option<Position> {
        LineIndex::new(&self.rope).offset_to_position(offset)
    }

    /// 文档总行数（最少为 1）。
    ///
    /// 与大多数编辑器一致，空文本视为单行空行。
    pub fn line_count(&self) -> u32 {
        LineIndex::new(&self.rope).line_count()
    }

    /// 指定行的可视列宽（忽略 `\r`，不计入换行符）。
    pub fn line_len(&self, line: u32) -> u32 {
        LineIndex::new(&self.rope).line_len(line)
    }

    /// 将位置夹紧到当前文档范围。
    ///
    /// 先夹行再夹列，避免把列夹到错误行宽。
    pub fn clamp_position(&self, position: Position) -> Position {
        LineIndex::new(&self.rope).clamp_position(position)
    }

    /// 返回指定字节偏移对应“前一个字符”的起始偏移。
    ///
    /// 要求 `offset` 本身是字符边界；落在字符中间时返回 `None`。
    pub fn prev_char_start(&self, offset: usize) -> Option<usize> {
        if offset == 0 || offset > self.rope.len_bytes() {
            return None;
        }
        let char_index = self.rope.byte_to_char(offset);
        if self.rope.char_to_byte(char_index) != offset || char_index == 0 {
            return None;
        }
        Some(self.rope.char_to_byte(char_index - 1))
    }

    /// 返回指定字节偏移处字符的结束偏移（下一个字符边界）。
    ///
    /// 要求 `offset` 位于当前字符起点，且不超过最后一个字符。
    pub fn next_char_end(&self, offset: usize) -> Option<usize> {
        if offset >= self.rope.len_bytes() {
            return None;
        }
        let char_index = self.rope.byte_to_char(offset);
        if self.rope.char_to_byte(char_index) != offset || char_index >= self.rope.len_chars() {
            return None;
        }
        Some(self.rope.char_to_byte(char_index + 1))
    }

    /// 读取指定字节偏移处的字符（需位于字符边界）。
    pub fn char_at(&self, offset: usize) -> Option<char> {
        if offset >= self.rope.len_bytes() {
            return None;
        }
        let char_index = self.rope.byte_to_char(offset);
        if self.rope.char_to_byte(char_index) != offset || char_index >= self.rope.len_chars() {
            return None;
        }
        Some(self.rope.char(char_index))
    }

    /// 校验字节区间合法且位于 UTF-8 字符边界。
    pub fn validate_byte_range(&self, range: Range<usize>) -> Result<(), TextBufferError> {
        self.validate_range(range)
    }

    /// 校验单个偏移是否合法且落在字符边界。
    fn validate_offset(&self, offset: usize) -> Result<(), TextBufferError> {
        if offset > self.rope.len_bytes() {
            return Err(TextBufferError::InvalidRange {
                start: offset,
                end: offset,
                len: self.rope.len_bytes(),
            });
        }
        let char_index = self.rope.byte_to_char(offset);
        if self.rope.char_to_byte(char_index) != offset {
            return Err(TextBufferError::NotCharBoundary { offset });
        }
        Ok(())
    }

    /// 校验字节范围：顺序合法、未越界、两端都在字符边界。
    fn validate_range(&self, range: Range<usize>) -> Result<(), TextBufferError> {
        if range.start > range.end || range.end > self.rope.len_bytes() {
            return Err(TextBufferError::InvalidRange {
                start: range.start,
                end: range.end,
                len: self.rope.len_bytes(),
            });
        }
        self.validate_offset(range.start)?;
        self.validate_offset(range.end)?;
        Ok(())
    }
}

impl fmt::Display for TextBuffer {
    /// 为 `fmt` 输出稳定的文本表示。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.rope.to_string())
    }
}

/// 将逻辑位置映射到字节偏移（基于 `&str` 的轻量实现）。
///
/// 与 `TextBuffer::position_to_offset` 语义对齐：忽略 `\r` 列宽并按文档边界夹紧。
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
            LF_CHAR => {
                line += 1;
                column = 0;
            }
            CR_CHAR => {}
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

/// 将字节偏移映射到逻辑位置（基于 `&str` 的轻量实现）。
///
/// 越界偏移会先夹到文本末尾，再映射到最近可达位置。
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
            LF_CHAR => {
                line += 1;
                column = 0;
            }
            CR_CHAR => {}
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
    let line_breaks = text.chars().filter(|ch| *ch == LF_CHAR).count();
    let line_count = line_breaks.saturating_add(1);
    u32::try_from(line_count).unwrap_or(u32::MAX)
}

/// 返回目标行的可视列宽（忽略行尾 `\r`），超出行数范围时返回 `0`。
fn line_len(text: &str, target_line: u32) -> u32 {
    let mut line = 0u32;
    let mut column = 0u32;

    for ch in text.chars() {
        if line != target_line {
            if ch == LF_CHAR {
                line += 1;
            }
            continue;
        }

        match ch {
            LF_CHAR => break,
            CR_CHAR => {}
            _ => column += 1,
        }
    }

    if line == target_line {
        column
    } else {
        0
    }
}

/// 按编辑器视角拆分文本行，并保留空行。
///
/// 该函数按 `\n` 分行，并去掉每行末尾的 `\r`，用于统一 CRLF/LF 读取表现。
pub fn split_lines(text: &str) -> Vec<String> {
    let mut lines = text
        .split(LF_CHAR)
        .map(|line| line.trim_end_matches(CR_CHAR).to_string())
        .collect::<Vec<_>>();

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// 识别文本的换行风格。
///
/// 同时出现多种换行符时返回 `Mixed`，未发现换行时默认视为 `LF`。
pub fn detect_line_ending(text: &str) -> LineEnding {
    let bytes = text.as_bytes();
    let mut has_crlf = false;
    let mut has_lf = false;
    let mut has_cr = false;

    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            CR_BYTE => {
                if bytes.get(index + 1) == Some(&LF_BYTE) {
                    has_crlf = true;
                    index += 2;
                    continue;
                }
                has_cr = true;
                index += 1;
            }
            LF_BYTE => {
                has_lf = true;
                index += 1;
            }
            _ => index += 1,
        }
    }

    let kinds = usize::from(has_crlf) + usize::from(has_lf) + usize::from(has_cr);
    if kinds > 1 {
        LineEnding::Mixed
    } else if has_crlf {
        LineEnding::Crlf
    } else if has_lf {
        LineEnding::Lf
    } else if has_cr {
        LineEnding::Cr
    } else {
        LineEnding::Lf
    }
}

#[cfg(test)]
mod tests {
    use super::{
        detect_line_ending, offset_to_position, position_to_offset, split_lines, TextBuffer,
        TextBufferError,
    };
    use zom_protocol::Position;

    #[test]
    /// 替换范围并同步相关状态。
    fn replace_range_and_slice_work() {
        let mut buffer = TextBuffer::from_text("hello");
        buffer
            .replace_range(5..5, " world")
            .expect("replace should succeed");

        assert_eq!(buffer.to_string(), "hello world");
        assert_eq!(buffer.slice(0..5).expect("slice should succeed"), "hello");
    }

    #[test]
    /// 替换范围并同步相关状态。
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
    /// 替换范围并同步相关状态。
    fn replace_range_rejects_non_char_boundary() {
        let mut buffer = TextBuffer::from_text("a中b");
        let err = buffer
            .replace_range(1..2, "x")
            .expect_err("non-char boundary should fail");
        assert_eq!(err, TextBufferError::NotCharBoundary { offset: 2 });
    }

    #[test]
    /// 计算位置结果。
    fn offset_to_position_works() {
        let buffer = TextBuffer::from_text("ab\ncd");
        assert_eq!(buffer.offset_to_position(0), Some(Position::new(0, 0)));
        assert_eq!(buffer.offset_to_position(2), Some(Position::new(0, 2)));
        assert_eq!(buffer.offset_to_position(3), Some(Position::new(1, 0)));
        assert_eq!(buffer.offset_to_position(5), Some(Position::new(1, 2)));
        assert_eq!(buffer.offset_to_position(6), None);
    }

    #[test]
    /// 计算偏移范围行结果。
    fn position_to_offset_clamps_out_of_range_line() {
        let text = "ab\ncd";
        let offset = position_to_offset(text, Position::new(99, 0));
        assert_eq!(offset_to_position(text, offset), Position::new(1, 0));
    }

    #[test]
    /// 计算位置文本结果。
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
    fn detect_line_ending_distinguishes_common_styles() {
        assert_eq!(
            detect_line_ending("a\r\nb\r\n"),
            zom_text_tokens::LineEnding::Crlf
        );
        assert_eq!(
            detect_line_ending("a\nb\n"),
            zom_text_tokens::LineEnding::Lf
        );
        assert_eq!(
            detect_line_ending("a\rb\r"),
            zom_text_tokens::LineEnding::Cr
        );
    }

    #[test]
    fn detect_line_ending_reports_mixed_styles() {
        assert_eq!(
            detect_line_ending("a\r\nb\n"),
            zom_text_tokens::LineEnding::Mixed
        );
        assert_eq!(
            detect_line_ending("a\rb\n"),
            zom_text_tokens::LineEnding::Mixed
        );
    }

    #[test]
    /// 计算位置偏移结果。
    fn unicode_position_and_offset_mapping_remains_stable() {
        let text = "a🙂中\ne\u{301}f";

        assert_eq!(position_to_offset(text, Position::new(0, 0)), 0);
        assert_eq!(position_to_offset(text, Position::new(0, 1)), 1);
        assert_eq!(position_to_offset(text, Position::new(0, 2)), 5);
        assert_eq!(position_to_offset(text, Position::new(0, 3)), 8);
        assert_eq!(position_to_offset(text, Position::new(1, 0)), 9);
        assert_eq!(position_to_offset(text, Position::new(1, 1)), 10);
        assert_eq!(position_to_offset(text, Position::new(1, 2)), 12);
        assert_eq!(position_to_offset(text, Position::new(1, 3)), 13);

        assert_eq!(offset_to_position(text, 0), Position::new(0, 0));
        assert_eq!(offset_to_position(text, 1), Position::new(0, 1));
        assert_eq!(offset_to_position(text, 5), Position::new(0, 2));
        assert_eq!(offset_to_position(text, 8), Position::new(0, 3));
        assert_eq!(offset_to_position(text, 9), Position::new(1, 0));
        assert_eq!(offset_to_position(text, 10), Position::new(1, 1));
        assert_eq!(offset_to_position(text, 12), Position::new(1, 2));
        assert_eq!(offset_to_position(text, 13), Position::new(1, 3));
    }

    #[test]
    /// 计算缓冲区结果。
    fn text_buffer_char_boundary_helpers_handle_multibyte_characters() {
        let buffer = TextBuffer::from_text("🙂a");

        assert_eq!(buffer.next_char_end(0), Some(4));
        assert_eq!(buffer.prev_char_start(4), Some(0));
        assert_eq!(buffer.next_char_end(4), Some(5));
        assert_eq!(buffer.prev_char_start(5), Some(4));
        assert_eq!(buffer.prev_char_start(1), None);
        assert_eq!(buffer.char_at(0), Some('🙂'));
        assert_eq!(buffer.char_at(1), None);
    }

    #[test]
    /// 替换范围并同步相关状态。
    fn replace_range_rejects_ranges_that_split_emoji_bytes() {
        let mut buffer = TextBuffer::from_text("a🙂b");

        let err = buffer
            .replace_range(2..5, "x")
            .expect_err("split multibyte boundary should fail");
        assert_eq!(err, TextBufferError::NotCharBoundary { offset: 2 });
    }
}
