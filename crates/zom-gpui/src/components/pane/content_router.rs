//! Pane 内容路由：根据 active tab 选择具体内容视图。

use gpui::Entity;
use zom_runtime::state::PaneState;

use crate::components::editor::EditorView;

pub(crate) struct PaneContentRouter {
    editor_view: Entity<EditorView>,
}

impl PaneContentRouter {
    pub(crate) fn new(editor_view: Entity<EditorView>) -> Self {
        Self { editor_view }
    }

    pub(crate) fn view_for_active_tab(&self, _pane: &PaneState) -> Entity<EditorView> {
        // 当前仅支持文本编辑器；后续可在此扩展 image/terminal 等内容类型。
        self.editor_view.clone()
    }
}
