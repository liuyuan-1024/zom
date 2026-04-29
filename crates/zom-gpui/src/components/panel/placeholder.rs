//! 占位面板的通用渲染组件。
// TODO: 仅用来填充面板内容，后续删除。

use gpui::{Div, IntoElement, div, prelude::*, rgb};

use crate::theme::color;

/// 仅含工具标题的占位组件。
pub(crate) struct Placeholder {
    title: &'static str,
}

impl Placeholder {
    /// 创建占位面板，仅承载一段中心文案。
    /// 适用于尚未实现具体内容的面板骨架。
    pub fn new(title: &'static str) -> Self {
        Self { title }
    }
}

impl IntoElement for Placeholder {
    /// 为 `Element` 提供语义化类型别名。
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

/// 渲染标题栏并组装对应界面节点。
pub(crate) fn render_title_only(title: &'static str) -> impl IntoElement {
    Placeholder::new(title)
}
