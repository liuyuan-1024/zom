/// 当前焦点所在的逻辑区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusTarget {
    /// 编辑器
    Editor,
    /// 命令面板
    Palette,
    /// 文件树
    FileTreePanel,
    /// Git
    GitPanel,
    /// 大纲
    OutlinePanel,
    /// 全局搜索
    ProjectSearch,
    /// LSP
    LSP,
    /// 终端
    Terminal,
    /// Debug Panel
    DebugPanel,
    /// 通知
    Notification,
}
