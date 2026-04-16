use std::{env, fs, path::PathBuf};
use zom_text::TextBuffer;

/// 生成工作区文件的绝对路径。
pub fn workspace_file_absolute_path(relative_path: &str) -> PathBuf {
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(relative_path)
}

/// 推断当前工作区项目名称。
pub fn detect_workspace_project_name() -> String {
    env::current_dir()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "workspace".into())
}

/// 读取真实文件内容，并转换成界面需要的预览数据。
pub fn load_buffer_preview(path: &PathBuf) -> (Vec<String>, String, String) {
    let Ok(text) = fs::read_to_string(path) else {
        return (
            vec![format!("// failed to read {}", path.display())],
            "LF".into(),
            "1:1".into(),
        );
    };

    let buffer = TextBuffer::from_text(text.clone());
    let lines = split_lines(buffer.as_str());
    let line_ending = detect_line_ending(&text);
    let cursor = format!("{}:{}", lines.len().max(1), 1);

    (lines, line_ending, cursor)
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
