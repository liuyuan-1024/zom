//! Pane 主体视图渲染与内容展示逻辑。

use std::{
    ops::Range,
    time::{Duration, Instant},
};

use gpui::{
    Animation, AnimationExt, AnyElement, App, Context, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, ScrollHandle, StatefulInteractiveElement, Styled, Window,
    div, px, rgb,
};
use zom_protocol::{BufferId, Position, Selection};
use zom_runtime::{
    projection::wrap_visual_line,
    state::{ActiveEditorSnapshot, PaneState},
};

use crate::{
    components::pane::tab_bar,
    theme::{color, size},
};

/// 软换行最小字符阈值（避免极窄容器下切得过碎）。
const SOFT_WRAP_MIN_CHARS: usize = 16;
/// 等宽字体单字符宽度近似值（用于从像素宽度估算换行阈值）。
const APPROX_MONO_CHAR_WIDTH_PX: f32 = 8.0;
/// 行号栏最小位数（减少 1~9 行时抖动）。
const GUTTER_MIN_DIGITS: usize = 2;
/// 单个可视行的估算高度（用于虚拟滚动窗口计算）。
const VISUAL_ROW_HEIGHT_PX: f32 = size::FONT_MD;
/// 可视区外额外渲染的行数，减少滚动时空白闪烁。
const VIRTUAL_OVERSCAN_ROWS: usize = 12;
/// 细线光标宽度。
const CARET_WIDTH_PX: f32 = 1.5;
/// 细线光标高度。
const CARET_HEIGHT_PX: f32 = size::FONT_MD;
/// 光标闪烁周期。
const CARET_BLINK_DURATION_MS: u64 = 1_000;
/// 光标移动后，暂时禁止闪烁的时长。
const CARET_BLINK_PAUSE_AFTER_MOVE_MS: u64 = 500;

/// 中央编辑窗格视图，负责标签栏与当前内容区渲染。
pub struct PaneView {
    state: PaneState,
    active_editor: Option<ActiveEditorSnapshot>,
    cursor: Position,
    last_cursor_moved_at: Option<Instant>,
    pending_scroll_to_cursor: bool,
    viewer_layout_cache: Option<ViewerLayoutCache>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
}

/// 软换行后的静态布局缓存（仅随文档版本和换行宽度变化）。
struct ViewerLayoutCache {
    buffer_id: BufferId,
    doc_version: u64,
    wrap_chunk: usize,
    line_count: usize,
    line_char_lens: Vec<usize>,
    line_wrap_counts: Vec<usize>,
    line_start_rows: Vec<usize>,
    wrapped_rows: Vec<WrappedRow>,
}

/// 单个可视行的静态布局数据。
struct WrappedRow {
    row_id: gpui::SharedString,
    line_number: Option<usize>,
    line_index: usize,
    line_char_len: usize,
    segment_start_column: usize,
    segment_end_column: usize,
    is_last_segment: bool,
    wrapped_line: String,
}

impl PaneView {
    /// 用初始 Pane 状态构建视图实体。
    pub fn new(
        state: PaneState,
        active_editor: Option<ActiveEditorSnapshot>,
        cx: &mut Context<Self>,
    ) -> Self {
        let cursor = active_editor
            .as_ref()
            .map(|editor| editor.selection.active())
            .unwrap_or_else(Position::zero);
        Self {
            state,
            active_editor,
            cursor,
            last_cursor_moved_at: None,
            pending_scroll_to_cursor: true,
            viewer_layout_cache: None,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
        }
    }

    /// 覆盖 Pane 状态，用于响应外部交互（例如文件树激活）。
    pub fn set_state(
        &mut self,
        state: PaneState,
        active_editor: Option<ActiveEditorSnapshot>,
        cx: &mut Context<Self>,
    ) {
        let next_cursor = active_editor
            .as_ref()
            .map(|editor| editor.selection.active())
            .unwrap_or_else(Position::zero);
        if self.cursor != next_cursor {
            self.last_cursor_moved_at = Some(Instant::now());
            self.pending_scroll_to_cursor = true;
        }
        self.state = state;
        self.active_editor = active_editor;
        self.cursor = next_cursor;
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
        if let (Some(_), Some(active_editor)) =
            (self.state.active_tab(), self.active_editor.as_ref())
        {
            let viewport_width_px: f32 = window.viewport_size().width.into();
            let scroll_width_px: f32 = self.scroll_handle.bounds().size.width.into();
            let line_count = cached_line_count(self.viewer_layout_cache.as_ref(), active_editor)
                .unwrap_or_else(|| line_count_from_text(&active_editor.text));
            let gutter_width_px = gutter_width_for_line_count(line_count);
            let wrap_chunk =
                soft_wrap_max_chars(scroll_width_px, viewport_width_px, gutter_width_px).max(1);
            let mut scroll_to_row = None;

            if self.pending_scroll_to_cursor {
                let row_index = {
                    let cache = ensure_viewer_layout_cache(
                        &mut self.viewer_layout_cache,
                        active_editor,
                        wrap_chunk,
                    );
                    cursor_visual_row_index(cache, self.cursor)
                };
                scroll_to_row = Some(row_index);
                self.pending_scroll_to_cursor = false;
            }

            let layout_cache = ensure_viewer_layout_cache(
                &mut self.viewer_layout_cache,
                active_editor,
                wrap_chunk,
            );
            let selection = active_editor.selection;
            let scroll_handle = self.scroll_handle.clone();
            let editor_has_focus = self.focus_handle.is_focused(window);
            let cursor = self.cursor;
            let last_cursor_moved_at = self.last_cursor_moved_at;
            return div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .child(render_viewer_content(
                    &scroll_handle,
                    layout_cache,
                    gutter_width_px,
                    selection,
                    editor_has_focus,
                    cursor,
                    last_cursor_moved_at,
                    scroll_to_row,
                    cx,
                ))
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
}

/// 渲染实际的文件内容查看器
fn render_viewer_content(
    scroll_handle: &ScrollHandle,
    layout_cache: &ViewerLayoutCache,
    gutter_width_px: f32,
    selection: Selection,
    editor_has_focus: bool,
    cursor: Position,
    last_cursor_moved_at: Option<Instant>,
    scroll_to_row: Option<usize>,
    _cx: &mut Context<PaneView>,
) -> impl IntoElement {
    let suppress_caret_blink = last_cursor_moved_at.is_some_and(|moved_at| {
        moved_at.elapsed() < Duration::from_millis(CARET_BLINK_PAUSE_AFTER_MOVE_MS)
    });
    let cursor_line = usize::try_from(cursor.line).unwrap_or(usize::MAX);
    let cursor_column = usize::try_from(cursor.column).unwrap_or(usize::MAX);
    let total_rows = layout_cache.wrapped_rows.len();
    let scroll_offset_y_px = (-f32::from(scroll_handle.offset().y)).max(0.0);
    let viewport_height_px =
        f32::from(scroll_handle.bounds().size.height).max(VISUAL_ROW_HEIGHT_PX);
    let visible_range = virtual_row_window(
        total_rows,
        scroll_offset_y_px,
        viewport_height_px,
        VIRTUAL_OVERSCAN_ROWS,
        scroll_to_row,
    );
    if let Some(target_row) = scroll_to_row {
        if target_row >= visible_range.start && target_row < visible_range.end {
            let local_row_index = target_row - visible_range.start;
            // children 顺序固定为：上占位、可视行片段、下占位。
            scroll_handle.scroll_to_item(1 + local_row_index);
        }
    }

    let top_spacer_height_px = visible_range.start as f32 * VISUAL_ROW_HEIGHT_PX;
    let bottom_spacer_height_px =
        total_rows.saturating_sub(visible_range.end) as f32 * VISUAL_ROW_HEIGHT_PX;
    let mut children = Vec::with_capacity(visible_range.len() + 2);
    children.push(render_virtual_spacer(top_spacer_height_px));

    for row in &layout_cache.wrapped_rows[visible_range.clone()] {
        let line_selected_range =
            selected_column_range_for_line(selection, row.line_index, row.line_char_len);
        let selected_range = selected_range_in_wrapped_segment(
            line_selected_range.as_ref(),
            row.segment_start_column,
            row.segment_end_column,
        );
        let is_cursor_line = row.line_index == cursor_line;
        let show_cursor_line_emphasis = is_cursor_line && editor_has_focus;
        let caret_column = if editor_has_focus {
            caret_column_in_wrapped_segment(
                is_cursor_line,
                cursor_column,
                row.line_char_len,
                row.segment_start_column,
                row.segment_end_column,
                row.is_last_segment,
            )
        } else {
            None
        };
        children.push(
            div()
                .id(row.row_id.clone())
                .w_full()
                .flex()
                .flex_row()
                .flex_none()
                .gap(px(size::GAP_3))
                .items_center()
                .child(render_gutter_cell(row.line_number, gutter_width_px))
                .child(render_text_cell(
                    &row.wrapped_line,
                    selected_range,
                    caret_column,
                    suppress_caret_blink,
                    show_cursor_line_emphasis,
                ))
                .into_any_element(),
        );
    }
    children.push(render_virtual_spacer(bottom_spacer_height_px));

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
        .track_scroll(scroll_handle)
        .children(children)
}

fn cached_line_count(
    cache: Option<&ViewerLayoutCache>,
    active_editor: &ActiveEditorSnapshot,
) -> Option<usize> {
    let cache = cache?;
    layout_cache_matches(cache, active_editor, cache.wrap_chunk).then_some(cache.line_count)
}

fn ensure_viewer_layout_cache<'a>(
    cache: &'a mut Option<ViewerLayoutCache>,
    active_editor: &ActiveEditorSnapshot,
    wrap_chunk: usize,
) -> &'a ViewerLayoutCache {
    let wrap_chunk = wrap_chunk.max(1);
    let should_rebuild = match cache.as_ref() {
        Some(existing) => !layout_cache_matches(existing, active_editor, wrap_chunk),
        None => true,
    };
    if should_rebuild {
        *cache = Some(build_viewer_layout_cache(active_editor, wrap_chunk));
    }
    cache
        .as_ref()
        .expect("viewer layout cache should exist after ensure")
}

fn layout_cache_matches(
    cache: &ViewerLayoutCache,
    active_editor: &ActiveEditorSnapshot,
    wrap_chunk: usize,
) -> bool {
    cache.buffer_id == active_editor.buffer_id
        && cache.doc_version == active_editor.doc_version
        && cache.wrap_chunk == wrap_chunk.max(1)
}

fn build_viewer_layout_cache(
    active_editor: &ActiveEditorSnapshot,
    wrap_chunk: usize,
) -> ViewerLayoutCache {
    let wrap_chunk = wrap_chunk.max(1);
    let buffer_lines = split_lines_for_viewer(&active_editor.text);
    let line_count = buffer_lines.len();
    let mut line_char_lens = Vec::with_capacity(line_count);
    let mut line_wrap_counts = Vec::with_capacity(line_count);
    let mut line_start_rows = Vec::with_capacity(line_count);
    let mut wrapped_rows = Vec::new();

    for (line_index, line) in buffer_lines.iter().enumerate() {
        line_start_rows.push(wrapped_rows.len());
        let line_char_len = line.chars().count();
        line_char_lens.push(line_char_len);
        let wrapped_lines = wrap_visual_line(line, wrap_chunk);
        let wrapped_count = wrapped_lines.len().max(1);
        line_wrap_counts.push(wrapped_count);

        for (wrapped_index, wrapped_line) in wrapped_lines.into_iter().enumerate() {
            let segment_start_column = wrapped_index * wrap_chunk;
            let segment_end_column = segment_start_column + wrapped_line.chars().count();
            let is_last_segment = wrapped_index + 1 == wrapped_count;
            wrapped_rows.push(WrappedRow {
                row_id: gpui::SharedString::from(format!(
                    "viewer-row-{line_index}-{wrapped_index}"
                )),
                line_number: (wrapped_index == 0).then_some(line_index + 1),
                line_index,
                line_char_len,
                segment_start_column,
                segment_end_column,
                is_last_segment,
                wrapped_line,
            });
        }
    }

    ViewerLayoutCache {
        buffer_id: active_editor.buffer_id,
        doc_version: active_editor.doc_version,
        wrap_chunk,
        line_count,
        line_char_lens,
        line_wrap_counts,
        line_start_rows,
        wrapped_rows,
    }
}

fn split_lines_for_viewer(text: &str) -> Vec<String> {
    text.split('\n').map(|line| line.to_string()).collect()
}

fn line_count_from_text(text: &str) -> usize {
    text.bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        .saturating_add(1)
}

fn virtual_row_window(
    total_rows: usize,
    scroll_offset_y_px: f32,
    viewport_height_px: f32,
    overscan_rows: usize,
    force_row: Option<usize>,
) -> Range<usize> {
    if total_rows == 0 {
        return 0..0;
    }

    let row_height_px = VISUAL_ROW_HEIGHT_PX.max(1.0);
    let first_visible =
        ((scroll_offset_y_px.max(0.0) / row_height_px).floor() as usize).min(total_rows - 1);
    let viewport_rows = (viewport_height_px.max(row_height_px) / row_height_px)
        .ceil()
        .max(1.0) as usize;
    let start = first_visible
        .saturating_sub(overscan_rows)
        .min(total_rows.saturating_sub(1));
    let end = (first_visible + viewport_rows + overscan_rows).min(total_rows);
    let mut range = start..end.max(start + 1).min(total_rows);

    if let Some(target_row) = force_row {
        let target_row = target_row.min(total_rows.saturating_sub(1));
        if target_row < range.start || target_row >= range.end {
            let focus_start = target_row.saturating_sub(viewport_rows / 2 + overscan_rows);
            let focus_end = (focus_start + viewport_rows + overscan_rows * 2).min(total_rows);
            let normalized_start = focus_end
                .saturating_sub(viewport_rows + overscan_rows * 2)
                .min(focus_start);
            range = normalized_start..focus_end.max(normalized_start + 1).min(total_rows);
        }
    }

    range
}

fn render_virtual_spacer(height_px: f32) -> AnyElement {
    div()
        .w_full()
        .h(px(height_px.max(0.0)))
        .flex_none()
        .into_any_element()
}

fn render_gutter_cell(line_number: Option<usize>, gutter_width_px: f32) -> AnyElement {
    div()
        .w(px(gutter_width_px))
        .flex_shrink_0()
        .text_right()
        .text_sm()
        .line_height(px(size::FONT_MD))
        .text_color(rgb(color::COLOR_FG_MUTED))
        .child(line_number.map_or_else(String::new, |number| number.to_string()))
        .into_any_element()
}

fn render_text_cell(
    wrapped_line: &str,
    selected_range: Option<Range<usize>>,
    caret_column: Option<usize>,
    suppress_caret_blink: bool,
    is_cursor_line: bool,
) -> AnyElement {
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
            selected_range,
            caret_column,
            suppress_caret_blink,
        ))
        .into_any_element()
}

fn selected_column_range_for_line(
    selection: Selection,
    line_index: usize,
    line_char_len: usize,
) -> Option<Range<usize>> {
    if selection.is_caret() {
        return None;
    }

    let start = selection.start();
    let end = selection.end();
    let line = u32::try_from(line_index).unwrap_or(u32::MAX);
    if line < start.line || line > end.line {
        return None;
    }

    let mut from = if line == start.line {
        start.column as usize
    } else {
        0
    };
    let mut to = if line == end.line {
        end.column as usize
    } else {
        line_char_len
    };

    from = from.min(line_char_len);
    to = to.min(line_char_len);
    (from < to).then_some(from..to)
}

fn selected_range_in_wrapped_segment(
    selected_columns_in_line: Option<&Range<usize>>,
    segment_start_column: usize,
    segment_end_column: usize,
) -> Option<Range<usize>> {
    let selected_columns_in_line = selected_columns_in_line?;
    let from = selected_columns_in_line.start.max(segment_start_column);
    let to = selected_columns_in_line.end.min(segment_end_column);
    (from < to).then_some((from - segment_start_column)..(to - segment_start_column))
}

fn caret_column_in_wrapped_segment(
    is_cursor_line: bool,
    cursor_column: usize,
    line_char_len: usize,
    segment_start_column: usize,
    segment_end_column: usize,
    is_last_segment: bool,
) -> Option<usize> {
    if !is_cursor_line {
        return None;
    }
    let clamped_column = cursor_column.min(line_char_len);
    if clamped_column < segment_start_column || clamped_column > segment_end_column {
        return None;
    }
    if clamped_column == segment_end_column && !is_last_segment {
        return None;
    }
    Some(clamped_column - segment_start_column)
}

fn render_wrapped_line_content(
    wrapped_line: &str,
    selected_range: Option<Range<usize>>,
    caret_column: Option<usize>,
    suppress_caret_blink: bool,
) -> AnyElement {
    if selected_range.is_none() && caret_column.is_none() {
        return div()
            .flex()
            .items_center()
            .child(wrapped_line.to_string())
            .into_any_element();
    }

    let chars = wrapped_line.chars().collect::<Vec<_>>();
    let len = chars.len();
    let caret_column = caret_column.filter(|column| *column <= len);
    let selected_range = selected_range.unwrap_or(0..0);
    let mut children = Vec::new();
    let mut cursor = 0usize;

    while cursor < len {
        if caret_column == Some(cursor) {
            children.push(render_caret(suppress_caret_blink).into_any_element());
        }

        let is_selected = selected_range.start <= cursor && cursor < selected_range.end;
        let mut end = cursor + 1;
        while end < len {
            if caret_column == Some(end) {
                break;
            }
            let end_selected = selected_range.start <= end && end < selected_range.end;
            if end_selected != is_selected {
                break;
            }
            end += 1;
        }

        let text = chars[cursor..end].iter().collect::<String>();
        children.push(render_text_chunk(text, is_selected));
        cursor = end;
    }

    if caret_column == Some(len) {
        children.push(render_caret(suppress_caret_blink).into_any_element());
    }

    if children.is_empty() {
        if caret_column == Some(0) {
            children.push(render_caret(suppress_caret_blink).into_any_element());
        } else {
            children.push(render_text_chunk(String::new(), false));
        }
    }

    div()
        .flex()
        .items_center()
        .children(children)
        .into_any_element()
}

fn render_text_chunk(text: String, selected: bool) -> AnyElement {
    if selected {
        div()
            .bg(rgb(color::COLOR_BG_ACTIVE))
            .child(text)
            .into_any_element()
    } else {
        div().child(text).into_any_element()
    }
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

fn gutter_width_for_line_count(line_count: usize) -> f32 {
    let digits = line_count.max(1).to_string().len().max(GUTTER_MIN_DIGITS);
    let content_width_px = digits as f32 * APPROX_MONO_CHAR_WIDTH_PX;
    let horizontal_padding_px = size::GAP_1 * 2.0;
    (content_width_px + horizontal_padding_px).max(size::GUTTER_MD)
}

fn soft_wrap_max_chars(
    scroll_width_px: f32,
    viewport_width_px: f32,
    gutter_width_px: f32,
) -> usize {
    let width_px = if scroll_width_px > 1.0 {
        scroll_width_px
    } else {
        viewport_width_px
    };
    let content_width_px =
        (width_px - (size::PADDING_SM * 2.0) - gutter_width_px - size::GAP_3).max(1.0);
    ((content_width_px / APPROX_MONO_CHAR_WIDTH_PX).floor() as usize).max(SOFT_WRAP_MIN_CHARS)
}

fn cursor_visual_row_index(layout_cache: &ViewerLayoutCache, cursor: Position) -> usize {
    if layout_cache.line_count == 0 {
        return 0;
    }

    let mut cursor_line = usize::try_from(cursor.line).unwrap_or(usize::MAX);
    cursor_line = cursor_line.min(layout_cache.line_count.saturating_sub(1));
    let cursor_column = usize::try_from(cursor.column).unwrap_or(usize::MAX);
    let line_char_len = layout_cache
        .line_char_lens
        .get(cursor_line)
        .copied()
        .unwrap_or(0);
    let clamped_column = cursor_column.min(line_char_len);
    let wrap_count = layout_cache
        .line_wrap_counts
        .get(cursor_line)
        .copied()
        .unwrap_or(1)
        .max(1);
    let wrapped_index =
        (clamped_column / layout_cache.wrap_chunk.max(1)).min(wrap_count.saturating_sub(1));
    layout_cache
        .line_start_rows
        .get(cursor_line)
        .copied()
        .unwrap_or(0)
        + wrapped_index
}

#[cfg(test)]
mod tests {
    use super::{
        ViewerLayoutCache, cursor_visual_row_index, gutter_width_for_line_count,
        line_count_from_text, soft_wrap_max_chars, virtual_row_window,
    };
    use zom_protocol::{BufferId, Position};

    #[test]
    fn soft_wrap_max_chars_uses_scroll_width_when_available() {
        let chars = soft_wrap_max_chars(320.0, 1000.0, 40.0);
        assert!(chars < 60);
    }

    #[test]
    fn gutter_width_expands_for_large_line_counts() {
        let narrow = gutter_width_for_line_count(99);
        let wide = gutter_width_for_line_count(100_000);
        assert!(wide > narrow);
    }

    #[test]
    fn cursor_visual_row_index_counts_wrapped_rows_before_cursor_line() {
        let cache = cache_for_cursor_test(vec![8, 3], vec![2, 1], vec![0, 2], 4);
        let row_index = cursor_visual_row_index(&cache, Position::new(1, 1));
        assert_eq!(row_index, 2);
    }

    #[test]
    fn cursor_visual_row_index_clamps_out_of_range_cursor_line() {
        let cache = cache_for_cursor_test(vec![3], vec![1], vec![0], 4);
        let row_index = cursor_visual_row_index(&cache, Position::new(9, 0));
        assert_eq!(row_index, 0);
    }

    #[test]
    fn line_count_from_text_counts_trailing_newline() {
        assert_eq!(line_count_from_text("a\n"), 2);
    }

    #[test]
    fn virtual_row_window_limits_to_visible_slice_with_overscan() {
        let range = virtual_row_window(1_000, 160.0, 160.0, 4, None);
        assert_eq!(range, 6..24);
    }

    #[test]
    fn virtual_row_window_includes_forced_row() {
        let range = virtual_row_window(100, 0.0, 64.0, 2, Some(80));
        assert!(range.contains(&80));
    }

    fn cache_for_cursor_test(
        line_char_lens: Vec<usize>,
        line_wrap_counts: Vec<usize>,
        line_start_rows: Vec<usize>,
        wrap_chunk: usize,
    ) -> ViewerLayoutCache {
        ViewerLayoutCache {
            buffer_id: BufferId::new(1),
            doc_version: 0,
            wrap_chunk,
            line_count: line_char_lens.len(),
            line_char_lens,
            line_wrap_counts,
            line_start_rows,
            wrapped_rows: Vec::new(),
        }
    }
}
