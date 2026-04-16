//! window chrome 的 bar 布局原语。
//! 这里集中维护顶栏、底栏共享的节奏、尺寸和容器样式。

use gpui::{Div, div, prelude::*, px};

use crate::theme::size;

/// 返回顶栏和底栏通用的容器样式。
pub(crate) fn bar() -> Div {
    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .p(px(size::SPACE_1))
}

/// 返回顶栏和底栏通用的水平分组样式。
pub(crate) fn group() -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(size::SPACE_1))
}
