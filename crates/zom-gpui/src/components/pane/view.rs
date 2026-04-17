use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement,
    Styled, Window, div, px, rgb,
};
use zom_app::state::PaneState;

use crate::{
    components::pane::tab_bar,
    theme::{color, size::GAP_3},
};

/// 查看器模式下的软换行阈值（按字符数近似）。
const SOFT_WRAP_MAX_CHARS: usize = 120;

pub struct PaneView {
    state: PaneState,
}

impl PaneView {
    pub fn new(state: PaneState) -> Self {
        Self { state }
    }

    /// 覆盖 Pane 状态，用于响应外部交互（例如文件树点击）。
    pub fn set_state(&mut self, state: PaneState, cx: &mut Context<Self>) {
        self.state = state;
        cx.notify();
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
        if let Some(active_tab) = self.state.active_tab() {
            return div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .child(self.render_viewer_content(&active_tab.buffer_lines))
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

    /// 渲染实际的文件内容查看器
    fn render_viewer_content(&self, buffer_lines: &[String]) -> impl IntoElement + '_ {
        let line_elements = buffer_lines
            .iter()
            .enumerate()
            .flat_map(|(line_index, line)| {
                wrap_visual_line(line, SOFT_WRAP_MAX_CHARS)
                    .into_iter()
                    .enumerate()
                    .map(move |(wrapped_index, wrapped_line)| {
                        let line_number = if wrapped_index == 0 {
                            (line_index + 1).to_string()
                        } else {
                            String::new()
                        };

                        div()
                            .w_full()
                            .flex()
                            .flex_row()
                            .gap(px(GAP_3))
                            // 顶部对齐：确保长文本软换行时，行号停留在第一行的高度
                            .items_start()
                            .child(
                                div()
                                    .w(px(40.0))
                                    .flex_shrink_0()
                                    .text_right()
                                    .text_sm()
                                    .text_color(rgb(color::COLOR_FG_MUTED))
                                    .child(line_number),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .w_full()
                                    .text_sm()
                                    .text_color(rgb(color::COLOR_FG_MUTED))
                                    .whitespace_normal()
                                    .child(wrapped_line),
                            )
                    })
            });

        div()
            // 建议：后续如果支持多 Tab，这里的 ID 应该加上当前 Tab 的唯一标识，防止切换文件时滚动条位置串位。
            .id("file-viewer-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .w_full()
            .bg(rgb(color::COLOR_BG_APP))
            .p(px(8.0))
            .overflow_scroll()
            .children(line_elements)
    }
}

/// 按字符数把长行拆成多个显示段，作为查看器模式的软换行。
fn wrap_visual_line(line: &str, max_chars_per_line: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }

    let chars = line.chars().collect::<Vec<_>>();
    chars
        .chunks(max_chars_per_line.max(1))
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}
