//! 编辑窗格与标签页状态模型。

use zom_editor::EditorState;
use zom_protocol::{BufferId, PaneId};
use zom_text::{detect_line_ending, split_lines};

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
    /// 标签页语言（由拓展名推断并缓存）。
    pub language: String,
    /// 标签页换行格式（缓存于 tab 元信息）。
    pub line_ending: String,
    /// 标签页编码（缓存于 tab 元信息）。
    pub encoding: String,
    /// 编辑器状态。
    pub editor_state: EditorState,
}

impl TabState {
    /// 返回用于查看器渲染的文本行数据。
    pub fn buffer_lines(&self) -> Vec<String> {
        split_lines(self.editor_state.text())
    }

    /// 返回完整文本内容。
    pub fn text(&self) -> &str {
        self.editor_state.text()
    }

    /// 返回该标签页的语言。
    pub fn language(&self) -> &str {
        &self.language
    }

    /// 返回文本换行格式。
    pub fn line_ending(&self) -> &str {
        &self.line_ending
    }

    /// 返回文本编码。
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// 根据当前编辑器文本刷新换行符缓存。
    pub fn refresh_line_ending_from_text(&mut self) {
        self.line_ending = detect_line_ending(self.editor_state.text());
    }
}
