//! 语言服务器面板视图。

use gpui::{App, Context, FocusHandle, Focusable, Render, Window};

use crate::components::panel::{placeholder, shell};

/// 语言服务器面板。
pub(crate) struct LanguageServersPanel {
    focus_handle: FocusHandle,
}

impl LanguageServersPanel {
    /// 创建语言服务器面板。
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for LanguageServersPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LanguageServersPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        shell::render_shell(
            "language-servers-panel-container",
            &self.focus_handle,
            placeholder::render_title_only("Language Servers"),
        )
    }
}
