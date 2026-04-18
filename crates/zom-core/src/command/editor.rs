/// 编辑器领域的命令语义。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorCommand {
    /// 插入一段文本。
    InsertText(String),
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
