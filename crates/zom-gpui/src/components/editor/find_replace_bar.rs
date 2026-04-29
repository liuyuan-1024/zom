//! 编辑器内嵌查找替换条组件。

use gpui::{Div, Stateful, div, prelude::*, px, rgb};

use crate::{
    components::chip::Chip,
    icon::AppIcon,
    theme::{color, size},
};

/// 查找替换条当前激活字段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActiveField {
    Find,
    Replace,
}

/// 查找替换条视图状态快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ViewModel {
    pub query: String,
    pub replacement: String,
    pub active_field: ActiveField,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub use_regex: bool,
    pub case_shortcut: Option<String>,
    pub word_shortcut: Option<String>,
    pub regex_shortcut: Option<String>,
    pub prev_shortcut: Option<String>,
    pub next_shortcut: Option<String>,
    pub replace_next_shortcut: Option<String>,
    pub replace_all_shortcut: Option<String>,
    pub close_shortcut: Option<String>,
}

/// 渲染查找替换条。
pub(crate) fn render(model: &ViewModel) -> Stateful<Div> {
    let find_bg = if matches!(model.active_field, ActiveField::Find) {
        color::COLOR_BG_ACTIVE
    } else {
        color::COLOR_BG_ELEMENT
    };
    let replace_bg = if matches!(model.active_field, ActiveField::Replace) {
        color::COLOR_BG_ACTIVE
    } else {
        color::COLOR_BG_ELEMENT
    };

    div()
        .id("editor-find-replace-bar")
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .p(px(8.0))
        .border_b_1()
        .border_color(rgb(color::COLOR_BORDER))
        .bg(rgb(color::COLOR_BG_PANEL))
        .child(
            div().flex().items_center().gap(px(8.0)).child(
                div()
                    .id("editor-find-replace-find-input")
                    .flex_1()
                    .bg(rgb(find_bg))
                    .border_1()
                    .border_color(rgb(color::COLOR_BORDER))
                    .rounded_sm()
                    .px(px(8.0))
                    .py(px(6.0))
                    .text_color(rgb(color::COLOR_FG_PRIMARY))
                    .child(if model.query.is_empty() {
                        "查找".to_string()
                    } else {
                        model.query.clone()
                    }),
            ),
        )
        .child(
            div().flex().items_center().gap(px(8.0)).child(
                div()
                    .id("editor-find-replace-replace-input")
                    .flex_1()
                    .bg(rgb(replace_bg))
                    .border_1()
                    .border_color(rgb(color::COLOR_BORDER))
                    .rounded_sm()
                    .px(px(8.0))
                    .py(px(6.0))
                    .text_color(rgb(color::COLOR_FG_PRIMARY))
                    .child(if model.replacement.is_empty() {
                        "替换".to_string()
                    } else {
                        model.replacement.clone()
                    }),
            ),
        )
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(size::GAP_1))
                        .child(
                            Chip::new("find-replace-chip-match-case")
                                .icon(AppIcon::FindReplaceCaseSensitive)
                                .active(model.case_sensitive)
                                .tooltip_hint("Match Case", model.case_shortcut.clone()),
                        )
                        .child(
                            Chip::new("find-replace-chip-whole-word")
                                .icon(AppIcon::FindReplaceWholeWord)
                                .active(model.whole_word)
                                .tooltip_hint("Whole Word", model.word_shortcut.clone()),
                        )
                        .child(
                            Chip::new("find-replace-chip-regex")
                                .icon(AppIcon::FindReplaceRegex)
                                .active(model.use_regex)
                                .tooltip_hint("正则", model.regex_shortcut.clone()),
                        ),
                )
                .child(
                    div().flex().items_center().gap(px(size::GAP_1)).children([
                        Chip::new("find-replace-chip-prev")
                            .icon(AppIcon::ChevronUp)
                            .tooltip_hint("查找上一个", model.prev_shortcut.clone()),
                        Chip::new("find-replace-chip-next")
                            .icon(AppIcon::ChevronDown)
                            .tooltip_hint("查找下一个", model.next_shortcut.clone()),
                        Chip::new("find-replace-chip-replace")
                            .icon(AppIcon::FindReplaceNext)
                            .tooltip_hint("替换下一个", model.replace_next_shortcut.clone()),
                        Chip::new("find-replace-chip-replace-all")
                            .icon(AppIcon::FindReplaceAll)
                            .tooltip_hint("替换全部", model.replace_all_shortcut.clone()),
                    ]),
                ),
        )
}
