//! 终端面板视图。

//! 终端面板视图。

use gpui::{App, Context, FocusHandle, Focusable, Render, Window};

use crate::components::panel::{placeholder, shell};

/// 终端面板。
pub(crate) struct TerminalPanel {
    focus_handle: FocusHandle,
}

impl TerminalPanel {
    /// 创建终端面板。
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for TerminalPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        shell::render_shell(
            "terminal-panel-container",
            &self.focus_handle,
            placeholder::render_title_only("Terminal"),
        )
    }
}
