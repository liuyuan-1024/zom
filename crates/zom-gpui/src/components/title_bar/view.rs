//! 顶部标题栏视图。

use gpui::{CursorStyle, FontWeight, div, prelude::*, px, rgb};
use zom_app::state::{DesktopAppState, TitleBarIcon};

use super::icons;
use crate::chrome;

/// 渲染顶栏，表达当前工作区。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    chrome::bar()
        .bg(rgb(0x161a22))
        .border_b_1()
        .border_color(rgb(0x262d3a))
        .child(
            chrome::group()
                .pl(px(chrome::title_bar_leading_inset()))
                .child(
                    chrome::chip().cursor(CursorStyle::PointingHand).child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0xe6edf7))
                            .child(state.workspace_name.clone()),
                    ),
                ),
        )
        .child(
            chrome::group().children(
                state
                    .title_bar
                    .right_items
                    .iter()
                    .map(|&icon| render_settings_button(icon)),
            ),
        )
}

/// 渲染标题栏右侧系统设置按钮。
fn render_settings_button(icon: TitleBarIcon) -> impl IntoElement {
    chrome::icon_chip()
        .cursor(CursorStyle::PointingHand)
        .child(icons::render(
            icon,
            chrome::titlebar_icon_size(),
            rgb(0xc9d4e6),
        ))
}
