//! 胶囊组件视图构造器。

use gpui::{CursorStyle, Div, ElementId, Stateful, StatefulInteractiveElement, div, prelude::*};

use super::{TooltipSpec, tooltip::tooltip_view};

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

/// 返回通用的文本胶囊样式。
fn chip() -> Div {
    div().flex().flex_row().items_center().justify_center()
    // .rounded_sm()
}

/// 返回通用的图标胶囊样式。
fn icon_chip() -> Div {
    div().flex().items_center().justify_center()
    // .rounded_sm()
}
