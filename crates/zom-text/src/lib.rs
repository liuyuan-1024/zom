use zom_core::Position;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBuffer {
    text: String,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_text(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn insert_str(&mut self, offset: usize, value: &str) {
        self.text.insert_str(offset, value);
    }

    pub fn remove_range(&mut self, start: usize, end: usize) {
        self.text.replace_range(start..end, "");
    }

    pub fn len_bytes(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn offset_to_position(&self, offset: usize) -> Option<Position> {
        if offset > self.text.len() {
            return None;
        }

        let mut row = 0u32;
        let mut col = 0u32;
        let mut current = 0usize;

        for ch in self.text.chars() {
            if current >= offset {
                break;
            }
            current += ch.len_utf8();
            if ch == '\n' {
                row += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        Some(Position::new(row, col))
    }
}

#[cfg(test)]
mod tests {
    use super::TextBuffer;
    use zom_core::Position;

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
}
