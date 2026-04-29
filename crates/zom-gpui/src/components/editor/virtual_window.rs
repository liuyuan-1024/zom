//! 虚拟窗口计算与占位渲染。

use std::ops::Range;

use gpui::{AnyElement, IntoElement, div, prelude::*, px};

/// `VISUAL_ROW_HEIGHT_PX` 的布局尺寸参数。
pub(super) const VISUAL_ROW_HEIGHT_PX: f32 = crate::theme::size::FONT_MD;
pub(super) const VIRTUAL_OVERSCAN_ROWS: usize = 12;

/// 计算当前应渲染的可视行窗口（含 overscan）。
///
/// `force_row` 用于强制把目标行纳入窗口，常见于光标跳转后首帧定位。
pub(super) fn virtual_row_window(
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

pub(super) fn render_virtual_spacer(height_px: f32) -> AnyElement {
    div()
        .w_full()
        .h(px(height_px.max(0.0)))
        .flex_none()
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::virtual_row_window;

    #[test]
    fn empty_rows_returns_empty_window() {
        assert_eq!(virtual_row_window(0, 0.0, 300.0, 12, None), 0..0);
    }

    #[test]
    /// 即使目标行在当前视口外，force_row 也必须被纳入渲染窗口。
    fn force_row_is_included_when_outside_visible_range() {
        let range = virtual_row_window(1000, 0.0, 200.0, 2, Some(300));
        assert!(range.contains(&300));
        assert!(range.start < range.end);
        assert!(range.end <= 1000);
    }

    #[test]
    fn non_empty_rows_always_return_non_empty_window() {
        let range = virtual_row_window(1, 10_000.0, 1.0, 0, None);
        assert_eq!(range, 0..1);
    }
}
