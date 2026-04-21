//! 文件缓冲区预览加载能力。

use std::{fs, path::Path};

use zom_editor::EditorState;
use zom_protocol::Position;
use zom_text::{detect_line_ending, split_lines};

/// 文件加载后的预览数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferPreview {
    /// 编辑器状态。
    pub editor_state: EditorState,
    /// 当前文件换行符格式。
    pub line_ending: String,
    /// 光标逻辑位置（零基行列）。
    pub cursor: Position,
}

/// 读取真实文件内容并构建预览数据。
pub fn load_buffer_preview(path: &Path) -> BufferPreview {
    let editor_state = match fs::read_to_string(path) {
        Ok(text) => EditorState::from_text(text),
        Err(_) => EditorState::from_text(format!("// failed to read {}", path.display())),
    };

    let line_ending = detect_line_ending(editor_state.text());
    let cursor_line =
        u32::try_from(split_lines(editor_state.text()).len().saturating_sub(1)).unwrap_or(u32::MAX);
    let cursor = Position::new(cursor_line, 0);

    BufferPreview {
        editor_state,
        line_ending,
        cursor,
    }
}
