//! 胶囊组件视图构造器。

use gpui::{
    CursorStyle, Div, ElementId, Stateful, StatefulInteractiveElement, div, prelude::*, px, rgb,
};

use super::{TooltipSpec, tooltip::tooltip_view};
use crate::chrome;

/// 胶囊组件的统一高度。
const CHIP_HEIGHT: f32 = 24.0;

/// 返回 chrome 体系下通用的文本胶囊样式。
pub(crate) fn chip() -> Div {
    div()
        .h(px(CHIP_HEIGHT))
        .px(px(chrome::chrome_padding()))
        .flex()
        .flex_row()
        .items_center()
        .bg(rgb(0x0f1319))
        .border_1()
        .border_color(rgb(0x2b3444))
        .rounded_sm()
}

/// 返回 chrome 体系下通用的图标胶囊样式。
pub(crate) fn icon_chip() -> Div {
    div()
        .w(px(CHIP_HEIGHT))
        .h(px(CHIP_HEIGHT))
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
        .text_color(rgb(0xd7e0ef))
        .bg(rgb(0x121923))
        .border_color(rgb(0x283243))
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
