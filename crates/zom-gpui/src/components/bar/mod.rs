//! window chrome 的 bar 布局原语。
//! 集中维护顶栏、底栏共享的节奏、尺寸和容器样式。
//! 负责汇总 bar 布局与原生红绿灯布局能力，并对外暴露稳定入口。

pub(crate) mod title_bar;
pub(crate) mod tool_bar;

use gpui::{Div, div, prelude::*, px, rgb};

use crate::theme::{color, size};

/// 返回顶栏和底栏通用的容器样式。
pub(crate) fn bar() -> Div {
    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .p(px(size::GAP_1))
        .bg(rgb(color::COLOR_BG_PANEL))
}

/// 返回顶栏和底栏通用的水平分组样式。
pub(crate) fn group() -> Div {
    div().flex().flex_row().items_center().gap(px(size::GAP_1))
}
