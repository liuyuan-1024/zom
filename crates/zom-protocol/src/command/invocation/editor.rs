//! 编辑器领域命令调用载荷定义。

/// 编辑器领域的无参动作语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorAction {
    /// 插入换行。
    InsertNewline,
    /// 光标向左移动。
    MoveLeft,
    /// 光标向右移动。
    MoveRight,
    /// 光标向上移动。
    MoveUp,
    /// 光标向下移动。
    MoveDown,
    /// 移动到当前行起点。
    MoveToStart,
    /// 移动到当前行终点。
    MoveToEnd,
    /// 向上翻一页。
    MovePageUp,
    /// 向下翻一页。
    MovePageDown,
    /// 向后删除一个字符。
    DeleteBackward,
    /// 向前删除一个字符。
    DeleteForward,
    /// 向后删除一个单词。
    DeleteWordBackward,
    /// 向前删除一个单词。
    DeleteWordForward,
    /// 撤销最近一次编辑。
    Undo,
    /// 重做最近一次撤销。
    Redo,
    /// 全选。
    SelectAll,
}

/// 编辑器命令的运行时调用。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorInvocation {
    /// 调用一个无参编辑动作。
    Action(EditorAction),
    /// 插入一段动态文本。
    InsertText {
        /// 需要插入到光标位置的文本内容。
        text: String,
    },
}

impl EditorInvocation {
    /// 构造一次文本插入调用。
    pub fn insert_text(text: impl Into<String>) -> Self {
        Self::InsertText { text: text.into() }
    }
}

impl From<EditorAction> for EditorInvocation {
    fn from(action: EditorAction) -> Self {
        Self::Action(action)
    }
}
