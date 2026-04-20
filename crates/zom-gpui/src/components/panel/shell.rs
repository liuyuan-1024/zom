//! 面板壳层容器渲染。
//! 仅负责内容容器能力。

use gpui::{FocusHandle, InteractiveElement, ParentElement, Styled, div, prelude::*, rgb};

use crate::theme::color;

/// 渲染统一面板壳层（仅内容容器）。
pub(crate) fn render_shell(
    id: &'static str,
    focus_handle: &FocusHandle,
    body: impl IntoElement,
) -> impl IntoElement {
    div()
        .id(id)
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .track_focus(focus_handle)
        .tab_index(0)
        .bg(rgb(color::COLOR_BG_PANEL))
        .child(div().size_full().overflow_hidden().child(body))
}
