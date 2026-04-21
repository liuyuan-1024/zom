//! Pane 主体视图渲染与内容展示逻辑。

use std::time::{Duration, Instant};

use gpui::{
    Animation, AnimationExt, AnyElement, App, Context, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled, Window,
    div, px, rgb,
};
use zom_protocol::Position;
use zom_runtime::{projection::wrap_visual_line, state::PaneState};

use crate::{
    components::pane::tab_bar,
    theme::{color, size},
};

/// 软换行最小字符阈值（避免极窄容器下切得过碎）。
const SOFT_WRAP_MIN_CHARS: usize = 16;
/// 等宽字体单字符宽度近似值（用于从像素宽度估算换行阈值）。
const APPROX_MONO_CHAR_WIDTH_PX: f32 = 8.0;
/// 细线光标宽度。
const CARET_WIDTH_PX: f32 = 1.5;
/// 细线光标高度。
const CARET_HEIGHT_PX: f32 = size::FONT_MD;
/// 光标在文本中的内部标记字符（私有使用区）。
const CARET_MARKER: char = '\u{E000}';
/// 光标闪烁周期。
const CARET_BLINK_DURATION_MS: u64 = 1_000;
/// 光标移动后，暂时禁止闪烁的时长。
const CARET_BLINK_PAUSE_AFTER_MOVE_MS: u64 = 500;

/// 中央编辑窗格视图，负责标签栏与当前内容区渲染。
pub struct PaneView {
    state: PaneState,
    cursor: Position,
    last_cursor_moved_at: Option<Instant>,
    pending_scroll_to_cursor: bool,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
}

impl PaneView {
    /// 用初始 Pane 状态构建视图实体。
    pub fn new(state: PaneState, cursor: Position, cx: &mut Context<Self>) -> Self {
        Self {
            state,
            cursor,
            last_cursor_moved_at: None,
            pending_scroll_to_cursor: true,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
        }
    }

    /// 覆盖 Pane 状态，用于响应外部交互（例如文件树激活）。
    pub fn set_state(&mut self, state: PaneState, cursor: Position, cx: &mut Context<Self>) {
        if self.cursor != cursor {
            self.last_cursor_moved_at = Some(Instant::now());
            self.pending_scroll_to_cursor = true;
        }
        self.state = state;
        self.cursor = cursor;
        cx.notify();
    }
}

impl Focusable for PaneView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PaneView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .tab_index(0)
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .bg(rgb(color::COLOR_BG_APP))
            .child(tab_bar::render(&self.state))
            .child(self.render_active_content(window, cx))
    }
}

impl PaneView {
    /// 渲染当前活动标签的内容（编辑区）
    fn render_active_content(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        if let Some(active_tab) = self.state.active_tab() {
            let buffer_lines = active_tab.buffer_lines();
            let viewport_width_px: f32 = window.viewport_size().width.into();
            let scroll_width_px: f32 = self.scroll_handle.bounds().size.width.into();
            let wrap_max_chars = soft_wrap_max_chars(scroll_width_px, viewport_width_px);

            if self.pending_scroll_to_cursor {
                let row_index = cursor_visual_row_index(&buffer_lines, self.cursor, wrap_max_chars);
                self.scroll_handle.scroll_to_item(row_index);
                self.pending_scroll_to_cursor = false;
            }
            return div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .child(self.render_viewer_content(buffer_lines, wrap_max_chars, cx))
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
    fn render_viewer_content(
        &self,
        buffer_lines: Vec<String>,
        wrap_max_chars: usize,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement + '_ {
        let suppress_caret_blink = self.last_cursor_moved_at.is_some_and(|moved_at| {
            moved_at.elapsed() < Duration::from_millis(CARET_BLINK_PAUSE_AFTER_MOVE_MS)
        });
        let cursor_line = usize::try_from(self.cursor.line).unwrap_or(usize::MAX);
        let cursor_column = usize::try_from(self.cursor.column).unwrap_or(usize::MAX);
        let line_elements = buffer_lines
            .iter()
            .enumerate()
            .flat_map(|(line_index, line)| {
                let is_cursor_line = line_index == cursor_line;
                let rendered_line = if is_cursor_line {
                    line_with_caret_marker(line, cursor_column)
                } else {
                    line.clone()
                };
                wrap_visual_line(&rendered_line, wrap_max_chars)
                    .into_iter()
                    .enumerate()
                    .map(move |(wrapped_index, wrapped_line)| {
                        let line_number = if wrapped_index == 0 {
                            (line_index + 1).to_string()
                        } else {
                            String::new()
                        };
                        let row_id = gpui::SharedString::from(format!(
                            "viewer-row-{line_index}-{wrapped_index}"
                        ));

                        div()
                            .id(row_id)
                            .w_full()
                            .flex()
                            .flex_row()
                            .flex_none()
                            .gap(px(size::GAP_3))
                            .items_center()
                            .child(
                                div()
                                    .w(px(size::GUTTER_MD))
                                    .flex_shrink_0()
                                    .text_right()
                                    .text_sm()
                                    .line_height(px(size::FONT_MD))
                                    .text_color(rgb(color::COLOR_FG_MUTED))
                                    .child(line_number),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .w_full()
                                    .text_sm()
                                    .line_height(px(size::FONT_MD))
                                    .text_color(rgb(if is_cursor_line {
                                        color::COLOR_FG_PRIMARY
                                    } else {
                                        color::COLOR_FG_MUTED
                                    }))
                                    .whitespace_nowrap()
                                    .child(render_wrapped_line_content(
                                        wrapped_line,
                                        suppress_caret_blink,
                                    )),
                            )
                    })
            });

        div()
            // 建议：后续如果支持多 Tab，这里的 ID 应该加上当前 Tab 的唯一标识，防止切换文件时滚动条位置串位。
            .id("file-viewer-scroll")
            .w_full()
            .flex_1()
            .flex()
            .flex_col()
            .bg(rgb(color::COLOR_BG_APP))
            .p(px(size::PADDING_SM))
            .overflow_scroll()
            .track_scroll(&self.scroll_handle)
            .children(line_elements)
    }
}

fn line_with_caret_marker(line: &str, column: usize) -> String {
    let char_len = line.chars().count();
    let target = column.min(char_len);
    let mut rendered = String::with_capacity(line.len() + 1);

    for (index, ch) in line.chars().enumerate() {
        if index == target {
            rendered.push(CARET_MARKER);
        }
        rendered.push(ch);
    }

    if target == char_len {
        rendered.push(CARET_MARKER);
    }

    rendered
}

fn render_wrapped_line_content(wrapped_line: String, suppress_caret_blink: bool) -> AnyElement {
    if let Some((before, after)) = wrapped_line.split_once(CARET_MARKER) {
        return div()
            .flex()
            .items_center()
            .child(before.to_string())
            .child(render_caret(suppress_caret_blink))
            .child(after.to_string())
            .into_any_element();
    }

    div()
        .flex()
        .items_center()
        .child(wrapped_line)
        .into_any_element()
}

fn render_caret(suppress_caret_blink: bool) -> impl IntoElement {
    div()
        .w(px(CARET_WIDTH_PX))
        .h(px(CARET_HEIGHT_PX))
        .flex_shrink_0()
        .bg(rgb(color::COLOR_FG_PRIMARY))
        .with_animation(
            "pane-caret-blink",
            Animation::new(Duration::from_millis(CARET_BLINK_DURATION_MS)).repeat(),
            move |this, delta| {
                let opacity = if suppress_caret_blink {
                    1.0
                } else if delta < 0.5 {
                    1.0
                } else {
                    0.0
                };
                this.opacity(opacity)
            },
        )
}

fn soft_wrap_max_chars(scroll_width_px: f32, viewport_width_px: f32) -> usize {
    let width_px = if scroll_width_px > 1.0 {
        scroll_width_px
    } else {
        viewport_width_px
    };
    let content_width_px =
        (width_px - (size::PADDING_SM * 2.0) - size::GUTTER_MD - size::GAP_3).max(1.0);
    ((content_width_px / APPROX_MONO_CHAR_WIDTH_PX).floor() as usize).max(SOFT_WRAP_MIN_CHARS)
}

fn cursor_visual_row_index(
    buffer_lines: &[String],
    cursor: Position,
    wrap_max_chars: usize,
) -> usize {
    if buffer_lines.is_empty() {
        return 0;
    }

    let mut cursor_line = usize::try_from(cursor.line).unwrap_or(usize::MAX);
    cursor_line = cursor_line.min(buffer_lines.len() - 1);
    let cursor_column = usize::try_from(cursor.column).unwrap_or(usize::MAX);

    let mut visual_row = 0usize;
    for (line_index, line) in buffer_lines.iter().enumerate() {
        if line_index < cursor_line {
            visual_row += wrap_visual_line(line, wrap_max_chars).len().max(1);
            continue;
        }

        let rendered_line = line_with_caret_marker(line, cursor_column);
        for (wrapped_index, wrapped_line) in wrap_visual_line(&rendered_line, wrap_max_chars)
            .into_iter()
            .enumerate()
        {
            if wrapped_line.contains(CARET_MARKER) {
                return visual_row + wrapped_index;
            }
        }
        return visual_row;
    }

    visual_row
}

#[cfg(test)]
mod tests {
    use super::{cursor_visual_row_index, soft_wrap_max_chars};
    use zom_protocol::Position;

    #[test]
    fn soft_wrap_max_chars_uses_scroll_width_when_available() {
        let chars = soft_wrap_max_chars(320.0, 1000.0);
        assert!(chars < 60);
    }

    #[test]
    fn cursor_visual_row_index_counts_wrapped_rows_before_cursor_line() {
        let lines = vec!["abcdefgh".to_string(), "xyz".to_string()];
        let row_index = cursor_visual_row_index(&lines, Position::new(1, 1), 4);
        assert_eq!(row_index, 2);
    }

    #[test]
    fn cursor_visual_row_index_clamps_out_of_range_cursor_line() {
        let lines = vec!["abc".to_string()];
        let row_index = cursor_visual_row_index(&lines, Position::new(9, 0), 4);
        assert_eq!(row_index, 0);
    }
}
