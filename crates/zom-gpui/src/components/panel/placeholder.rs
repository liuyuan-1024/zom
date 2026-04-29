//! 占位面板的通用渲染组件。
// TODO: 仅用来填充面板内容，后续删除。

use gpui::{Div, IntoElement, div, prelude::*, rgb};

use crate::theme::color;

/// 仅含工具标题的占位组件。
pub(crate) struct Placeholder {
    title: &'static str,
}

impl Placeholder {
    pub fn new(title: &'static str) -> Self {
        Self { title }
    }
}

impl IntoElement for Placeholder {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_sm()
            .text_color(rgb(color::COLOR_FG_MUTED))
            .child(self.title)
    }
}

pub(crate) fn render_title_only(title: &'static str) -> impl IntoElement {
    Placeholder::new(title)
}
