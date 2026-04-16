//! macOS 原生红绿灯按钮组的布局模型。
//! 这里不负责绘制按钮，只负责统一描述其位置、占位和顶栏避让规则。

use gpui::{Pixels, Point, point, px};

use crate::theme::size;

// 圆点视觉直径
const NATIVE_LIGHT_SIZE: f32 = 12.0;
// 圆点边缘之间的间距
const NATIVE_LIGHT_GAP: f32 = 8.0;

/// 返回 macOS 红绿灯按钮组在顶栏中的左上角位置。
pub(crate) fn position() -> Point<Pixels> {
    point(px(bar_padding_left()), px(vertical_offset_in_bar()))
}

/// 返回红绿灯在顶栏中预留的槽宽。
pub(crate) fn slot_width() -> f32 {
    bar_padding_left() + lights_width() + trailing_spacing()
}

/// 红绿灯整体宽度。
fn lights_width() -> f32 {
    NATIVE_LIGHT_SIZE * 3.0 + NATIVE_LIGHT_GAP * 2.0
}

/// 顶栏左侧用于放置红绿灯的起始边距。
fn bar_padding_left() -> f32 {
    size::SPACE_1
}

/// 红绿灯右侧需要额外预留的安全间距。
fn trailing_spacing() -> f32 {
    size::SPACE_2
}

/// 顶栏最终高度。
/// 顶栏高度由内部最高元素和上下 padding 共同决定。
fn bar_height() -> f32 {
    size::FONT_MD + size::SPACE_1 * 2.0
}

/// 红绿灯在顶栏中的垂直居中偏移。
fn vertical_offset_in_bar() -> f32 {
    (bar_height() - NATIVE_LIGHT_SIZE) / 2.0
}
