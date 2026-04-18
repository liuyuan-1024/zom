//! 顶部标题栏视图。

use gpui::{App, ClickEvent, Window, div, prelude::*, px, rgb};
use zom_app::state::{DesktopAppState, TitleBarIcon};
use zom_core::{Command, command::WorkspaceCommand};
use zom_input::shortcut_hint;

use super::icons;
use crate::chrome;
use crate::components::chip;
use crate::components::title_bar::traffic_lights;
use crate::theme::{color, size};

/// 渲染顶栏，表达当前工作区。
pub(crate) fn render(
    state: &DesktopAppState,
    on_project_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    chrome::bar()
        .border_b_1()
        .border_color(rgb(color::COLOR_BORDER))
        .child(
            chrome::group().pl(px(traffic_lights::slot_width())).child(
                chip::interactive_chip(
                    "title-bar-project_name",
                    chip::TooltipSpec::new(
                        format!("选择项目"),
                        shortcut_hint(&Command::from(WorkspaceCommand::OpenProjectPicker)),
                    ),
                )
                .on_click(on_project_click)
                .child(
                    div()
                        .text_xs()
                        .line_height(px(size::FONT_MD))
                        .text_color(rgb(color::COLOR_FG_PRIMARY))
                        .child(state.project_name.clone()),
                ),
            ),
        )
        .child(
            chrome::group().children(
                state
                    .title_bar
                    .right_icons
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
        size::ICON_MD,
        rgb(color::COLOR_FG_MUTED),
    ))
}
