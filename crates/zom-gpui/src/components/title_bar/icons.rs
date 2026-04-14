//! 顶部标题栏专属的图标定义与渲染。

use gpui::{Hsla, div, prelude::*, px, svg};
use zom_app::state::TitleBarIcon;

/// 将应用层语义映射为具体的图标资源路径。
fn icon_path(icon: TitleBarIcon) -> &'static str {
    match icon {
        TitleBarIcon::Settings => "icons/title_bar/title_settings.svg",
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
        .child(svg().path(icon_path(icon)).size(px(size)).text_color(color))
}
