//! 编辑窗格与标签页状态模型。

use zom_editor::EditorBuffer;
use zom_protocol::{BufferId, PaneId};

/// 窗格模型（带有标签页和具体内容展示区）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneState {
    /// 窗格唯一标识。
    pub id: PaneId,
    /// 当前窗格打开的标签页集合。
    pub tabs: Vec<TabState>,
    /// 当前激活标签页下标。
    pub active_tab_index: Option<usize>,
}

impl PaneState {
    /// 返回当前激活的标签页。
    pub fn active_tab(&self) -> Option<&TabState> {
        self.active_tab_index.and_then(|index| self.tabs.get(index))
    }
}

/// 标签页模型
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabState {
    /// 标签页绑定的缓冲区 ID。
    pub buffer_id: BufferId,
    /// 标签页展示标题。
    pub title: String,
    /// 工作区相对路径，用于标识该标签页绑定的文件。
    pub relative_path: String,
    /// 编辑器缓冲区。
    pub buffer: EditorBuffer,
}

impl TabState {
    /// 返回用于查看器渲染的文本行数据。
    pub fn buffer_lines(&self) -> Vec<String> {
        self.buffer.lines()
    }
}
