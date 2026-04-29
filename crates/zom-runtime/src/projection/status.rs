//! 状态展示投影。

use zom_protocol::Position;

use crate::state::{DesktopAppState, DesktopNotificationLevel};

/// 将零基光标位置投影为用户可读的 `line:column`（一基）。
pub fn cursor_text(position: Position) -> String {
    format!("{}:{}", position.line + 1, position.column + 1)
}

/// 将通知状态投影为工具栏可展示的短文案。
pub fn notification_status_text(state: &DesktopAppState) -> Option<String> {
    // 常驻状态提示优先级高于未读计数，避免错误/进度信息被计数文案覆盖。
    if let Some(notification) = state.active_status_notification.as_ref() {
        let level = match notification.level {
            DesktopNotificationLevel::Info => "INFO",
            DesktopNotificationLevel::Warning => "WARN",
            DesktopNotificationLevel::Error => "ERROR",
        };
        return Some(format!("{level}: {}", notification.message));
    }

    if state.unread_notification_count > 0 {
        return Some(format!("NOTI {}", state.unread_notification_count));
    }

    None
}

#[cfg(test)]
mod tests {
    use zom_protocol::FocusTarget;
    use zom_protocol::Position;

    use super::{cursor_text, notification_status_text};
    use crate::state::DesktopAppState;
    use crate::state::{DesktopNotification, DesktopNotificationLevel, DesktopNotificationSource};

    #[test]
    /// 光标文案使用从 1 开始的行列显示。
    fn cursor_text_uses_one_based_display() {
        assert_eq!(cursor_text(Position::new(0, 0)), "1:1");
        assert_eq!(cursor_text(Position::new(9, 3)), "10:4");
    }

    #[test]
    /// 计算状态栏通知结果。
    fn notification_status_prefers_active_status_notification() {
        let mut state = DesktopAppState::from_current_workspace();
        state.unread_notification_count = 5;
        state.active_status_notification = Some(DesktopNotification {
            id: 7,
            level: DesktopNotificationLevel::Error,
            source: DesktopNotificationSource::System,
            message: "task failed".into(),
            created_at_ms: 1,
            updated_at_ms: 1,
            is_read: false,
            dedupe_key: None,
            occurrence_count: 1,
        });

        assert_eq!(
            notification_status_text(&state),
            Some("ERROR: task failed".to_string())
        );
    }

    #[test]
    /// 计算状态栏结果。
    fn notification_status_uses_unread_count_when_no_status_item() {
        let mut state = DesktopAppState::from_current_workspace();
        state.focused_target = FocusTarget::Editor;
        state.active_status_notification = None;
        state.unread_notification_count = 3;

        assert_eq!(notification_status_text(&state), Some("NOTI 3".to_string()));
    }
}
