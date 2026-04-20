//! 通知面板视图。

//! 通知面板视图。

use gpui::{App, Context, FocusHandle, Focusable, Render, Window};

use crate::components::panel::{placeholder, shell};

/// 通知面板。
pub(crate) struct NotificationPanel {
    focus_handle: FocusHandle,
}

impl NotificationPanel {
    /// 创建通知面板。
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for NotificationPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotificationPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        shell::render_shell(
            "notification-panel-container",
            &self.focus_handle,
            placeholder::render_title_only("Notifications"),
        )
    }
}
