//! Pane 内容路由：根据 active tab 选择具体内容视图。

use gpui::Entity;
use zom_runtime::state::PaneState;

use crate::components::editor::EditorView;

/// 窗格内容路由器，按活动标签类型选择具体内容视图实体。
pub(crate) struct PaneContentRouter {
    editor_view: Entity<EditorView>,
}

impl PaneContentRouter {
    /// 创建窗格内容路由器，并注入当前默认编辑器视图实体。
    /// 路由器后续根据标签类型返回对应视图实例。
    pub(crate) fn new(editor_view: Entity<EditorView>) -> Self {
        Self { editor_view }
    }

    /// 返回当前活动标签应渲染的视图实体。
    /// 现阶段统一映射为文本编辑器，后续可扩展到终端或其他内容类型。
    pub(crate) fn view_for_active_tab(&self, _pane: &PaneState) -> Entity<EditorView> {
        // 当前仅支持文本编辑器；后续可在此扩展 image/terminal 等内容类型。
        self.editor_view.clone()
    }
}
