//! 顶部标题栏视图。

use gpui::{FontWeight, div, prelude::*, px, rgb};
use zom_app::state::{DesktopAppState, TitleBarIcon};

use super::icons;
use crate::chrome;
use crate::components::chip;

/// 渲染顶栏，表达当前工作区。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    let workspace_name = state.project_name.clone();

    chrome::bar()
        .bg(rgb(0x161a22))
        .border_b_1()
        .border_color(rgb(0x262d3a))
        .child(
            chrome::group()
                .pl(px(chrome::title_bar_leading_inset()))
                .child(
                    chip::interactive_chip(
                        "title-bar-workspace-chip",
                        chip::TooltipSpec::new(
                            format!("Project: {workspace_name}"),
                            Some("Cmd+Shift+P"),
                        ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0xe6edf7))
                            .child(state.project_name.clone()),
                    ),
                ),
        )
        .child(
            chrome::group().children(
                state
                    .title_bar
                    .right_items
                    .iter()
                    .enumerate()
                    .map(|(index, &icon)| render_settings_button(index, icon)),
            ),
        )
}

/// 渲染标题栏右侧系统设置按钮。
fn render_settings_button(index: usize, icon: TitleBarIcon) -> impl IntoElement {
    let spec = icons::spec(icon);

    chip::interactive_icon_chip(
        ("title-bar-item", index),
        chip::TooltipSpec::new(spec.label, spec.shortcut),
    )
    .child(icons::render(
        icon,
        chrome::titlebar_icon_size(),
        rgb(0xc9d4e6),
    ))
}
