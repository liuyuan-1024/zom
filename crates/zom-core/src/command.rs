//! 命令协议层。
//! 这里表达的是“用户想做什么”，而不是“具体如何执行”。

/// 跨系统共享的顶层命令。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// 作用于文本编辑器的命令。
    Editor(EditorCommand),
    /// 作用于工作台或界面的命令。
    Workspace(WorkspaceCommand),
}

/// 编辑器领域的命令语义。
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// 工作台领域的命令语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCommand {
    /// 关闭当前激活项。
    CloseActiveItem,
    /// 打开命令面板。
    OpenCommandPalette,
    /// 打开文件查找器。
    OpenFileFinder,
    /// 切换侧边栏显示状态。
    ToggleSidebar,
    /// 将焦点切回编辑器。
    FocusEditor,
    /// 将焦点切到侧边栏。
    FocusSidebar,
    /// 将焦点切到命令面板。
    FocusPalette,
}
