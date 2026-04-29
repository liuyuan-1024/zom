//! 快捷键面板视图。

use gpui::{
    App, Context, FocusHandle, Focusable, InteractiveElement, ParentElement, Render, Styled,
    StatefulInteractiveElement, Window, div, px, rgb,
};
use zom_input::{default_shortcut_registry, format_keystroke_for_display, format_scope_for_display};

use crate::{
    components::panel::shell,
    theme::{color, size},
};

/// 快捷键面板。
pub(crate) struct ShortcutPanel {
    focus_handle: FocusHandle,
}

impl ShortcutPanel {
    /// 创建快捷键面板。
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for ShortcutPanel {
    /// 返回当前组件的焦点句柄，用于键盘焦点路由。
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ShortcutPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let mut rows = shortcut_rows();
        rows.sort_by(|left, right| {
            left.scope
                .cmp(&right.scope)
                .then(left.command_title.cmp(&right.command_title))
                .then(left.keystroke.cmp(&right.keystroke))
        });

        let body = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(color::COLOR_BG_PANEL))
            .child(
                div()
                    .w_full()
                    .px(px(size::GAP_2))
                    .py(px(size::GAP_2))
                    .border_b_1()
                    .border_color(rgb(color::COLOR_BORDER))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(color::COLOR_FG_PRIMARY))
                            .child("快捷键面板"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(color::COLOR_FG_MUTED))
                            .child("仅用于查看命令对应的快捷键"),
                    ),
            )
            .child(
                div()
                    .id("shortcut-panel-list")
                    .flex_1()
                    .overflow_scroll()
                    .px(px(size::GAP_1))
                    .py(px(size::GAP_1))
                    .children(rows.into_iter().enumerate().map(|(index, row)| {
                        div()
                            .id(("shortcut-row", index))
                            .w_full()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(size::GAP_2))
                            .px(px(size::GAP_1))
                            .py(px(size::GAP_1))
                            .border_b_1()
                            .border_color(rgb(color::COLOR_BORDER))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(color::COLOR_FG_PRIMARY))
                                            .child(row.command_title),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(color::COLOR_FG_MUTED))
                                            .child(row.scope),
                                    ),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(color::COLOR_FG_MUTED))
                                    .child(row.keystroke),
                            )
                    })),
            );

        shell::render_shell("shortcut-panel-container", &self.focus_handle, body)
    }
}

#[derive(Debug, Clone)]
struct ShortcutRow {
    command_title: String,
    scope: String,
    keystroke: String,
}

fn shortcut_rows() -> Vec<ShortcutRow> {
    default_shortcut_registry()
        .bindings()
        .iter()
        .map(|binding| ShortcutRow {
            command_title: binding.command.meta().title.to_string(),
            scope: format_scope_for_display(binding.scope),
            keystroke: format_keystroke_for_display(&binding.keystroke),
        })
        .collect()
}
