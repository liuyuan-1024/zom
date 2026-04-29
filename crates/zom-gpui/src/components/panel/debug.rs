//! Debug 面板视图。

use gpui::{App, Context, FocusHandle, Focusable, Render, Window};

use crate::components::panel::{placeholder, shell};

/// 调试面板。
pub(crate) struct DebugPanel {
    focus_handle: FocusHandle,
}

impl DebugPanel {
    /// 创建调试面板。
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for DebugPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DebugPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        shell::render_shell(
            "debug-panel-container",
            &self.focus_handle,
            placeholder::render_title_only("Debug"),
        )
    }
}
