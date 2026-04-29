//! Chip 组件视图构造与表现封装。
//! 定位：全键盘驱动下的状态指示器与快捷键提示容器（无点击交互）

use gpui::{
    AnyElement, CursorStyle, Div, ElementId, Stateful, StatefulInteractiveElement, div, prelude::*,
    px, rgb, svg,
};

use super::{tooltip::TooltipSpec, tooltip::tooltip_view};
use crate::{
    icon::AppIcon,
    theme::{color, size},
};

/// 语义化的胶囊组件构建器
/// 幽灵样式：无边距，无边框，无默认背景色。
pub(crate) struct Chip {
    id: ElementId,
    is_active: bool,
    tooltip: Option<TooltipSpec>,
    label: Option<AnyElement>,
    label_size: f32,
    icon: Option<AppIcon>,
    icon_size: f32,
}

impl Chip {
    /// 创建一个默认未激活的 Chip，并注入稳定元素 ID。
    /// 初始样式使用中等字号与图标尺寸，后续可链式覆写。
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            is_active: false,
            tooltip: None,
            label: None,
            label_size: size::FONT_CHIP,
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

    /// 按需覆盖默认图标大小
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    /// 显式添加文字标签
    pub fn label(mut self, label: impl IntoElement) -> Self {
        self.label = Some(label.into_any_element());
        self
    }

    /// 按需覆盖默认文字大小
    pub fn label_size(mut self, size: f32) -> Self {
        self.label_size = size;
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

    /// 将构建参数收敛为可渲染状态节点，统一处理颜色、图标和 tooltip 组合逻辑。
    fn into_stateful(self) -> Stateful<Div> {
        let text_color = if self.is_active {
            color::COLOR_FG_PRIMARY
        } else {
            color::COLOR_FG_MUTED
        };
        let icon_color = text_color;

        let mut base = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            // 显式使用，避免依赖 gpui 默认文本度量
            .text_size(px(self.label_size))
            .line_height(px(self.label_size))
            .cursor(CursorStyle::PointingHand);

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
                            .text_color(rgb(icon_color)),
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
    /// 为 `Element` 提供语义化类型别名。
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        self.into_stateful()
    }
}
