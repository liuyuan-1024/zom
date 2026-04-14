//! 顶部标题栏专属的图标定义与渲染。

use gpui::{Hsla, div, prelude::*, px, svg};

/// 标题栏会用到的图标类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TitleBarIcon {
    /// 设置图标。
    Settings,
}

impl TitleBarIcon {
    /// 返回图标对应的 SVG 资源路径。
    fn path(self) -> &'static str {
        match self {
            Self::Settings => "icons/title_bar/title_settings.svg",
        }
    }
}

/// 渲染标题栏中的单色 SVG 图标。
pub(super) fn render(icon: TitleBarIcon, size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    let color = color.into();

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(icon.path()).size(px(size)).text_color(color))
}
