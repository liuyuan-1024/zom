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
mod notification;
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

/// 应用内通知等级。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopNotificationLevel {
    /// 常规提示信息。
    Info,
    /// 警告提示信息。
    Warning,
    /// 错误提示信息。
    Error,
}

/// 通知事件来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopNotificationSource {
    /// 工作台行为（打开项目、面板切换等）。
    Workspace,
    /// 系统内部事件。
    System,
    /// 调试辅助事件。
    Debug,
}

/// 通知类型（用于策略路由）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopNotificationKind {
    /// 常规提示。
    General,
    /// 过程型进度。
    Progress,
}

/// 一次待分发的通知事件输入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopNotificationEvent {
    /// 通知等级。
    pub level: DesktopNotificationLevel,
    /// 事件来源。
    pub source: DesktopNotificationSource,
    /// 事件类型。
    pub kind: DesktopNotificationKind,
    /// 提示文案。
    pub message: String,
    /// 是否属于“用户主动触发动作”的反馈。
    pub user_initiated: bool,
    /// 去重键（同键短窗口内聚合）。
    pub dedupe_key: Option<String>,
}

impl DesktopNotificationEvent {
    /// 创建一条常规通知事件。
    pub fn new(
        level: DesktopNotificationLevel,
        source: DesktopNotificationSource,
        message: impl Into<String>,
    ) -> Self {
        Self {
            level,
            source,
            kind: DesktopNotificationKind::General,
            message: message.into(),
            user_initiated: false,
            dedupe_key: None,
        }
    }

    /// 标记该事件为用户主动触发反馈。
    pub fn user_initiated(mut self) -> Self {
        self.user_initiated = true;
        self
    }

    /// 为事件设置去重键。
    pub fn with_dedupe_key(mut self, dedupe_key: impl Into<String>) -> Self {
        self.dedupe_key = Some(dedupe_key.into());
        self
    }
}

/// 通知侧边栏和悬浮提示共享的通知实体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopNotification {
    /// 通知 id（按写入顺序递增）。
    pub id: u64,
    /// 通知等级。
    pub level: DesktopNotificationLevel,
    /// 事件来源。
    pub source: DesktopNotificationSource,
    /// 提示文案。
    pub message: String,
    /// 首次写入时间戳（毫秒）。
    pub created_at_ms: u128,
    /// 最近一次聚合时间戳（毫秒）。
    pub updated_at_ms: u128,
    /// 是否已读。
    pub is_read: bool,
    /// 去重键（同键在短窗口内聚合）。
    pub dedupe_key: Option<String>,
    /// 聚合计数（最小为 1）。
    pub occurrence_count: u32,
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
    /// 通知侧边栏持久化列表（按时间顺序追加）。
    pub notifications: Vec<DesktopNotification>,
    /// 当前展示中的悬浮提示（toast）。
    pub active_toast_notification: Option<DesktopNotification>,
    /// 当前状态栏展示中的通知（持续状态提示）。
    pub active_status_notification: Option<DesktopNotification>,
    /// 通知面板未读数量。
    pub unread_notification_count: usize,
    /// 通知面板当前选中的通知 id。
    pub(crate) selected_notification_id: Option<u64>,
    /// 下一帧通知面板需要选中的通知 id。
    pub(crate) pending_notification_selection_id: Option<u64>,
    /// 下一帧需要应用的焦点请求（仅应用层内部可写）。
    pub(crate) pending_focus_target: Option<FocusTarget>,
    /// 下一帧需要由 UI 层执行的一次性动作。
    pub(crate) pending_ui_action: Option<DesktopUiAction>,
    /// 下一条通知 id（单调递增）。
    pub(crate) next_notification_id: u64,
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

    /// 消费一次通知面板待选中通知 id（供 UI 层在下一帧应用）。
    pub fn take_pending_notification_selection_id(&mut self) -> Option<u64> {
        self.pending_notification_selection_id.take()
    }

    /// 返回当前激活编辑器的只读快照（用于渲染层）。
    pub fn active_editor_snapshot(&self) -> Option<ActiveEditorSnapshot> {
        let active_tab = self.pane.active_tab()?;
        let editor_state = self.editor_states.get(&active_tab.buffer_id)?;
        Some(ActiveEditorSnapshot {
            buffer_id: active_tab.buffer_id,
            doc_version: editor_state.version().get(),
            selection: editor_state.selection(),
            text: editor_state.text(),
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
