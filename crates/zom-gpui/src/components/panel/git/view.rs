//! Git 面板视图。

//! Git 面板视图。

use gpui::{App, Context, FocusHandle, Focusable, Render, Window};

use crate::components::panel::{placeholder, shell};

/// Git 面板。
pub(crate) struct GitPanel {
    focus_handle: FocusHandle,
}

impl GitPanel {
    /// 创建 Git 面板。
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for GitPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for GitPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        shell::render_shell(
            "git-panel-container",
            &self.focus_handle,
            placeholder::render_title_only("Git"),
        )
    }
}
