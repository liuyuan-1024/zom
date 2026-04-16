/// 工作台领域的命令语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCommand {
    /// 将焦点切到文件树。
    FocusFileTree,
    /// 将焦点切到命令面板。
    FocusCommandPalette,
    /// 关闭当前激活标签页。
    CloseActiveTab,
    /// 激活前一个标签页。
    ActivatePrevTab,
    /// 激活下一个标签页。
    ActivateNextTab,
}
