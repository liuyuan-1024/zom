use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use zom_app::state::PaneState;

use crate::{
    components::pane::tab_bar,
    theme::{color, size::SPACE_3},
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

impl Render for PaneView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .bg(rgb(color::COLOR_BG_APP))
            .child(tab_bar::render(&self.state))
            .child(self.render_active_content())
    }
}

impl PaneView {
    /// 渲染当前活动标签的内容（编辑区）
    fn render_active_content(&self) -> impl IntoElement {
        if let Some(active_index) = self.state.active_tab_index {
            return div()
                .flex()
                .flex_col()
                .flex_1()
                .child(self.render_editor_preview_content())
                .into_any_element();
        }

        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(color::COLOR_FG_MUTED))
            .child("No Active Editor")
            .into_any_element()
    }

    /// 渲染文本内容
    fn render_editor_preview_content(&self) -> impl IntoElement {
        let line_elements = self.editor_preview.iter().enumerate().map(|(index, line)| {
            div()
                .w_full()
                .flex()
                .flex_row()
                .gap(px(SPACE_3))
                .child(
                    div()
                        .w(px(40.0))
                        .text_right()
                        .text_sm()
                        .text_color(rgb(color::COLOR_FG_MUTED))
                        .child((index + 1).to_string()),
                )
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(rgb(color::COLOR_FG_MUTED))
                        .child(line.clone()),
                )
        });

        div()
            .flex()
            .flex_col()
            .flex_1()
            .bg(rgb(color::COLOR_BG_APP))
            .children(line_elements)
    }
}
