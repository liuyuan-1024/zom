//! 占位面板的通用渲染函数。
// TODO: 仅用来填充面板内容，后续删除。

use gpui::{ParentElement, Styled, div, prelude::*, rgb};

use crate::theme::color;

/// 渲染仅含工具标题的占位内容。
pub(crate) fn render_title_only(title: &'static str) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .text_color(rgb(color::COLOR_FG_MUTED))
        .child(title)
}
