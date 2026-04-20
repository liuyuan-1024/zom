//! zom-text 的文本缓冲区抽象与基础操作。

use zom_protocol::Position;

/// 轻量文本缓冲区，提供基础插入/删除与位置映射能力。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBuffer {
    text: String,
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

    /// 在指定字节偏移处插入字符串。
    pub fn insert_str(&mut self, offset: usize, value: &str) {
        self.text.insert_str(offset, value);
    }

    /// 删除给定字节区间内的文本。
    pub fn remove_range(&mut self, start: usize, end: usize) {
        self.text.replace_range(start..end, "");
    }

    /// 返回缓冲区字节长度。
    pub fn len_bytes(&self) -> usize {
        self.text.len()
    }

    /// 判断缓冲区是否为空。
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// 将字节偏移映射到行列坐标，越界时返回 `None`。
    pub fn offset_to_position(&self, offset: usize) -> Option<Position> {
        if offset > self.text.len() {
            return None;
        }

        let mut line = 0u32;
        let mut column = 0u32;
        let mut current = 0usize;

        for ch in self.text.chars() {
            if current >= offset {
                break;
            }
            current += ch.len_utf8();
            if ch == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }

        Some(Position::new(line, column))
    }
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
    use super::{TextBuffer, detect_line_ending, split_lines};
    use zom_protocol::Position;

    #[test]
    fn insert_and_remove_text() {
        let mut buffer = TextBuffer::from_text("hello");
        buffer.insert_str(5, " world");
        assert_eq!(buffer.as_str(), "hello world");

        buffer.remove_range(5, 11);
        assert_eq!(buffer.as_str(), "hello");
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
