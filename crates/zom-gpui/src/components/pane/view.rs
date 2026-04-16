use gpui::{Context, FontWeight, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use zom_app::state::PaneState;

use crate::{
    components::pane::tab_bar,
    spacing::{SPACE_1, SPACE_3, SPACE_4, SPACE_5},
};

pub struct PaneView {
    state: PaneState,
    editor_preview: Vec<String>,
}

impl PaneView {
    pub fn new(state: PaneState, editor_preview: Vec<String>) -> Self {
        Self {
            state,
            editor_preview,
        }
    }
}

impl PaneView {
    /// 渲染当前活动标签的内容（编辑区）
    fn render_active_content(&self) -> impl IntoElement {
        if let Some(active_index) = self.state.active_tab_index {
            if let Some(active_tab) = self.state.tabs.get(active_index) {
                return div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .px(px(SPACE_5))
                    .py(px(SPACE_4))
                    .gap(px(SPACE_3))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0x8090ab))
                            .child(active_tab.title.clone()), // 显示当前文件标题
                    )
                    .child(self.render_editor_preview_content())
                    .into_any_element();
            }
        }

        // 如果没有活跃标签，显示空占位
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(0x5c6880))
            .child("No Active Editor")
            .into_any_element()
    }

    /// 渲染文本内容
    fn render_editor_preview_content(&self) -> impl IntoElement {
        let line_elements = self.editor_preview.iter().enumerate().map(|(index, line)| {
            div()
                .w_full()
                .min_h(px(28.0))
                .flex()
                .flex_row()
                .gap(px(SPACE_3))
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
            .gap(px(SPACE_1))
            .p(px(SPACE_4))
            .bg(rgb(0x0d1117))
            .border_1()
            .border_color(rgb(0x232b38))
            .rounded_sm()
            .children(line_elements)
    }
}

impl Render for PaneView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .bg(rgb(0x10151d))
            .child(tab_bar::render(&self.state))
            .child(self.render_active_content())
    }
}
