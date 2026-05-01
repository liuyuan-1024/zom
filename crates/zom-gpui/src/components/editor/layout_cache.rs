//! 文本布局缓存与行映射。

use zom_protocol::{BufferId, LineRange, Position};
use zom_runtime::{projection::wrap_visual_line, state::ActiveEditorSnapshot};
use zom_text_tokens::{LF_BYTE, LF_CHAR};

use crate::theme::size;

pub(super) const SOFT_WRAP_MIN_CHARS: usize = 16;
/// `APPROX_MONO_CHAR_WIDTH_PX` 的布局尺寸参数。
pub(super) const APPROX_MONO_CHAR_WIDTH_PX: f32 = 8.0;
pub(super) const GUTTER_MIN_DIGITS: usize = 2;

pub(super) struct ViewerLayoutCache {
    /// 当前缓存绑定的缓冲区 id。
    pub(super) buffer_id: BufferId,
    /// 构建缓存时对应的文档版本；版本变化即触发重建。
    pub(super) doc_version: u64,
    /// 每个软换行片段允许的最大字符数。
    pub(super) wrap_chunk: usize,
    /// 原始逻辑行数（按 `\n` 切分）。
    pub(super) line_count: usize,
    /// 每行字符数（按 char 计，不是字节）。
    pub(super) line_char_lens: Vec<usize>,
    /// 每行被软换行切分后的段数。
    pub(super) line_wrap_counts: Vec<usize>,
    /// 每行在 `wrapped_rows` 中的起始可视行索引。
    pub(super) line_start_rows: Vec<usize>,
    /// 展平后的可视行数据（供虚拟列表直接渲染）。
    pub(super) wrapped_rows: Vec<WrappedRow>,
}

pub(super) struct WrappedRow {
    /// 稳定渲染 key，避免滚动复用时元素抖动。
    pub(super) row_id: gpui::SharedString,
    /// 仅首段显示行号；续段为 `None`。
    pub(super) line_number: Option<usize>,
    /// 所属逻辑行索引。
    pub(super) line_index: usize,
    /// 所属逻辑行字符总长。
    pub(super) line_char_len: usize,
    /// 当前片段在逻辑行中的起始列（含）。
    pub(super) segment_start_column: usize,
    /// 当前片段在逻辑行中的结束列（不含）。
    pub(super) segment_end_column: usize,
    /// 是否该逻辑行最后一段（影响光标落点判定）。
    pub(super) is_last_segment: bool,
    /// 当前片段文本内容。
    pub(super) wrapped_line: String,
}

/// 读取缓存行数；仅当缓存命中文档版本与换行参数时返回。
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
    // `wrap_chunk` 统一最小为 1，避免分段除法与索引逻辑出现 0 宽异常。
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

pub(super) fn apply_dirty_line_invalidation(
    cache: &mut Option<ViewerLayoutCache>,
    active_editor: &ActiveEditorSnapshot,
    wrap_chunk: usize,
    dirty_lines: &[LineRange],
) {
    if dirty_lines.is_empty() {
        return;
    }
    let wrap_chunk = wrap_chunk.max(1);
    let Some(cache_ref) = cache.as_mut() else {
        return;
    };
    if cache_ref.buffer_id != active_editor.buffer_id || cache_ref.wrap_chunk != wrap_chunk {
        *cache = None;
        return;
    }

    let dirty_start = dirty_lines
        .iter()
        .map(|range| usize::try_from(range.start_line).unwrap_or(usize::MAX))
        .min()
        .unwrap_or(0);
    rebuild_cache_tail(cache_ref, active_editor, wrap_chunk, dirty_start);
}

/// 统计文本总行数，用于视图布局预估。
///
/// 行数定义为“LF 数 + 1”，与编辑器空文档单行语义保持一致。
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

/// 根据视口宽度与 gutter 宽度估算每行最大字符数，并保证不低于最小软换行阈值。
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

/// 将逻辑光标映射到软换行后的可视行索引，越界行列会先夹紧到文档范围。
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

/// 判断现有缓存是否仍可复用。
fn layout_cache_matches(
    cache: &ViewerLayoutCache,
    active_editor: &ActiveEditorSnapshot,
    wrap_chunk: usize,
) -> bool {
    cache.buffer_id == active_editor.buffer_id
        && cache.doc_version == active_editor.doc_version
        && cache.wrap_chunk == wrap_chunk.max(1)
}

/// 构建完整布局缓存，把逻辑行预展开成可视行列表。
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

fn rebuild_cache_tail(
    cache: &mut ViewerLayoutCache,
    active_editor: &ActiveEditorSnapshot,
    wrap_chunk: usize,
    dirty_start_line: usize,
) {
    let wrap_chunk = wrap_chunk.max(1);
    let buffer_lines = split_lines_for_viewer(&active_editor.text);
    let line_count = buffer_lines.len();
    let start_line = dirty_start_line.min(line_count);

    cache.buffer_id = active_editor.buffer_id;
    cache.doc_version = active_editor.doc_version;
    cache.wrap_chunk = wrap_chunk;
    cache.line_count = line_count;

    cache.line_char_lens.truncate(start_line);
    cache.line_wrap_counts.truncate(start_line);
    cache.line_start_rows.truncate(start_line);

    let start_row = if start_line == 0 {
        0
    } else {
        let previous_line = start_line - 1;
        cache.line_start_rows[previous_line] + cache.line_wrap_counts[previous_line]
    };
    cache.wrapped_rows.truncate(start_row);

    for (line_index, line) in buffer_lines.iter().enumerate().skip(start_line) {
        cache.line_start_rows.push(cache.wrapped_rows.len());
        let line_char_len = line.chars().count();
        cache.line_char_lens.push(line_char_len);
        let wrapped_lines = wrap_visual_line(line, wrap_chunk);
        let wrapped_count = wrapped_lines.len().max(1);
        cache.line_wrap_counts.push(wrapped_count);

        for (wrapped_index, wrapped_line) in wrapped_lines.into_iter().enumerate() {
            let segment_start_column = wrapped_index * wrap_chunk;
            let segment_end_column = segment_start_column + wrapped_line.chars().count();
            let is_last_segment = wrapped_index + 1 == wrapped_count;
            cache.wrapped_rows.push(WrappedRow {
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
}

fn split_lines_for_viewer(text: &str) -> Vec<String> {
    text.split(LF_CHAR).map(|line| line.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use zom_protocol::BufferId;
    use zom_runtime::state::ActiveEditorSnapshot;

    use super::{
        apply_dirty_line_invalidation, build_viewer_layout_cache, gutter_width_for_line_count,
        line_count_from_text, soft_wrap_max_chars,
    };

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
    /// 行号 gutter 宽度应随位数增长。
    fn gutter_width_grows_for_more_digits() {
        let width_small = gutter_width_for_line_count(9);
        let width_large = gutter_width_for_line_count(1000);
        assert!(width_large > width_small);
    }

    #[test]
    fn dirty_line_invalidation_rebuilds_tail_and_keeps_prefix_rows() {
        let before = ActiveEditorSnapshot {
            buffer_id: BufferId::new(1),
            doc_version: 1,
            selection: zom_protocol::Selection::caret(zom_protocol::Position::zero()),
            text: "aa\nbb\ncc".to_string(),
        };
        let after = ActiveEditorSnapshot {
            buffer_id: BufferId::new(1),
            doc_version: 2,
            selection: zom_protocol::Selection::caret(zom_protocol::Position::zero()),
            text: "aa\nbX\ncc".to_string(),
        };
        let mut cache = Some(build_viewer_layout_cache(&before, 16));
        let prefix_row_before = cache
            .as_ref()
            .expect("cache should exist")
            .wrapped_rows
            .first()
            .expect("first row")
            .wrapped_line
            .clone();

        apply_dirty_line_invalidation(
            &mut cache,
            &after,
            16,
            &[zom_protocol::LineRange::new(1, 2)],
        );

        let cache = cache.expect("cache should exist after invalidation");
        assert_eq!(cache.doc_version, 2);
        assert_eq!(cache.line_count, 3);
        assert_eq!(cache.wrapped_rows[0].wrapped_line, prefix_row_before);
        assert_eq!(cache.wrapped_rows[1].wrapped_line, "bX");
    }

    #[test]
    fn dirty_line_invalidation_handles_line_count_shift() {
        let before = ActiveEditorSnapshot {
            buffer_id: BufferId::new(1),
            doc_version: 1,
            selection: zom_protocol::Selection::caret(zom_protocol::Position::zero()),
            text: "a\nb\nc".to_string(),
        };
        let after = ActiveEditorSnapshot {
            buffer_id: BufferId::new(1),
            doc_version: 2,
            selection: zom_protocol::Selection::caret(zom_protocol::Position::zero()),
            text: "a\nx\ny\nb\nc".to_string(),
        };
        let mut cache = Some(build_viewer_layout_cache(&before, 16));

        apply_dirty_line_invalidation(
            &mut cache,
            &after,
            16,
            &[zom_protocol::LineRange::new(1, 4)],
        );

        let cache = cache.expect("cache should exist after invalidation");
        assert_eq!(cache.line_count, 5);
        assert_eq!(cache.wrapped_rows[0].wrapped_line, "a");
        assert_eq!(cache.wrapped_rows[1].wrapped_line, "x");
        assert_eq!(cache.wrapped_rows[2].wrapped_line, "y");
        assert_eq!(cache.wrapped_rows[3].wrapped_line, "b");
        assert_eq!(cache.wrapped_rows[4].wrapped_line, "c");
    }
}
