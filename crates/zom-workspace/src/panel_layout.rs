//! 面板停靠位规则与目标映射。

use zom_protocol::FocusTarget;

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

/// 返回目标面板所属停靠区域。
pub fn panel_dock(target: FocusTarget) -> Option<PanelDock> {
    match target {
        FocusTarget::FileTreePanel
        | FocusTarget::GitPanel
        | FocusTarget::OutlinePanel
        | FocusTarget::ProjectSearchPanel
        | FocusTarget::LanguageServersPanel => Some(PanelDock::Left),
        FocusTarget::NotificationPanel => Some(PanelDock::Right),
        FocusTarget::TerminalPanel | FocusTarget::DebugPanel => Some(PanelDock::Bottom),
        _ => None,
    }
}

/// 返回指定停靠区域允许挂载的面板目标列表。
pub fn dock_targets(dock: PanelDock) -> &'static [FocusTarget] {
    match dock {
        PanelDock::Left => &[
            FocusTarget::FileTreePanel,
            FocusTarget::GitPanel,
            FocusTarget::OutlinePanel,
            FocusTarget::ProjectSearchPanel,
            FocusTarget::LanguageServersPanel,
        ],
        PanelDock::Right => &[FocusTarget::NotificationPanel],
        PanelDock::Bottom => &[FocusTarget::TerminalPanel, FocusTarget::DebugPanel],
    }
}

#[cfg(test)]
mod tests {
    use zom_protocol::FocusTarget;

    use super::{PanelDock, dock_targets, panel_dock};

    #[test]
    fn notification_maps_to_right_dock() {
        assert_eq!(
            panel_dock(FocusTarget::NotificationPanel),
            Some(PanelDock::Right)
        );
    }

    #[test]
    fn terminal_and_debug_map_to_bottom_dock() {
        assert_eq!(
            panel_dock(FocusTarget::TerminalPanel),
            Some(PanelDock::Bottom)
        );
        assert_eq!(panel_dock(FocusTarget::DebugPanel), Some(PanelDock::Bottom));
    }

    #[test]
    fn right_dock_only_accepts_notification() {
        assert_eq!(
            dock_targets(PanelDock::Right),
            &[FocusTarget::NotificationPanel]
        );
    }
}
