//! 焦点目标与悬浮层目标语义定义。

/// 工作台面板停靠区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelDock {
    /// 左侧面板列。
    Left,
    /// 右侧面板列。
    Right,
    /// 中央列底部面板区域。
    Bottom,
}

/// 工具栏入口分组。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolBarSide {
    /// 工具栏左侧入口组。
    Left,
    /// 工具栏右侧入口组。
    Right,
}

/// 当前焦点所在的逻辑区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusTarget {
    /// 编辑器
    Editor,

    /// 命令
    Palette,
    /// 设置悬浮层
    SettingsOverlay,
    /// 查找替换悬浮层
    FindReplaceOverlay,

    /// 文件树面板
    FileTreePanel,
    /// Git 面板
    GitPanel,
    /// 大纲面板
    OutlinePanel,
    /// 全局搜索面板
    ProjectSearchPanel,
    /// 语言服务器面板
    LanguageServersPanel,
    /// 终端面板
    TerminalPanel,
    /// Debug 面板
    DebugPanel,
    /// 通知面板
    NotificationPanel,
}

impl FocusTarget {
    /// 所有焦点目标。
    pub const ALL: [Self; 12] = [
        Self::Editor,
        Self::Palette,
        Self::SettingsOverlay,
        Self::FindReplaceOverlay,
        Self::FileTreePanel,
        Self::GitPanel,
        Self::OutlinePanel,
        Self::ProjectSearchPanel,
        Self::LanguageServersPanel,
        Self::TerminalPanel,
        Self::DebugPanel,
        Self::NotificationPanel,
    ];

    /// 所有受工作台显隐策略管理的面板目标。
    pub const VISIBILITY_MANAGED_PANELS: [Self; 8] = [
        Self::FileTreePanel,
        Self::GitPanel,
        Self::OutlinePanel,
        Self::ProjectSearchPanel,
        Self::LanguageServersPanel,
        Self::TerminalPanel,
        Self::DebugPanel,
        Self::NotificationPanel,
    ];

    /// 左侧停靠区可挂载面板目标。
    pub const LEFT_DOCK_PANELS: [Self; 5] = [
        Self::FileTreePanel,
        Self::GitPanel,
        Self::OutlinePanel,
        Self::ProjectSearchPanel,
        Self::LanguageServersPanel,
    ];

    /// 右侧停靠区可挂载面板目标。
    pub const RIGHT_DOCK_PANELS: [Self; 1] = [Self::NotificationPanel];

    /// 底部停靠区可挂载面板目标。
    pub const BOTTOM_DOCK_PANELS: [Self; 2] = [Self::TerminalPanel, Self::DebugPanel];

    /// 判断当前目标是否属于“可显隐管理”的工作台面板。
    pub const fn is_visibility_managed_panel(self) -> bool {
        matches!(
            self,
            Self::FileTreePanel
                | Self::GitPanel
                | Self::OutlinePanel
                | Self::ProjectSearchPanel
                | Self::LanguageServersPanel
                | Self::TerminalPanel
                | Self::DebugPanel
                | Self::NotificationPanel
        )
    }

    /// 判断当前目标是否属于悬浮层焦点。
    pub const fn is_overlay(self) -> bool {
        matches!(self, Self::SettingsOverlay | Self::FindReplaceOverlay)
    }

    /// 判断当前面板在应用启动时是否默认可见。
    pub const fn is_visible_by_default(self) -> bool {
        matches!(self, Self::FileTreePanel)
    }

    /// 返回目标面板所属停靠区域。
    pub const fn panel_dock(self) -> Option<PanelDock> {
        match self {
            Self::FileTreePanel
            | Self::GitPanel
            | Self::OutlinePanel
            | Self::ProjectSearchPanel
            | Self::LanguageServersPanel => Some(PanelDock::Left),
            Self::NotificationPanel => Some(PanelDock::Right),
            Self::TerminalPanel | Self::DebugPanel => Some(PanelDock::Bottom),
            _ => None,
        }
    }

    /// 返回面板命令在工具栏中的分组。
    pub const fn tool_bar_side(self) -> Option<ToolBarSide> {
        match self {
            Self::FileTreePanel
            | Self::GitPanel
            | Self::OutlinePanel
            | Self::ProjectSearchPanel
            | Self::LanguageServersPanel => Some(ToolBarSide::Left),
            Self::TerminalPanel | Self::DebugPanel | Self::NotificationPanel => {
                Some(ToolBarSide::Right)
            }
            _ => None,
        }
    }
}

/// 返回目标面板所属停靠区域。
pub const fn panel_dock(target: FocusTarget) -> Option<PanelDock> {
    target.panel_dock()
}

/// 返回指定停靠区域允许挂载的面板目标列表。
pub fn dock_targets(dock: PanelDock) -> &'static [FocusTarget] {
    match dock {
        PanelDock::Left => &FocusTarget::LEFT_DOCK_PANELS,
        PanelDock::Right => &FocusTarget::RIGHT_DOCK_PANELS,
        PanelDock::Bottom => &FocusTarget::BOTTOM_DOCK_PANELS,
    }
}

/// 工作台悬浮层身份。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverlayTarget {
    /// 设置悬浮层。
    Settings,
    /// 查找替换悬浮层。
    FindReplace,
}

impl From<OverlayTarget> for FocusTarget {
    fn from(target: OverlayTarget) -> Self {
        match target {
            OverlayTarget::Settings => FocusTarget::SettingsOverlay,
            OverlayTarget::FindReplace => FocusTarget::FindReplaceOverlay,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FocusTarget, PanelDock, ToolBarSide, dock_targets, panel_dock};

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

    #[test]
    fn panel_dock_mapping_matches_catalog() {
        assert_eq!(
            panel_dock(FocusTarget::NotificationPanel),
            Some(PanelDock::Right)
        );
        assert_eq!(
            panel_dock(FocusTarget::TerminalPanel),
            Some(PanelDock::Bottom)
        );
    }

    #[test]
    fn dock_targets_use_single_source_catalog() {
        assert_eq!(
            dock_targets(PanelDock::Right),
            &FocusTarget::RIGHT_DOCK_PANELS
        );
        assert_eq!(
            dock_targets(PanelDock::Bottom),
            &FocusTarget::BOTTOM_DOCK_PANELS
        );
    }

    #[test]
    fn toolbar_side_groups_panel_commands() {
        assert_eq!(
            FocusTarget::FileTreePanel.tool_bar_side(),
            Some(ToolBarSide::Left)
        );
        assert_eq!(
            FocusTarget::NotificationPanel.tool_bar_side(),
            Some(ToolBarSide::Right)
        );
        assert_eq!(FocusTarget::Editor.tool_bar_side(), None);
    }
}
