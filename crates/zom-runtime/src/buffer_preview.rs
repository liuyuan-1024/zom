//! 文件缓冲区预览加载能力。

use std::{fs, path::Path};

use zom_editor::EditorBuffer;
use zom_protocol::Position;

/// 文件加载后的预览数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferPreview {
    /// 编辑器缓冲区。
    pub buffer: EditorBuffer,
    /// 当前文件换行符格式。
    pub line_ending: String,
    /// 光标逻辑位置（零基行列）。
    pub cursor: Position,
}

/// 读取真实文件内容并构建预览数据。
pub fn load_buffer_preview(path: &Path) -> BufferPreview {
    let buffer = match fs::read_to_string(path) {
        Ok(text) => EditorBuffer::from_text(text),
        Err(_) => EditorBuffer::from_text(format!("// failed to read {}", path.display())),
    };

    let line_ending = buffer.line_ending();
    let cursor_line = u32::try_from(buffer.line_count().saturating_sub(1)).unwrap_or(u32::MAX);
    let cursor = Position::new(cursor_line, 0);

    BufferPreview {
        buffer,
        line_ending,
        cursor,
    }
}
