//! 应用层通用路径与文本工具函数。

use std::{
    env, fs,
    path::{Path, PathBuf},
};
use zom_text::TextBuffer;

/// 生成工作区文件的绝对路径。
pub fn workspace_file_absolute_path(workspace_root: &Path, relative_path: &str) -> PathBuf {
    workspace_root.join(relative_path)
}

/// 推断当前进程所在目录，作为工作区根目录的默认值。
pub fn detect_workspace_root() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 规范化工作区目录：优先返回规范路径，失败时回退到原路径。
pub fn normalize_workspace_root(path: impl Into<PathBuf>) -> PathBuf {
    let path = path.into();
    fs::canonicalize(&path).unwrap_or(path)
}

/// 从工作区根目录推断项目名称。
pub fn project_name_from_root(workspace_root: &Path) -> String {
    workspace_root
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "workspace".into())
}

/// 从相对路径提取文件名，作为标签标题等展示用途。
pub fn file_name_from_path(relative_path: &str) -> String {
    Path::new(relative_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| relative_path.to_string())
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
