/// 当前焦点所在的逻辑区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusTarget {
    /// 编辑器
    Editor,

    /// 命令
    Palette,
    /// 设置
    Setting,

    /// 文件树面板
    FileTreePanel,
    /// Git 面板
    GitPanel,
    /// 大纲面板
    OutlinePanel,
    /// 全局搜索窗格
    ProjectSearchPane,
    /// 语言服务器
    LanguageServers,
    /// 终端面板
    TerminalPanel,
    /// Debug 面板
    DebugPanel,
    /// 通知面板
    NotificationPanel,
}

impl FocusTarget {
    /// 所有焦点目标。
    pub const ALL: [Self; 10] = [
        Self::Editor,
        Self::Palette,
        Self::FileTreePanel,
        Self::GitPanel,
        Self::OutlinePanel,
        Self::ProjectSearchPane,
        Self::LanguageServers,
        Self::TerminalPanel,
        Self::DebugPanel,
        Self::NotificationPanel,
    ];

    /// 所有受工作台显隐策略管理的面板目标。
    pub const VISIBILITY_MANAGED_PANELS: [Self; 8] = [
        Self::FileTreePanel,
        Self::GitPanel,
        Self::OutlinePanel,
        Self::ProjectSearchPane,
        Self::LanguageServers,
        Self::TerminalPanel,
        Self::DebugPanel,
        Self::NotificationPanel,
    ];

    /// 判断当前目标是否属于“可显隐管理”的工作台面板。
    pub const fn is_visibility_managed_panel(self) -> bool {
        matches!(
            self,
            Self::FileTreePanel
                | Self::GitPanel
                | Self::OutlinePanel
                | Self::ProjectSearchPane
                | Self::LanguageServers
                | Self::TerminalPanel
                | Self::DebugPanel
                | Self::NotificationPanel
        )
    }

    /// 判断当前面板在应用启动时是否默认可见。
    pub const fn is_visible_by_default(self) -> bool {
        matches!(self, Self::FileTreePanel)
    }
}

#[cfg(test)]
mod tests {
    use super::FocusTarget;

    #[test]
    fn file_tree_is_visibility_managed_and_visible_by_default() {
        assert!(FocusTarget::FileTreePanel.is_visibility_managed_panel());
        assert!(FocusTarget::FileTreePanel.is_visible_by_default());
    }

    #[test]
    fn editor_is_not_visibility_managed_panel() {
        assert!(!FocusTarget::Editor.is_visibility_managed_panel());
        assert!(!FocusTarget::Editor.is_visible_by_default());
    }
}
