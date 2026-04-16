//! Pane 专属的图标定义与渲染。

use gpui::{Hsla, div, prelude::*, px, svg};

/// Pane 内部使用的图标语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneIcon {
    Close,
}

pub(super) struct PaneIconSpec {
    pub path: &'static str,
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
}

pub(super) fn spec(icon: PaneIcon) -> PaneIconSpec {
    match icon {
        PaneIcon::Close => PaneIconSpec {
            path: "icons/tab/close.svg",
            label: "Close",
            shortcut: Some("Cmd+W"),
        },
    }
}

/// 渲染 Pane 中的单色 SVG 图标。
pub(super) fn render(icon: PaneIcon, size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    let color = color.into();
    let spec = spec(icon);

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(spec.path).size(px(size)).text_color(color))
}
