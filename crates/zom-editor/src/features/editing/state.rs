//! 编辑器核心状态定义。

use zom_protocol::{Position, Selection};
use zom_text::TextBuffer;

/// 文档版本号。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocVersion(u64);

impl DocVersion {
    /// 返回初始版本。
    pub const fn zero() -> Self {
        Self(0)
    }

    /// 返回下一个版本号。
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// 读取内部值。
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for DocVersion {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// 文本偏移（字节单位）。
pub type Offset = usize;

/// 编辑器状态快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorState {
    buffer: TextBuffer,
    selection: Selection,
    version: DocVersion,
}

impl EditorState {
    /// 用文本创建初始状态，默认版本号为 0、光标位于起点。
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            buffer: TextBuffer::from_text(text),
            selection: Selection::caret(Position::zero()),
            version: DocVersion::zero(),
        }
    }

    /// 返回当前完整文本副本（面向快照/序列化场景）。
    pub fn text(&self) -> String {
        self.buffer.to_string()
    }

    /// 返回文本长度（字节）。
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// 判断当前文档是否为空。
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 返回当前选区。
    pub fn selection(&self) -> Selection {
        self.selection
    }

    /// 将逻辑位置映射到字节偏移（越界时夹紧到文档边界）。
    pub fn position_to_offset(&self, position: Position) -> Offset {
        self.buffer.position_to_offset(position)
    }

    /// 将字节偏移映射到逻辑位置（越界时夹紧到文档边界）。
    pub fn offset_to_position(&self, offset: Offset) -> Position {
        self.buffer
            .offset_to_position(offset.min(self.buffer.len()))
            .expect("clamped offset should always map to a position")
    }

    /// 返回当前版本号。
    pub fn version(&self) -> DocVersion {
        self.version
    }

    /// 读取底层文本缓冲区（仅供编辑域内部复用）。
    pub(crate) fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    /// 由既有部件组装状态（用于事务推进后的状态重建）。
    pub(crate) fn from_parts(
        buffer: TextBuffer,
        selection: Selection,
        version: DocVersion,
    ) -> Self {
        Self {
            buffer,
            selection,
            version,
        }
    }
}

/// 将选区夹紧到给定文本范围内。
pub fn clamp_selection_to_text(buffer: &TextBuffer, selection: Selection) -> Selection {
    let anchor_offset = buffer.position_to_offset(selection.anchor());
    let active_offset = buffer.position_to_offset(selection.active());
    Selection::new(
        buffer
            .offset_to_position(anchor_offset)
            .expect("mapped anchor offset should be valid"),
        buffer
            .offset_to_position(active_offset)
            .expect("mapped active offset should be valid"),
    )
}

#[cfg(test)]
mod tests {
    use zom_protocol::{Position, Selection};

    use super::{DocVersion, EditorState};

    #[test]
    fn state_defaults_to_zero_version_and_caret() {
        let state = EditorState::from_text("abc");

        assert_eq!(state.version(), DocVersion::zero());
        assert_eq!(state.selection(), Selection::caret(Position::zero()));
        assert_eq!(state.text(), "abc");
    }

    #[test]
    fn position_and_offset_mapping_roundtrip() {
        let state = EditorState::from_text("ab\n中d");
        let offset = state.position_to_offset(Position::new(1, 1));
        assert_eq!(state.offset_to_position(offset), Position::new(1, 1));
    }
}
