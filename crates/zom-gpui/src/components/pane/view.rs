//! Pane 容器视图：只负责标签栏和内容路由，不承载文本渲染细节。

use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window, div, rgb,
};

use crate::{
    components::{
        editor::EditorView,
        pane::{content_router::PaneContentRouter, tab_bar},
    },
    root_view::store::AppStore,
    theme::color,
};

/// 主窗格视图，负责标签栏渲染与活动内容挂载。
pub struct PaneView {
    store: Entity<AppStore>,
    content_router: PaneContentRouter,
    focus_handle: FocusHandle,
}

impl PaneView {
    /// 创建窗格视图并订阅 store 变更，确保标签区与内容区同步刷新。
    /// 初始化时会创建默认编辑器子视图并持有焦点句柄。
    pub fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        cx.observe(&store, |_this, _, cx| {
            cx.notify();
        })
        .detach();
        let editor_view = cx.new(|cx| EditorView::new(store.clone(), cx));
        Self {
            store,
            content_router: PaneContentRouter::new(editor_view),
            focus_handle: cx.focus_handle(),
        }
    }

    /// 将编辑器视图设为当前焦点目标。
    pub(crate) fn focus_editor(&self, window: &mut Window, cx: &mut Context<Self>) {
        let pane = self.store.read(cx).select_pane_state();
        let view = self.content_router.view_for_active_tab(&pane);
        cx.focus_view(&view, window);
    }
}

impl Focusable for PaneView {
    /// 返回当前组件的焦点句柄，用于键盘焦点路由。
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PaneView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pane = self.store.read(cx).select_pane_state();
        let content = self.content_router.view_for_active_tab(&pane);

        div()
            .track_focus(&self.focus_handle)
            .tab_index(0)
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .bg(rgb(color::COLOR_BG_APP))
            .child(tab_bar::render(&pane))
            .child(content)
    }
}
