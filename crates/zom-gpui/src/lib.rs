//! `zom-gpui` 负责把应用状态渲染成桌面界面。
//! 当前阶段先提供一个最小可运行的编辑器壳子。

mod assets;
mod chrome;
mod components;
use components::{title_bar, tool_bar};

use gpui::{
    App, Application, Bounds, Context, FontWeight, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use zom_app::{BufferSummary, DesktopAppState, SidebarSection};

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
                move |_, cx| cx.new(|_| ZomRootView::new(state)),
            )
            .unwrap();

            cx.activate(true);
        });
}

/// 根视图，负责拼装最外层界面布局。
pub struct ZomRootView {
    /// 用于展示的应用状态。
    state: DesktopAppState,
}

impl ZomRootView {
    /// 用应用状态创建根视图。
    pub fn new(state: DesktopAppState) -> Self {
        Self { state }
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
                    .child(render_sidebar(&self.state.sidebar_sections))
                    .child(render_editor_surface(&self.state)),
            )
            .child(tool_bar::render(&self.state))
    }
}

/// 渲染左侧侧边栏。
fn render_sidebar(sections: &[SidebarSection]) -> impl IntoElement {
    let section_elements = sections.iter().map(render_sidebar_section);

    div()
        .w(px(260.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(rgb(0x0d1117))
        .border_r_1()
        .border_color(rgb(0x222938))
        .px_3()
        .py_3()
        .gap_4()
        .children(section_elements)
}

/// 渲染单个侧边栏分组。
fn render_sidebar_section(section: &SidebarSection) -> impl IntoElement {
    let item_elements = section.items.iter().map(|item| {
        div()
            .w_full()
            .h(px(28.0))
            .flex()
            .items_center()
            .px_2()
            .rounded_sm()
            .text_sm()
            .text_color(rgb(0xc8d1e5))
            .child(item.clone())
    });

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0x76839c))
                .child(section.title.clone()),
        )
        .children(item_elements)
}

/// 渲染标签栏和主编辑区。
fn render_editor_surface(state: &DesktopAppState) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .h_full()
        .bg(rgb(0x10151d))
        .child(render_tab_strip(&state.buffers))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .px_5()
                .py_4()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x8090ab))
                        .child(state.active_buffer.clone()),
                )
                .child(render_editor_preview(&state.editor_preview)),
        )
}

/// 渲染顶部标签栏。
fn render_tab_strip(buffers: &[BufferSummary]) -> impl IntoElement {
    let tabs = buffers.iter().map(render_tab);

    div()
        .w_full()
        .h(px(42.0))
        .flex()
        .flex_row()
        .items_end()
        .px_3()
        .bg(rgb(0x151b24))
        .border_b_1()
        .border_color(rgb(0x262d3a))
        .children(tabs)
}

/// 渲染单个标签页。
fn render_tab(buffer: &BufferSummary) -> impl IntoElement {
    let base = div()
        .h(px(36.0))
        .min_w(px(120.0))
        .px_3()
        .mr_2()
        .flex()
        .items_center()
        .rounded_t_sm()
        .border_1()
        .border_b_0()
        .text_sm()
        .child(buffer.title.clone());

    if buffer.is_active {
        base.bg(rgb(0x10151d))
            .border_color(rgb(0x2f88ff))
            .text_color(rgb(0xf3f6fb))
    } else {
        base.bg(rgb(0x1b2230))
            .border_color(rgb(0x2a3242))
            .text_color(rgb(0x8d9ab1))
    }
}

/// 渲染编辑区内的文本预览。
fn render_editor_preview(lines: &[String]) -> impl IntoElement {
    let line_elements = lines.iter().enumerate().map(|(index, line)| {
        div()
            .w_full()
            .min_h(px(28.0))
            .flex()
            .flex_row()
            .gap_4()
            .child(
                div()
                    .w(px(40.0))
                    .text_right()
                    .text_sm()
                    .text_color(rgb(0x5c6880))
                    .child((index + 1).to_string()),
            )
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(rgb(0xd9e2f2))
                    .child(line.clone()),
            )
    });

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .p_4()
        .bg(rgb(0x0d1117))
        .border_1()
        .border_color(rgb(0x232b38))
        .rounded_sm()
        .children(line_elements)
}
