//! 编辑器领域命令调用载荷定义。

/// 编辑器领域的无参动作语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorAction {
    /// 打开当前活动编辑器的查找替换条。
    OpenFindReplace,
    /// 查找条：查找上一个。
    FindPrev,
    /// 查找条：查找下一个。
    FindNext,
    /// 查找条：替换下一个。
    ReplaceNext,
    /// 查找条：替换全部。
    ReplaceAll,
    /// 查找条：切换大小写匹配。
    ToggleFindCaseSensitive,
    /// 查找条：切换整词匹配。
    ToggleFindWholeWord,
    /// 查找条：切换正则模式。
    ToggleFindRegex,
    /// 插入换行。
    InsertNewline,
    /// 插入缩进。
    InsertIndent,
    /// 反缩进。
    Outdent,
    /// 光标向左移动。
    MoveLeft,
    /// 光标向右移动。
    MoveRight,
    /// 光标向上移动。
    MoveUp,
    /// 光标向下移动。
    MoveDown,
    /// 光标移动到当前行起点。
    MoveToStart,
    /// 光标移动到当前行终点。
    MoveToEnd,
    /// 光标向上移动一页。
    MovePageUp,
    /// 光标向下移动一页。
    MovePageDown,
    /// 向左扩展选区。
    SelectLeft,
    /// 向右扩展选区。
    SelectRight,
    /// 向上扩展选区。
    SelectUp,
    /// 向下扩展选区。
    SelectDown,
    /// 向当前行起点扩展选区。
    SelectToStart,
    /// 向当前行终点扩展选区。
    SelectToEnd,
    /// 向上扩展一页选区。
    SelectPageUp,
    /// 向下扩展一页选区。
    SelectPageDown,
    /// 全选。
    SelectAll,
    /// 向后删除一个字符。
    DeleteBackward,
    /// 向前删除一个字符。
    DeleteForward,
    /// 向后删除一个单词。
    DeleteWordBackward,
    /// 向前删除一个单词。
    DeleteWordForward,
    /// 复制当前选区。
    Copy,
    /// 剪切当前选区。
    Cut,
    /// 粘贴剪贴板内容。
    Paste,
    /// 撤销最近一次编辑。
    Undo,
    /// 重做最近一次撤销。
    Redo,
}

/// 查找替换动作类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FindReplaceAction {
    /// 查找下一个匹配项。
    FindNext,
    /// 查找上一个匹配项。
    FindPrev,
    /// 替换下一个匹配项。
    ReplaceNext,
    /// 替换全部匹配项。
    ReplaceAll,
}

/// 编辑器内单文件查找替换请求。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FindReplaceRequest {
    /// 查找模式（literal 或 regex）；为空时通常由执行层短路为 no-op。
    pub query: String,
    /// 替换文本（仅替换操作使用）。
    pub replacement: String,
    /// 执行动作。
    pub action: FindReplaceAction,
    /// 是否区分大小写。
    pub case_sensitive: bool,
    /// 是否全词匹配。
    pub whole_word: bool,
    /// 是否按正则表达式解释 `query`；为 `false` 时应先进行转义再匹配。
    pub use_regex: bool,
}

impl FindReplaceRequest {
    /// 构造查找替换请求。
    ///
    /// 该构造只组装协议数据，不做正则合法性等运行时校验。
    pub fn new(
        query: impl Into<String>,
        replacement: impl Into<String>,
        action: FindReplaceAction,
        case_sensitive: bool,
        whole_word: bool,
        use_regex: bool,
    ) -> Self {
        Self {
            query: query.into(),
            replacement: replacement.into(),
            action,
            case_sensitive,
            whole_word,
            use_regex,
        }
    }
}

/// 编辑器命令的运行时调用。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EditorInvocation {
    /// 调用一个无参编辑动作。
    Action(EditorAction),
    /// 插入一段动态文本。
    InsertText {
        /// 需要插入到光标位置的文本内容（支持多字符批量输入）。
        text: String,
    },
    /// 单文件查找替换请求。
    FindReplace {
        /// 请求参数。
        request: FindReplaceRequest,
    },
}

impl EditorInvocation {
    /// 构造一次文本插入调用。
    ///
    /// 用于承载动态 payload，因此与 `EditorAction`（无参动作）分离。
    pub fn insert_text(text: impl Into<String>) -> Self {
        Self::InsertText { text: text.into() }
    }

    /// 构造一次查找替换调用。
    pub fn find_replace(request: FindReplaceRequest) -> Self {
        Self::FindReplace { request }
    }
}

impl From<EditorAction> for EditorInvocation {
    fn from(action: EditorAction) -> Self {
        Self::Action(action)
    }
}
