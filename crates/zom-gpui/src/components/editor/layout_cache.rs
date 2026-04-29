//! 文本布局缓存与行映射。

use zom_protocol::{BufferId, Position};
use zom_runtime::{projection::wrap_visual_line, state::ActiveEditorSnapshot};
use zom_text_tokens::{LF_BYTE, LF_CHAR};

use crate::theme::size;

pub(super) const SOFT_WRAP_MIN_CHARS: usize = 16;
pub(super) const APPROX_MONO_CHAR_WIDTH_PX: f32 = 8.0;
pub(super) const GUTTER_MIN_DIGITS: usize = 2;

pub(super) struct ViewerLayoutCache {
    pub(super) buffer_id: BufferId,
    pub(super) doc_version: u64,
    pub(super) wrap_chunk: usize,
    pub(super) line_count: usize,
    pub(super) line_char_lens: Vec<usize>,
    pub(super) line_wrap_counts: Vec<usize>,
    pub(super) line_start_rows: Vec<usize>,
    pub(super) wrapped_rows: Vec<WrappedRow>,
}

pub(super) struct WrappedRow {
    pub(super) row_id: gpui::SharedString,
    pub(super) line_number: Option<usize>,
    pub(super) line_index: usize,
    pub(super) line_char_len: usize,
    pub(super) segment_start_column: usize,
    pub(super) segment_end_column: usize,
    pub(super) is_last_segment: bool,
    pub(super) wrapped_line: String,
}

pub(super) fn cached_line_count(
    cache: Option<&ViewerLayoutCache>,
    active_editor: &ActiveEditorSnapshot,
) -> Option<usize> {
    let cache = cache?;
    layout_cache_matches(cache, active_editor, cache.wrap_chunk).then_some(cache.line_count)
}

pub(super) fn ensure_viewer_layout_cache<'a>(
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

pub(super) fn line_count_from_text(text: &str) -> usize {
    text.bytes()
        .filter(|byte| *byte == LF_BYTE)
        .count()
        .saturating_add(1)
}

pub(super) fn gutter_width_for_line_count(line_count: usize) -> f32 {
    let digits = line_count.max(1).to_string().len().max(GUTTER_MIN_DIGITS);
    let content_width_px = digits as f32 * APPROX_MONO_CHAR_WIDTH_PX;
    let horizontal_padding_px = size::GAP_1 * 2.0;
    (content_width_px + horizontal_padding_px).max(size::GUTTER_MD)
}

pub(super) fn soft_wrap_max_chars(
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

pub(super) fn cursor_visual_row_index(layout_cache: &ViewerLayoutCache, cursor: Position) -> usize {
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
                row_id: gpui::SharedString::from(format!("viewer-row-{line_index}-{wrapped_index}")),
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
    text.split(LF_CHAR).map(|line| line.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::{gutter_width_for_line_count, line_count_from_text, soft_wrap_max_chars};

    #[test]
    fn line_count_handles_trailing_newline() {
        assert_eq!(line_count_from_text(""), 1);
        assert_eq!(line_count_from_text("a"), 1);
        assert_eq!(line_count_from_text("a\n"), 2);
        assert_eq!(line_count_from_text("a\nb\nc"), 3);
    }

    #[test]
    fn soft_wrap_never_drops_below_minimum() {
        let chunk = soft_wrap_max_chars(0.0, 1.0, 10_000.0);
        assert!(chunk >= 16);
    }

    #[test]
    fn gutter_width_grows_for_more_digits() {
        let width_small = gutter_width_for_line_count(9);
        let width_large = gutter_width_for_line_count(1000);
        assert!(width_large > width_small);
    }
}
