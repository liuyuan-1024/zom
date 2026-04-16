//! `zom-gpui` 负责把应用状态渲染成桌面界面。
//! 当前阶段先提供一个最小可运行的编辑器壳子。

mod assets;
mod chrome;
mod components;
mod spacing;
use components::{file_tree::FileTreePanel, title_bar, tool_bar};

use gpui::{
    AnyView, App, Application, Bounds, Context, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use zom_app::state::DesktopAppState;

use crate::components::pane::PaneView;

/// 启动桌面界面。
pub fn run() {
    Application::new()
        .with_assets(assets::ZomAssets::new())
        .run(|cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(1280.), px(820.0)), cx);
            let state = DesktopAppState::sample();

            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Zom".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(chrome::traffic_light_position()),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                move |_, cx| cx.new(|cx| ZomRootView::new(state, cx)),
            )
            .unwrap();

            cx.activate(true);
        });
}

/// 根视图，负责拼装最外层界面布局。
pub struct ZomRootView {
    /// 用于展示的应用状态。
    state: DesktopAppState,
    /// 文件树
    file_tree_panel: AnyView,
    /// Pane 视图
    pane_view: AnyView,
}

impl ZomRootView {
    /// 用应用状态创建根视图。
    pub fn new(state: DesktopAppState, cx: &mut Context<Self>) -> Self {
        let file_tree_panel = cx
            .new(|_| FileTreePanel::new(state.file_tree.clone()))
            .into();

        let pane_view = cx
            .new(|_| PaneView::new(state.pane.clone(), state.editor_preview.clone()))
            .into();

        Self {
            state,
            file_tree_panel,
            pane_view,
        }
    }
}

impl Render for ZomRootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x111318))
            .text_color(rgb(0xe6edf7))
            .child(title_bar::render(&self.state))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    .child(self.file_tree_panel.clone())
                    .child(self.pane_view.clone()),
            )
            .child(tool_bar::render(&self.state))
    }
}
