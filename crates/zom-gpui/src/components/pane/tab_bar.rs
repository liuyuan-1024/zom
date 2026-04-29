//! Pane 标签栏渲染与交互元素构造。

use crate::{
    components::chip::Chip,
    icon::AppIcon,
    theme::{color, opacity, size},
};
use gpui::{CursorStyle, IntoElement, div, prelude::*, px, rgb};
use zom_input::shortcut_hint;
use zom_protocol::{CommandInvocation, WorkspaceAction};
use zom_runtime::state::{PaneState, TabState};

struct TabIconSpec {
    icon: AppIcon,
    label: &'static str,
    shortcut: Option<String>,
}

/// 渲染 Pane 顶部的标签栏
pub(super) fn render(pane: &PaneState) -> impl IntoElement {
    let tabs_elements = pane.tabs.iter().enumerate().map(|(index, tab)| {
        let is_active = Some(index) == pane.active_tab_index;
        render_tab(tab, is_active, index)
    });

    div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .bg(rgb(color::COLOR_BG_PANEL))
        .border_b_1()
        .border_color(rgb(color::COLOR_BORDER))
        .children(tabs_elements)
}

/// 渲染单个标签页
fn render_tab(tab: &TabState, is_active: bool, index: usize) -> impl IntoElement {
    let group_id = format!("tab-{}", index);

    let mut tab_style = div()
        .group(group_id.clone())
        .relative()
        .p(px(size::GAP_1))
        .flex()
        .items_center()
        .justify_center()
        .border_r_1()
        .border_color(rgb(color::COLOR_BORDER))
        .text_sm()
        .cursor(CursorStyle::PointingHand)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(size::GAP_1))
                // 等宽占位，避免关闭按钮显隐时标题飘移
                .child(render_close_button_placeholder())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(tab.title.clone()),
                )
                .child(render_close_button(&group_id, index)),
        );

    if is_active {
        tab_style = tab_style
            .relative()
            .bg(rgb(color::COLOR_BG_APP))
            .text_color(rgb(color::COLOR_FG_PRIMARY));
    } else {
        tab_style = tab_style
            .bg(rgb(color::COLOR_BG_ELEMENT))
            // 未激活的 Tab 文字用次要色
            .text_color(rgb(color::COLOR_FG_MUTED))
    }

    tab_style
}

/// 渲染左侧悬浮关闭按钮
fn render_close_button(group_id: &str, index: usize) -> impl IntoElement {
    let spec = close_icon_spec();

    div()
        .size(px(size::CONTROL_XS))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(px(size::CONTROL_XS))
                .opacity(opacity::OPACITY_HIDDEN)
                .group_hover(group_id.to_string(), |style| {
                    style.opacity(opacity::OPACITY_VISIBLE)
                })
                .child(
                    Chip::new(("tab-close", index))
                        .icon(spec.icon)
                        .tooltip_hint(spec.label, spec.shortcut),
                ),
        )
}

/// 返回标签关闭按钮的图标规格。
fn close_icon_spec() -> TabIconSpec {
    TabIconSpec {
        icon: AppIcon::Close,
        label: "关闭",
        shortcut: shortcut_hint(&CommandInvocation::from(WorkspaceAction::CloseFocused)),
    }
}

/// 渲染和关闭按钮等宽的占位槽，用来维持标题居中。
fn render_close_button_placeholder() -> impl IntoElement {
    div().size(px(size::CONTROL_XS)).flex_shrink_0()
}
