//! `render_find_replace` 模块，负责 当前 域相关能力与数据组织。
use gpui::{Context, Div, Stateful};
use zom_input::shortcut_hint;
use zom_protocol::{CommandInvocation, EditorAction, WorkspaceAction};

use super::WorkspaceView;
use crate::{components::editor::find_replace_bar, root_view::store::FindReplaceField};

impl WorkspaceView {
    /// 渲染查找替换并组装对应界面节点。
    pub(super) fn render_find_replace_bar(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let state = self.store.read(cx).select_find_replace_overlay().clone();
        let active_field = match state.active_field {
            FindReplaceField::Find => find_replace_bar::ActiveField::Find,
            FindReplaceField::Replace => find_replace_bar::ActiveField::Replace,
        };
        let model = find_replace_bar::ViewModel {
            query: state.query,
            replacement: state.replacement,
            active_field,
            case_sensitive: state.case_sensitive,
            whole_word: state.whole_word,
            use_regex: state.use_regex,
            case_shortcut: shortcut_hint(&CommandInvocation::from(
                EditorAction::ToggleFindCaseSensitive,
            )),
            word_shortcut: shortcut_hint(&CommandInvocation::from(
                EditorAction::ToggleFindWholeWord,
            )),
            regex_shortcut: shortcut_hint(&CommandInvocation::from(EditorAction::ToggleFindRegex)),
            prev_shortcut: shortcut_hint(&CommandInvocation::from(EditorAction::FindPrev)),
            next_shortcut: shortcut_hint(&CommandInvocation::from(EditorAction::FindNext)),
            replace_next_shortcut: shortcut_hint(&CommandInvocation::from(
                EditorAction::ReplaceNext,
            )),
            replace_all_shortcut: shortcut_hint(&CommandInvocation::from(EditorAction::ReplaceAll)),
            close_shortcut: shortcut_hint(&CommandInvocation::from(WorkspaceAction::CloseFocused)),
        };
        find_replace_bar::render(&model)
    }
}
