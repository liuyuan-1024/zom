//! 编辑器文本缓冲区领域模型。

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
}

#[cfg(test)]
mod tests {
    use super::EditorBuffer;

    #[test]
    fn lines_and_line_ending_are_projected_from_text() {
        let buffer = EditorBuffer::from_text("a\r\n\r\nb\r\n");
        assert_eq!(buffer.line_ending(), "CRLF");
        assert_eq!(buffer.lines(), vec!["a", "", "b", ""]);
        assert_eq!(buffer.line_count(), 4);
    }
}
