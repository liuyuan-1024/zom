//! 项目搜索面板视图。

use gpui::{App, Context, FocusHandle, Focusable, Render, Window};

use crate::components::panel::{placeholder, shell};

/// 项目搜索面板。
pub(crate) struct ProjectSearchPanel {
    focus_handle: FocusHandle,
}

impl ProjectSearchPanel {
    /// 创建项目搜索面板。
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for ProjectSearchPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ProjectSearchPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        shell::render_shell(
            "project-search-panel-container",
            &self.focus_handle,
            placeholder::render_title_only("Project Search"),
        )
    }
}
