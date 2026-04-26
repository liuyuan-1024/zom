//! 面板停靠位规则与目标映射。
//! 元数据单一来源位于 `zom-protocol::focus`，此模块仅做转发导出。

pub use zom_protocol::{PanelDock, dock_targets, panel_dock};

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
