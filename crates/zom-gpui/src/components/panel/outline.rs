//! 大纲面板视图。

use gpui::{App, Context, FocusHandle, Focusable, Render, Window};

use crate::components::panel::{placeholder, shell};

/// 大纲面板。
pub(crate) struct OutlinePanel {
    focus_handle: FocusHandle,
}

impl OutlinePanel {
    /// 创建大纲面板。
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for OutlinePanel {
    /// 返回当前组件的焦点句柄，用于键盘焦点路由。
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for OutlinePanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        shell::render_shell(
            "outline-panel-container",
            &self.focus_handle,
            placeholder::render_title_only("Outline"),
        )
    }
}
