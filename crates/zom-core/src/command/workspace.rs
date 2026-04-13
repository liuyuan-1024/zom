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
