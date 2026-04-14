//! 胶囊组件共用的 tooltip 语义与视图。

use gpui::{AnyView, App, Context, FontWeight, Window, div, prelude::*, px, rgb};

use crate::spacing::SPACE_1;

/// 可复用的悬停提示语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TooltipSpec {
    /// 主要提示文案。
    label: String,
    /// 对应的快捷键文案。
    shortcut: Option<String>,
}

impl TooltipSpec {
    /// 构造一个新的提示规格。
    pub(crate) fn new(label: impl Into<String>, shortcut: Option<impl Into<String>>) -> Self {
        Self {
            label: label.into(),
            shortcut: shortcut.map(Into::into),
        }
    }

    /// 返回主要提示文案。
    pub(super) fn label(&self) -> &str {
        &self.label
    }

    /// 返回快捷键文案。
    pub(super) fn shortcut(&self) -> Option<&str> {
        self.shortcut.as_deref()
    }
}

/// 构造胶囊组件共用的 tooltip 视图。
pub(super) fn tooltip_view(
    label: impl Into<String>,
    shortcut: Option<impl Into<String>>,
    cx: &mut App,
) -> AnyView {
    let label = label.into();
    let shortcut = shortcut.map(Into::into);

    cx.new(|_| TooltipView::new(label, shortcut)).into()
}

/// 胶囊组件共用的 tooltip 小视图。
struct TooltipView {
    /// 主文案，通常是入口名称。
    label: String,
    /// 该入口对应的快捷键。
    shortcut: Option<String>,
}

impl TooltipView {
    /// 创建一个新的 tooltip 视图实例。
    fn new(label: String, shortcut: Option<String>) -> Self {
        Self { label, shortcut }
    }
}

impl Render for TooltipView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let base = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(SPACE_1))
            .px(px(SPACE_1))
            .py(px(SPACE_1))
            .bg(rgb(0x151b24))
            .border_1()
            .border_color(rgb(0x2b3444))
            .rounded_sm()
            .shadow_md()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(0xe6edf7))
                    .child(self.label.clone()),
            );

        if let Some(shortcut) = &self.shortcut {
            base.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x8d9ab1))
                    .child(shortcut.clone()),
            )
        } else {
            base
        }
    }
}
