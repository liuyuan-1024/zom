//! 胶囊组件视图构造器。

use gpui::{
    CursorStyle, Div, ElementId, Stateful, StatefulInteractiveElement, div, prelude::*, px, rgb,
};

use super::{TooltipSpec, tooltip::tooltip_view};
use crate::theme::{color, spacing};

/// 返回 chrome 体系下通用的文本胶囊样式。
pub(crate) fn chip() -> Div {
    div()
        .px(px(spacing::SPACE_1))
        .flex()
        .flex_row()
        .items_center()
        .bg(rgb(color::COLOR_BG_ELEMENT))
        .border_1()
        .border_color(rgb(color::COLOR_BORDER))
        .rounded_sm()
}

/// 返回 chrome 体系下通用的图标胶囊样式。
pub(crate) fn icon_chip() -> Div {
    div()
        .px(px(spacing::SPACE_1))
        .flex()
        .items_center()
        .justify_center()
        .rounded_sm()
}

/// 返回用于只读状态值的胶囊样式。
/// 这类胶囊不承担点击语义，只负责表达当前状态文本。
pub(crate) fn status_chip() -> Div {
    chip()
        .text_xs()
        .text_color(rgb(color::COLOR_FG_MUTED))
        .bg(rgb(color::COLOR_BG_PANEL))
        .border_color(rgb(color::COLOR_BORDER))
}

/// 返回带统一悬停提示的文本胶囊按钮。
pub(crate) fn interactive_chip(id: impl Into<ElementId>, tooltip: TooltipSpec) -> Stateful<Div> {
    chip()
        .cursor(CursorStyle::PointingHand)
        .id(id)
        .tooltip(move |_, cx| {
            tooltip_view(
                tooltip.label().to_string(),
                tooltip.shortcut().map(str::to_string),
                cx,
            )
        })
}

/// 返回带统一悬停提示的图标胶囊按钮。
pub(crate) fn interactive_icon_chip(
    id: impl Into<ElementId>,
    tooltip: TooltipSpec,
) -> Stateful<Div> {
    icon_chip()
        .cursor(CursorStyle::PointingHand)
        .id(id)
        .tooltip(move |_, cx| {
            tooltip_view(
                tooltip.label().to_string(),
                tooltip.shortcut().map(str::to_string),
                cx,
            )
        })
}
