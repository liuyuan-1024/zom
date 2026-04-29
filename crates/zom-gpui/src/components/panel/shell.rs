//! 面板壳层容器渲染。
//! 仅负责内容容器能力。

use gpui::{
    AnyElement, Div, ElementId, FocusHandle, IntoElement, ParentElement, Stateful, div, prelude::*,
    rgb,
};

use crate::theme::color;

/// 统一面板壳层构建器（仅内容容器）。
pub(crate) struct PanelShell {
    id: ElementId,
    focus_handle: Option<FocusHandle>,
    children: Vec<AnyElement>,
}

impl PanelShell {
    /// 初始化面板壳层
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            focus_handle: None,
            children: Vec::new(),
        }
    }

    /// 绑定焦点句柄 (可选)
    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }
}

// 自动获得 .child() 和 .children() 的能力
impl ParentElement for PanelShell {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl IntoElement for PanelShell {
    // 带有 .id() 的 div 在 GPUI 中类型为 Stateful<Div>
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let mut base = div()
            .id(self.id)
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .tab_index(0) // 确保容器可聚焦
            .bg(rgb(color::COLOR_BG_PANEL));

        // 仅在显式传入时绑定焦点
        if let Some(focus_handle) = self.focus_handle {
            base = base.track_focus(&focus_handle);
        }

        // 内部布局封装，业务方只管传 child，不用关心 overflow 怎么裁切
        base.child(div().size_full().overflow_hidden().children(self.children))
            .into_element()
    }
}

pub(crate) fn render_shell(
    id: impl Into<ElementId>,
    focus_handle: &FocusHandle,
    body: impl IntoElement,
) -> Stateful<Div> {
    PanelShell::new(id).track_focus(focus_handle).child(body).into_element()
}
