//! 光标绘制与闪烁动画。

use std::time::Duration;

use gpui::{Animation, AnimationExt, IntoElement, Styled, div, px, rgb};

use crate::theme::{color, size};

/// `CARET_WIDTH_PX` 的布局尺寸参数。
pub(super) const CARET_WIDTH_PX: f32 = 1.5;
/// `CARET_HEIGHT_PX` 的布局尺寸参数。
pub(super) const CARET_HEIGHT_PX: f32 = size::FONT_MD;
pub(super) const CARET_BLINK_DURATION_MS: u64 = 1_000;
pub(super) const CARET_BLINK_PAUSE_AFTER_MOVE_MS: u64 = 500;

/// 渲染插入光标，并在允许闪烁时按固定周期切换透明度。
pub(super) fn render_caret(should_suppress_caret_blink: bool) -> impl IntoElement {
    div()
        .w(px(CARET_WIDTH_PX))
        .h(px(CARET_HEIGHT_PX))
        .flex_shrink_0()
        .bg(rgb(color::COLOR_FG_PRIMARY))
        .with_animation(
            "pane-caret-blink",
            Animation::new(Duration::from_millis(CARET_BLINK_DURATION_MS)).repeat(),
            move |this, delta| {
                let opacity = if should_suppress_caret_blink || delta < 0.5 {
                    1.0
                } else {
                    0.0
                };
                this.opacity(opacity)
            },
        )
}
