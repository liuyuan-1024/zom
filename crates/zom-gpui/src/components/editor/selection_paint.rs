//! 选区、行号和文本片段渲染。

use std::ops::Range;

use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px, rgb};
use zom_protocol::Selection;

use crate::theme::{color, size};

use super::caret::render_caret;

pub(super) fn render_gutter_cell(line_number: Option<usize>, gutter_width_px: f32) -> AnyElement {
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

/// 渲染文本区单元：文本、选区底色与光标会在同一行内容树里合成。
pub(super) fn render_text_cell(
    wrapped_line: &str,
    selected_range: Option<Range<usize>>,
    caret_column: Option<usize>,
    should_suppress_caret_blink: bool,
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
            should_suppress_caret_blink,
        ))
        .into_any_element()
}

/// 计算某逻辑行上的选区列范围（按半开区间）。
///
/// 无选区、行不在选区内、或区间收缩为空时返回 `None`。
pub(super) fn selected_column_range_for_line(
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

/// 把“整行选区列范围”裁剪到当前软换行片段内。
pub(super) fn selected_range_in_wrapped_segment(
    selected_columns_in_line: Option<&Range<usize>>,
    segment_start_column: usize,
    segment_end_column: usize,
) -> Option<Range<usize>> {
    let selected_columns_in_line = selected_columns_in_line?;
    let from = selected_columns_in_line.start.max(segment_start_column);
    let to = selected_columns_in_line.end.min(segment_end_column);
    (from < to).then_some((from - segment_start_column)..(to - segment_start_column))
}

/// 计算光标是否落在当前软换行片段内，并返回片段内列号。
///
/// 非末片段不允许光标落在 `segment_end_column`，避免双片段重复绘制光标。
pub(super) fn caret_column_in_wrapped_segment(
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

/// 渲染软换行片段内部内容，并按需要插入光标元素。
fn render_wrapped_line_content(
    wrapped_line: &str,
    selected_range: Option<Range<usize>>,
    caret_column: Option<usize>,
    should_suppress_caret_blink: bool,
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
            children.push(render_caret(should_suppress_caret_blink).into_any_element());
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
        children.push(render_caret(should_suppress_caret_blink).into_any_element());
    }

    if children.is_empty() {
        if caret_column == Some(0) {
            children.push(render_caret(should_suppress_caret_blink).into_any_element());
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

/// 渲染连续同状态文本块（选中/未选中）。
fn render_text_chunk(text: String, is_selected: bool) -> AnyElement {
    if is_selected {
        div()
            .bg(rgb(color::COLOR_BG_ACTIVE))
            .child(text)
            .into_any_element()
    } else {
        div().child(text).into_any_element()
    }
}
