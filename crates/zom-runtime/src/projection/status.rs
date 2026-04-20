//! 状态展示投影。

use zom_protocol::Position;

/// 将零基光标位置投影为用户可读的 `line:column`（一基）。
pub fn cursor_text(position: Position) -> String {
    format!("{}:{}", position.line + 1, position.column + 1)
}

#[cfg(test)]
mod tests {
    use zom_protocol::Position;

    use super::cursor_text;

    #[test]
    fn cursor_text_uses_one_based_display() {
        assert_eq!(cursor_text(Position::new(0, 0)), "1:1");
        assert_eq!(cursor_text(Position::new(9, 3)), "10:4");
    }
}
