//! 桌面应用状态与命令分发主状态机。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use zom_editor::EditorState;
use zom_protocol::{BufferId, FocusTarget, OverlayTarget, Selection};

use crate::state::{
    FileTreeState, PaneState, PanelDock, TitleBarState, ToolBarState, dock_targets,
};

mod command;
mod focus;
mod project;
mod tabs;

#[cfg(test)]
mod tests;

/// 需要在 UI 层执行的副作用动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopUiAction {
    /// 退出应用。
    QuitApp,
    /// 最小化当前窗口。
    MinimizeWindow,
    /// 打开项目目录选择器。
    OpenProjectPicker,
}

/// 当前激活编辑器的渲染快照（面向 UI，只读）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveEditorSnapshot {
    /// 所属 buffer id。
    pub buffer_id: BufferId,
    /// 文档版本号。
    pub doc_version: u64,
    /// 当前选区。
    pub selection: Selection,
    /// 当前完整文本。
    pub text: String,
}

/// 桌面端根界面使用的应用状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAppState {
    /// 顶部标题栏状态。
    pub title_bar: TitleBarState,
    /// 底部工具栏信息。
    pub tool_bar: ToolBarState,
    /// 左侧文件树内容。
    pub file_tree: FileTreeState,
    /// 窗格
    pub pane: PaneState,
    /// 活跃标签页对应的编辑器状态仓库（key = buffer id）。
    pub(crate) editor_states: HashMap<BufferId, EditorState>,
    /// 当前聚焦目标。
    pub focused_target: FocusTarget,
    /// 当前可见的工作台面板集合。
    pub visible_panels: HashSet<FocusTarget>,
    /// 当前可见的悬浮层。
    pub active_overlay: Option<OverlayTarget>,
    /// 当前打开项目的名称。
    pub project_name: String,
    /// 当前打开项目的根目录绝对路径。
    pub project_root: PathBuf,
    /// 下一帧需要应用的焦点请求（仅应用层内部可写）。
    pub(crate) pending_focus_target: Option<FocusTarget>,
    /// 下一帧需要由 UI 层执行的一次性动作。
    pub(crate) pending_ui_action: Option<DesktopUiAction>,
}

impl DesktopAppState {
    /// 确保文件树存在初始选中项（用于首次获取键盘焦点前）。
    pub fn ensure_file_tree_selection(&mut self) -> bool {
        self.file_tree.ensure_selection()
    }

    /// 返回指定面板当前是否可见。
    pub fn is_panel_visible(&self, target: FocusTarget) -> bool {
        if !target.is_visibility_managed_panel() {
            return true;
        }
        self.visible_panels.contains(&target)
    }

    /// 返回指定停靠区域当前可见的面板目标。
    pub fn visible_panel_in_dock(&self, dock: PanelDock) -> Option<FocusTarget> {
        dock_targets(dock)
            .iter()
            .copied()
            .find(|target| self.is_panel_visible(*target))
    }

    /// 隐藏指定停靠区域当前可见面板。
    /// 若该面板当前持有焦点，会自动回退焦点到编辑区。
    pub fn hide_visible_panel_in_dock(&mut self, dock: PanelDock) -> bool {
        let Some(target) = self.visible_panel_in_dock(dock) else {
            return false;
        };
        self.set_panel_visible(target, false);
        if self.focused_target == target {
            self.focus_editor();
        }
        true
    }

    /// 消费一次待处理的焦点目标（供 UI 层在下一帧应用）。
    pub fn take_pending_focus_target(&mut self) -> Option<FocusTarget> {
        self.pending_focus_target.take()
    }

    /// 消费一次待处理 UI 动作（供 UI 层在下一帧应用）。
    pub fn take_pending_ui_action(&mut self) -> Option<DesktopUiAction> {
        self.pending_ui_action.take()
    }

    /// 返回当前激活编辑器的只读快照（用于渲染层）。
    pub fn active_editor_snapshot(&self) -> Option<ActiveEditorSnapshot> {
        let active_tab = self.pane.active_tab()?;
        let editor_state = self.editor_states.get(&active_tab.buffer_id)?;
        Some(ActiveEditorSnapshot {
            buffer_id: active_tab.buffer_id,
            doc_version: editor_state.version().get(),
            selection: editor_state.selection(),
            text: editor_state.text().to_string(),
        })
    }

    pub(super) fn active_buffer_id(&self) -> Option<BufferId> {
        self.pane.active_tab().map(|tab| tab.buffer_id)
    }

    pub(super) fn editor_state(&self, buffer_id: BufferId) -> Option<&EditorState> {
        self.editor_states.get(&buffer_id)
    }

    pub(super) fn replace_editor_state(&mut self, buffer_id: BufferId, next_state: EditorState) {
        self.editor_states.insert(buffer_id, next_state);
    }

    pub(super) fn take_editor_state(&mut self, buffer_id: BufferId) -> Option<EditorState> {
        self.editor_states.remove(&buffer_id)
    }

    pub(super) fn remove_editor_state(&mut self, buffer_id: BufferId) {
        self.editor_states.remove(&buffer_id);
    }

    pub(super) fn clear_editor_states(&mut self) {
        self.editor_states.clear();
    }
}
