//! 通知中心相关命令规范声明。

use crate::command::kind::{CommandKind, CommandKindSpec};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::WorkspaceNotificationMarkSelectedRead,
        "workspace.notification.mark_selected_read",
        "通知标记当前已读",
        "将通知面板当前选中的通知标记为已读。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceNotificationMarkAllRead,
        "workspace.notification.mark_all_read",
        "通知标记全部已读",
        "将通知面板中的所有通知标记为已读。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceNotificationClearAll,
        "workspace.notification.clear_all",
        "通知清空全部",
        "清空通知面板中的全部通知。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceNotificationClearRead,
        "workspace.notification.clear_read",
        "通知清空已读",
        "清空通知面板中的已读通知。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceNotificationFocusUnreadError,
        "workspace.notification.focus_unread_error",
        "通知聚焦未读错误",
        "显示通知面板并聚焦到最近的未读错误通知。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceNotificationSelectPrev,
        "workspace.notification.select_prev",
        "通知选择上一条",
        "在通知面板中选择上一条通知。",
    ),
    CommandKindSpec::new(
        CommandKind::WorkspaceNotificationSelectNext,
        "workspace.notification.select_next",
        "通知选择下一条",
        "在通知面板中选择下一条通知。",
    ),
];
