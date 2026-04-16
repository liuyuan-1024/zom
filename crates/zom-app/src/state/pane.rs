use zom_core::{BufferId, PaneId};

/// 窗格模型（带有标签页和具体内容展示区）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneState {
    pub id: PaneId,
    pub tabs: Vec<TabState>,
    pub active_tab_index: Option<usize>,
}

/// 标签页模型
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabState {
    pub buffer_id: BufferId,
    pub title: String,
}
