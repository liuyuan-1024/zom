//! 文件缓冲区预览加载能力。

use std::{fs, path::Path};

use zom_editor::EditorState;
use zom_text::detect_line_ending;
use zom_text_tokens::{CR_CHAR, CRLF, LF, LineEnding};

/// 文件加载后的预览数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferPreview {
    /// 编辑器状态。
    pub editor_state: EditorState,
    /// 原始文件换行符格式（用于保存时 preserve）。
    pub line_ending: LineEnding,
}

/// 读取缓冲区预览失败原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadBufferPreviewError {
    /// 文件读取失败。
    ReadFailed,
    /// 文件不是 UTF-8 文本。
    NonUtf8Text,
}

/// 读取真实文件内容并构建预览数据。
pub fn load_buffer_preview(path: &Path) -> Result<BufferPreview, LoadBufferPreviewError> {
    let bytes = fs::read(path).map_err(|_| LoadBufferPreviewError::ReadFailed)?;
    let text = String::from_utf8(bytes).map_err(|_| LoadBufferPreviewError::NonUtf8Text)?;
    Ok(BufferPreview {
        line_ending: detect_line_ending(&text),
        editor_state: EditorState::from_text(normalize_to_lf(&text)),
    })
}

fn normalize_to_lf(text: &str) -> String {
    text.replace(CRLF, LF).replace(CR_CHAR, LF)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{LoadBufferPreviewError, load_buffer_preview, normalize_to_lf};

    #[test]
    fn normalize_to_lf_collapses_crlf_and_cr() {
        assert_eq!(normalize_to_lf("a\r\nb\rc"), "a\nb\nc");
    }

    #[test]
    fn load_buffer_preview_rejects_missing_file() {
        let missing = std::env::temp_dir().join("zom-missing-buffer-preview-file");
        let result = load_buffer_preview(&missing);
        assert_eq!(result, Err(LoadBufferPreviewError::ReadFailed));
    }

    #[test]
    fn load_buffer_preview_rejects_non_utf8_text() {
        let path = std::env::temp_dir().join("zom-non-utf8-buffer-preview.bin");
        fs::write(&path, [0xff, 0xfe]).expect("write non-utf8 bytes");
        let result = load_buffer_preview(&path);
        fs::remove_file(&path).expect("cleanup non-utf8 test file");
        assert_eq!(result, Err(LoadBufferPreviewError::NonUtf8Text));
    }
}
