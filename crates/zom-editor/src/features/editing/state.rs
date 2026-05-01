use zom_protocol::{Position, Selection};
use zom_text::TextBuffer;

/// 编辑器文档版本号，用于 editor/runtime 之间的乐观并发控制。
///
/// `EditorState` 每次提交有效变更（文本或选区）都会递增版本，
/// 上层可用它拒绝过期请求，避免“旧操作覆盖新状态”。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocVersion(u64);

impl DocVersion {
    /// 新建文档快照的初始版本。
    pub const fn zero() -> Self {
        Self(0)
    }

    /// 返回严格递增后的版本值。
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// 导出底层计数值，供序列化和协议层互操作使用。
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for DocVersion {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

pub type Offset = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorState {
    /// 文本真值源；所有位置/偏移换算都依赖该缓冲区。
    buffer: TextBuffer,
    /// 逻辑坐标（`line`,`column`）下的选区（anchor/active）。
    selection: Selection,
    /// 当前快照版本，供事务和 runtime 桥接层做并发校验。
    version: DocVersion,
}

impl EditorState {
    /// 用给定文本构造状态：光标默认在 `(0,0)`，版本为 `0`。
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            buffer: TextBuffer::from_text(text),
            selection: Selection::caret(Position::zero()),
            version: DocVersion::zero(),
        }
    }

    pub fn text(&self) -> String {
        self.buffer.to_string()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    /// 返回当前选区文本；若为 caret 或切片边界异常，则返回 `None`。
    pub fn selected_text(&self) -> Option<String> {
        let selection = self.selection();
        if selection.is_caret() {
            return None;
        }
        let from = self.position_to_offset(selection.start());
        let to = self.position_to_offset(selection.end());
        if from >= to {
            return None;
        }
        self.text().get(from..to).map(ToOwned::to_owned)
    }

    /// 逻辑位置转 UTF-8 字节偏移；越界输入按 `TextBuffer` 规则自动钳制。
    pub fn position_to_offset(&self, position: Position) -> Offset {
        self.buffer.position_to_offset(position)
    }

    /// 字节偏移转逻辑位置；会先钳制到当前文档末尾，保证总能落在有效位置。
    pub fn offset_to_position(&self, offset: Offset) -> Position {
        self.buffer
            .offset_to_position(offset.min(self.buffer.len()))
            .expect("clamped offset should always map to a position")
    }

    pub fn version(&self) -> DocVersion {
        self.version
    }

    pub(crate) fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

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

/// 通过 `position -> offset -> position` 回写选区，统一修正非法 anchor/active。
///
/// 这个过程可同时处理越界坐标、多字节字符边界和“落在文档末尾之后”的情况。
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

    #[test]
    fn selected_text_returns_none_for_caret_and_some_for_range() {
        let base = EditorState::from_text("abcd");
        let caret = EditorState::from_parts(
            base.buffer().clone(),
            Selection::caret(Position::new(0, 1)),
            base.version(),
        );
        assert_eq!(caret.selected_text(), None);

        let range = EditorState::from_parts(
            base.buffer().clone(),
            Selection::new(Position::new(0, 1), Position::new(0, 3)),
            base.version(),
        );
        assert_eq!(range.selected_text().as_deref(), Some("bc"));
    }
}
