/// 编辑器标签页的摘要信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferSummary {
    /// 标签页标题。
    pub title: String,
    /// 该标签页是否为当前激活项。
    pub is_active: bool,
}

/// 文件树节点类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTreeNodeKind {
    /// 目录节点。
    Directory,
    /// 文件节点。
    File,
}

/// 文件树中的单个节点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeNode {
    /// 节点显示名称。
    pub name: String,
    /// 节点在工作区中的相对路径。
    pub path: String,
    /// 节点类型。
    pub kind: FileTreeNodeKind,
    /// 当前节点是否处于展开状态。
    pub is_expanded: bool,
    /// 当前节点是否被文件树选中。
    pub is_selected: bool,
    /// 当前节点是否对应激活文件。
    pub is_active: bool,
    /// 子节点列表。
    pub children: Vec<FileTreeNode>,
}

/// 文件树展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeState {
    /// 面板标题。
    pub title: String,
    /// 根节点列表。
    pub roots: Vec<FileTreeNode>,
}

/// 标题栏使用的图标语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleBarIcon {
    /// 系统设置。
    Settings,
}

/// 标题栏展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleBarState {
    /// 标题栏右侧的工具入口。
    pub right_icons: Vec<TitleBarIcon>,
}

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

/// 桌面端根界面使用的应用状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAppState {
    /// 顶部标题栏状态。
    pub title_bar: TitleBarState,
    /// 底部工具栏信息。
    pub tool_bar: ToolBarState,
    /// 当前打开项目的名称。
    pub project_name: String,
    /// 当前激活文件。
    pub active_buffer: String,
    /// 打开的标签页。
    pub buffers: Vec<BufferSummary>,
    /// 左侧文件树内容。
    pub file_tree: FileTreeState,
    /// 主编辑区预览文本。
    pub editor_preview: Vec<String>,
}
