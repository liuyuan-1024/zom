//! 纯文本编辑器视图：负责渲染装配与状态桥接。

use std::time::Instant;

use gpui::{
    App, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement,
    Render, ScrollHandle, StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use zom_protocol::{Position, Selection};
use zom_runtime::state::ActiveEditorSnapshot;

use crate::{
    root_view::store::AppStore,
    theme::{color, size},
};

use super::{
    caret::CARET_BLINK_PAUSE_AFTER_MOVE_MS,
    layout_cache::{
        ViewerLayoutCache, cached_line_count, cursor_visual_row_index, ensure_viewer_layout_cache,
        gutter_width_for_line_count, line_count_from_text, soft_wrap_max_chars,
    },
    selection_paint::{
        caret_column_in_wrapped_segment, render_gutter_cell, render_text_cell,
        selected_column_range_for_line, selected_range_in_wrapped_segment,
    },
    virtual_window::{
        VIRTUAL_OVERSCAN_ROWS, VISUAL_ROW_HEIGHT_PX, render_virtual_spacer, virtual_row_window,
    },
};

pub(crate) struct EditorView {
    store: Entity<AppStore>,
    cursor: Position,
    last_cursor_moved_at: Option<Instant>,
    should_scroll_to_cursor: bool,
    viewer_layout_cache: Option<ViewerLayoutCache>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
}

struct ViewerRenderSpec<'a> {
    scroll_handle: &'a ScrollHandle,
    layout_cache: &'a ViewerLayoutCache,
    gutter_width_px: f32,
    selection: Selection,
    is_editor_focused: bool,
    cursor: Position,
    last_cursor_moved_at: Option<Instant>,
    scroll_to_row: Option<usize>,
}

impl EditorView {
    pub(crate) fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        let cursor = store
            .read(cx)
            .select_active_editor_snapshot()
            .as_ref()
            .map(|editor| editor.selection.active())
            .unwrap_or_else(Position::zero);

        cx.observe(&store, |this, store, cx| {
            let next_cursor = store
                .read(cx)
                .select_active_editor_snapshot()
                .as_ref()
                .map(|editor| editor.selection.active())
                .unwrap_or_else(Position::zero);
            if this.cursor != next_cursor {
                this.last_cursor_moved_at = Some(Instant::now());
                this.should_scroll_to_cursor = true;
            }
            this.cursor = next_cursor;
            cx.notify();
        })
        .detach();

        Self {
            store,
            cursor,
            last_cursor_moved_at: None,
            should_scroll_to_cursor: true,
            viewer_layout_cache: None,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
        }
    }

    fn active_editor_snapshot(&self, cx: &App) -> Option<ActiveEditorSnapshot> {
        self.store.read(cx).select_active_editor_snapshot()
    }

    fn render_active_content(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some(active_editor) = self.active_editor_snapshot(cx) else {
            return div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(color::COLOR_FG_MUTED))
                .child("No Active Editor")
                .into_any_element();
        };

        let viewport_width_px: f32 = window.viewport_size().width.into();
        let scroll_width_px: f32 = self.scroll_handle.bounds().size.width.into();
        let line_count = cached_line_count(self.viewer_layout_cache.as_ref(), &active_editor)
            .unwrap_or_else(|| line_count_from_text(&active_editor.text));
        let gutter_width_px = gutter_width_for_line_count(line_count);
        let wrap_chunk =
            soft_wrap_max_chars(scroll_width_px, viewport_width_px, gutter_width_px).max(1);
        let mut scroll_to_row = None;

        if self.should_scroll_to_cursor {
            let row_index = {
                let cache = ensure_viewer_layout_cache(
                    &mut self.viewer_layout_cache,
                    &active_editor,
                    wrap_chunk,
                );
                cursor_visual_row_index(cache, self.cursor)
            };
            scroll_to_row = Some(row_index);
            self.should_scroll_to_cursor = false;
        }

        let layout_cache =
            ensure_viewer_layout_cache(&mut self.viewer_layout_cache, &active_editor, wrap_chunk);
        let selection = active_editor.selection;
        let scroll_handle = self.scroll_handle.clone();
        let is_editor_focused = self.focus_handle.is_focused(window);
        let cursor = self.cursor;
        let last_cursor_moved_at = self.last_cursor_moved_at;
        let render_spec = ViewerRenderSpec {
            scroll_handle: &scroll_handle,
            layout_cache,
            gutter_width_px,
            selection,
            is_editor_focused,
            cursor,
            last_cursor_moved_at,
            scroll_to_row,
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .child(render_viewer_content(render_spec))
            .into_any_element()
    }
}

impl Focusable for EditorView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .tab_index(0)
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .bg(rgb(color::COLOR_BG_APP))
            .child(self.render_active_content(window, cx))
    }
}

fn render_viewer_content(spec: ViewerRenderSpec<'_>) -> impl IntoElement {
    let ViewerRenderSpec {
        scroll_handle,
        layout_cache,
        gutter_width_px,
        selection,
        is_editor_focused,
        cursor,
        last_cursor_moved_at,
        scroll_to_row,
    } = spec;

    let should_suppress_caret_blink = last_cursor_moved_at.is_some_and(|moved_at| {
        moved_at.elapsed() < std::time::Duration::from_millis(CARET_BLINK_PAUSE_AFTER_MOVE_MS)
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
    if let Some(target_row) = scroll_to_row
        && target_row >= visible_range.start
        && target_row < visible_range.end
    {
        let local_row_index = target_row - visible_range.start;
        scroll_handle.scroll_to_item(1 + local_row_index);
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
        let show_cursor_line_emphasis = is_cursor_line && is_editor_focused;
        let caret_column = if is_editor_focused {
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
                    should_suppress_caret_blink,
                    show_cursor_line_emphasis,
                ))
                .into_any_element(),
        );
    }
    children.push(render_virtual_spacer(bottom_spacer_height_px));

    div()
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
