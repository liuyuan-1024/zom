use crate::{
    components::{
        chip,
        pane::icons::{self, PaneIcon},
    },
    theme::{
        color,
        size::{self, SPACE_1, SPACE_4},
    },
};
use gpui::{CursorStyle, IntoElement, div, prelude::*, px, rgb};
use zom_app::state::{PaneState, TabState};

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
        .items_end()
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
        .py(px(SPACE_1))
        .px(px(SPACE_4))
        .flex()
        .items_center()
        .justify_center()
        .border_r_1()
        .border_color(rgb(color::COLOR_BORDER))
        .text_sm()
        .cursor(CursorStyle::PointingHand)
        .child(render_close_button(&group_id, index))
        .child(
            div()
                .overflow_hidden()
                .whitespace_nowrap()
                .child(tab.title.clone()),
        );

    if is_active {
        tab_style = tab_style
            .relative()
            .bg(rgb(color::COLOR_BG_APP))
            .text_color(rgb(color::COLOR_FG_PRIMARY))
            .child(
                div()
                    .absolute()
                    .bottom(px(-1.0))
                    .left_0()
                    .right_0()
                    .h(px(1.0))
                    // 遮住底部边框，与编辑器区融为一体
                    .bg(rgb(color::COLOR_BG_APP)),
            );
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
    let icon = PaneIcon::Close;
    let spec = icons::spec(icon);

    div()
        .absolute()
        .left(px(4.0))
        .top_0()
        .bottom_0()
        .flex()
        .items_center()
        .child(
            chip::interactive_icon_chip(
                ("tab-close", index),
                chip::TooltipSpec::new(spec.label, spec.shortcut),
            )
            .size(px(size::CONTROL_XS))
            .opacity(0.0)
            .group_hover(group_id.to_string(), |style| style.opacity(1.0))
            .hover(|style| style.bg(rgb(color::COLOR_BG_HOVER)))
            .child(icons::render(
                icon,
                size::ICON_XS,
                rgb(color::COLOR_FG_MUTED),
            )),
        )
}
