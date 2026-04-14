//! window chrome 的 bar 布局原语。
//! 这里集中维护顶栏、底栏共享的节奏、尺寸和容器样式。

use gpui::{Div, div, prelude::*, px};

use crate::spacing::SPACE_1;

/// 顶栏和底栏共用的内边距，统一控制四周留白。
const CHROME_PADDING: f32 = SPACE_1;
/// 顶栏和底栏内部元素共用的间距。
const CHROME_GAP: f32 = SPACE_1;
/// 顶栏图标使用的尺寸。
const TITLEBAR_ICON_SIZE: f32 = 15.0;
/// 工具栏图标使用的尺寸。
const TOOL_ICON_SIZE: f32 = 15.0;
/// 顶栏与底栏的内容高度基准。
/// bar 的整体高度应由图标尺寸加上下间距推导，而不是由胶囊高度反推。
const CHROME_BAR_CONTENT_HEIGHT: f32 = TITLEBAR_ICON_SIZE;
/// 顶栏和底栏的总高度，由图标尺寸和上下留白共同推导得出。
pub(super) const CHROME_BAR_HEIGHT: f32 = CHROME_BAR_CONTENT_HEIGHT + CHROME_PADDING * 2.0;

/// 返回顶栏和底栏通用的容器样式。
pub(crate) fn bar() -> Div {
    div()
        .w_full()
        .h(px(CHROME_BAR_HEIGHT))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .p(px(CHROME_PADDING))
}

/// 返回顶栏和底栏通用的水平分组样式。
pub(crate) fn group() -> Div {
    div().flex().flex_row().items_center().gap(px(CHROME_GAP))
}

/// 返回 chrome 共用的胶囊内边距。
pub(crate) fn chrome_padding() -> f32 {
    CHROME_PADDING
}

/// 返回顶栏图标的统一尺寸。
pub(crate) fn titlebar_icon_size() -> f32 {
    TITLEBAR_ICON_SIZE
}

/// 返回工具栏图标的统一尺寸。
pub(crate) fn tool_icon_size() -> f32 {
    TOOL_ICON_SIZE
}
