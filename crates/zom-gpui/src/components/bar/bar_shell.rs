//! window chrome 的 bar 布局原语。
//! 集中维护顶栏、底栏共享的节奏、尺寸和容器样式。
//! 负责汇总 bar 布局与原生红绿灯布局能力，并对外暴露稳定入口。

use gpui::{AnyElement, Div, div, prelude::*, px, rgb};

use crate::theme::{color, size};

/// 顶栏/底栏统一壳层布局容器构建器
pub(crate) struct BarShell {
    left_children: Vec<AnyElement>,
    right_children: Vec<AnyElement>,
    is_top: bool,
}

impl BarShell {
    /// 创建顶部或底部条容器，并初始化左右插槽为空集合。
    /// `is_top` 决定边框样式与定位语义。
    pub fn new(is_top: bool) -> Self {
        Self {
            left_children: Vec::new(),
            right_children: Vec::new(),
            is_top,
        }
    }

    /// 向左侧组添加元素
    pub fn left(mut self, child: impl IntoElement) -> Self {
        self.left_children.push(child.into_any_element());
        self
    }

    /// 向右侧组添加元素
    pub fn right(mut self, child: impl IntoElement) -> Self {
        self.right_children.push(child.into_any_element());
        self
    }
}

impl IntoElement for BarShell {
    type Element = Div;

    /// 组装顶栏/底栏通用壳层，依据 `is_top` 切换边框方向并拼接左右插槽内容。
    fn into_element(self) -> Self::Element {
        let mut base = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .p(px(size::GAP_1))
            .bg(rgb(color::COLOR_BG_PANEL));

        // 智能处理边框：顶栏在下，底栏在上
        if self.is_top {
            base = base.border_b_1().border_color(rgb(color::COLOR_BORDER));
        } else {
            base = base.border_t_1().border_color(rgb(color::COLOR_BORDER));
        }

        let left_group = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(size::GAP_1_5))
            .children(self.left_children);

        let right_group = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(size::GAP_1_5))
            .children(self.right_children);

        base.child(left_group).child(right_group).into_element()
    }
}
