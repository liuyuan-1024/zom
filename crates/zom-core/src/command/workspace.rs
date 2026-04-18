use crate::FocusTarget;

/// 工作台领域的命令语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCommand {
    /// 聚焦到指定面板
    FocusPanel(FocusTarget),
    /// 显示或隐藏指定面板。
    TogglePanel(FocusTarget),

    /// 作用于文件树的命令。
    FileTree(FileTreeCommand),
    /// 作用于标签页的命令。
    Tab(TabCommand),
}

/// 标签页命令语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabCommand {
    /// 关闭当前激活标签页。
    CloseActiveTab,
    /// 激活前一个标签页。
    ActivatePrevTab,
    /// 激活下一个标签页。
    ActivateNextTab,
}

/// 文件树命令语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTreeCommand {
    /// 选中上一条可见节点。
    SelectPrev,
    /// 选中下一条可见节点。
    SelectNext,
    /// 目录展开，或进入已展开目录的第一个子节点。
    ExpandOrDescend,
    /// 目录折叠，或回到父节点。
    CollapseOrAscend,
    /// 激活当前选中节点（文件打开，目录切换展开态）。
    ActivateSelection,
}
