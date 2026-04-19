//! 顶部标题栏专属的图标定义与渲染。

use gpui::{Hsla, div, prelude::*, px, svg};
use zom_app::projection::shortcut_hint;
use zom_app::state::TitleBarIcon;
use zom_core::{CommandInvocation, WorkspaceAction};

/// 标题栏图标的展示规格。
pub(super) struct TitleBarIconSpec {
    /// SVG 资源路径。
    pub path: &'static str,
    /// 悬停时显示的名称。
    pub label: &'static str,
    /// 悬停时显示的快捷键。
    pub shortcut: Option<String>,
}

/// 将应用层语义映射为标题栏自身维护的展示规格。
pub(super) fn spec(icon: TitleBarIcon) -> TitleBarIconSpec {
    match icon {
        TitleBarIcon::Settings => TitleBarIconSpec {
            path: "icons/title_bar/title_settings.svg",
            label: "Settings",
            shortcut: shortcut_hint(&CommandInvocation::from(WorkspaceAction::OpenSettings)),
        },
    }
}

/// 渲染标题栏中的单色 SVG 图标。
pub(super) fn render(icon: TitleBarIcon, size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    let color = color.into();
    let spec = spec(icon);

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(spec.path).size(px(size)).text_color(color))
}
