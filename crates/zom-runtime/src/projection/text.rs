//! 文本相关展示投影。

/// 按字符数把长行拆成多个显示段，供查看器模式软换行使用。
pub fn wrap_visual_line(line: &str, max_chars_per_line: usize) -> Vec<String> {
    zom_editor::wrap_visual_line(line, max_chars_per_line)
}

#[cfg(test)]
mod tests {
    use super::wrap_visual_line;

    #[test]
    fn wrap_visual_line_preserves_empty_line() {
        assert_eq!(wrap_visual_line("", 10), vec![String::new()]);
    }

    #[test]
    fn wrap_visual_line_splits_by_max_chars() {
        assert_eq!(
            wrap_visual_line("abcdef", 2),
            vec!["ab".to_string(), "cd".to_string(), "ef".to_string()]
        );
    }

    #[test]
    fn wrap_visual_line_guards_against_zero_width() {
        assert_eq!(
            wrap_visual_line("abc", 0),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
    }
}
