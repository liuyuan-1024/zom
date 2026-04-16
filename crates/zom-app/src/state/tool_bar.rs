/// 工具栏展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolBarEntry {
    /// 图标语义。
    pub icon: ToolBarIcon,
}

/// 工具栏使用的图标语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolBarIcon {
    /// 文件树入口。
    Files,
    /// Git 入口。
    GitBranch,
    /// Outline 入口。
    Outline,
    /// 搜索入口。
    Search,
    /// LSP 入口。
    LanguageServer,
    /// 终端入口。
    Terminal,
    /// 调试入口。
    Debug,
    /// 通知入口。
    Notifications,
}

/// 工具栏展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolBarState {
    /// 左侧工具入口。
    pub left_tools: Vec<ToolBarEntry>,
    /// 光标位置文本。
    pub cursor: String,
    /// 当前文本语言类型。
    pub language: String,
    /// 当前文件换行符格式。
    pub line_ending: String,
    /// 当前文件编码。
    pub encoding: String,
    /// 右侧工具入口。
    pub right_tools: Vec<ToolBarEntry>,
}
