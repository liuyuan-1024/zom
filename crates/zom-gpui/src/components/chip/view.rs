//! Chip 组件视图构造与表现封装。
//! 定位：全键盘驱动下的状态指示器与快捷键提示容器（无点击交互）

use gpui::{
    AnyElement, Div, ElementId, Stateful, StatefulInteractiveElement, div, prelude::*, px, rgb, svg,
};

use super::{TooltipSpec, tooltip::tooltip_view};
use crate::{
    icon::AppIcon,
    theme::{color, size},
};

/// 胶囊的视觉风格变体
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub(crate) enum ChipStyle {
    /// 幽灵样式：无边框，无默认背景色。
    /// 常用于图标按钮（顶栏、底栏、Tab关闭）。
    #[default]
    Ghost,
    /// 轮廓样式：带边框和内边距，区分选中态。
    /// 常用于文本过滤或操作按钮（查找替换、通知栏操作）。
    Outlined,
}

/// 语义化的胶囊组件构建器
pub(crate) struct Chip {
    id: ElementId,
    style: ChipStyle,
    is_active: bool,
    tooltip: Option<TooltipSpec>,
    label: Option<AnyElement>,
    icon: Option<AppIcon>,
    icon_size: f32,
}

impl Chip {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: ChipStyle::default(),
            is_active: false,
            tooltip: None,
            label: None,
            icon: None,
            icon_size: size::ICON_MD,
        }
    }

    /// 显式添加图标
    /// 传入统一图标语义，具体路径由 `components/icon.rs` 集中映射。
    pub fn icon(mut self, icon: AppIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 按需覆盖默认的 ICON_MD 大小
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    /// 显式添加文字标签
    pub fn label(mut self, label: impl IntoElement) -> Self {
        self.label = Some(label.into_any_element());
        self
    }

    /// 设置视觉风格
    pub fn style(mut self, style: ChipStyle) -> Self {
        self.style = style;
        self
    }

    /// 设置是否处于激活状态（主要影响 Outlined 风格的背景和文字颜色）
    pub fn active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// 统一设置 tooltip（可选快捷键）。
    pub fn tooltip_hint(
        mut self,
        label: impl Into<String>,
        shortcut: Option<impl Into<String>>,
    ) -> Self {
        self.tooltip = Some(TooltipSpec::new(label, shortcut));
        self
    }

    /// 仅提示文案的 tooltip。
    pub fn tooltip_text(self, label: impl Into<String>) -> Self {
        self.tooltip_hint(label, Option::<String>::None)
    }

    fn into_stateful(self) -> Stateful<Div> {
        let text_color = if self.is_active {
            color::COLOR_FG_PRIMARY
        } else {
            color::COLOR_FG_MUTED
        };

        let mut base = div().id(self.id).flex().items_center().justify_center();

        match self.style {
            ChipStyle::Ghost => {
                base = base.hover(|style| style.bg(rgb(color::COLOR_BG_HOVER)));
            }
            ChipStyle::Outlined => {
                let bg_color = if self.is_active {
                    color::COLOR_BG_ACTIVE
                } else {
                    color::COLOR_BG_ELEMENT
                };
                let text_color = if self.is_active {
                    color::COLOR_FG_PRIMARY
                } else {
                    color::COLOR_FG_MUTED
                };

                base = base
                    .px(px(size::GAP_1))
                    .py(px(size::GAP_1))
                    .border_1()
                    .border_color(rgb(color::COLOR_BORDER))
                    .rounded_sm()
                    .bg(rgb(bg_color))
                    .text_xs()
                    .text_color(rgb(text_color))
                    .hover(|style| style.bg(rgb(color::COLOR_BG_HOVER)));
            }
        }

        // 文字颜色统一设置
        base = base.text_color(rgb(text_color));

        if let Some(tooltip) = self.tooltip {
            base = base.tooltip(move |_, cx| {
                tooltip_view(
                    tooltip.label().to_string(),
                    tooltip.shortcut().map(str::to_string),
                    cx,
                )
            });
        }

        let mut container = div().flex().items_center().gap(px(size::GAP_1));

        // 渲染内部接管：使用相同的 text_color 为 SVG 染色
        if let Some(icon) = self.icon {
            container = container.child(
                div()
                    .size(px(self.icon_size))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path(icon.asset_path())
                            .size(px(self.icon_size))
                            .text_color(rgb(text_color)),
                    ),
            );
        }

        if let Some(label) = self.label {
            container = container.child(label);
        }

        base.child(container)
    }
}

impl IntoElement for Chip {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        self.into_stateful()
    }
}
