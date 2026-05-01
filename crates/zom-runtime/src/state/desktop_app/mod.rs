//! 桌面应用状态与命令分发主状态机。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use zom_editor::{
    EditorState, ViewportModel as EditorViewportModel, ViewportMutation, ViewportUpdate,
};
use zom_protocol::{
    BufferId, DocumentVersion, EditorToRuntimeEvent, FocusTarget, LineRange, OverlayTarget,
    Selection, ViewportInvalidationReason, ViewportState,
};

use crate::state::{
    FileTreeState, PaneState, PanelDock, TitleBarState, ToolBarState, dock_targets,
};

mod command;
mod editor_command;
mod focus;
mod history;
mod project;
mod tabs;
mod toast;

#[cfg(test)]
mod tests;

/// 需要在 UI 层执行的副作用动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopUiAction {
    /// 退出应用。
    QuitApp,
    /// 最小化当前窗口。
    MinimizeWindow,
    /// 打开项目目录选择器。
    OpenProjectPicker,
    /// 打开查找替换入口。
    OpenFindReplace,
    /// 将文本写入系统剪贴板。
    WriteClipboard(String),
    /// 从系统剪贴板读取文本并执行粘贴。
    PasteFromClipboard,
}

/// 应用内 toast 等级。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopToastLevel {
    /// 常规提示信息。
    Info,
    /// 警告提示信息。
    Warning,
    /// 错误提示信息。
    Error,
}

/// 一次待分发的 toast 事件输入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopToastEvent {
    /// toast 等级。
    pub level: DesktopToastLevel,
    /// 提示文案。
    pub message: String,
    /// 是否属于“用户主动触发动作”的反馈。
    pub is_user_initiated: bool,
}

impl DesktopToastEvent {
    /// 创建一条常规 toast 事件。
    pub fn new(level: DesktopToastLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            is_user_initiated: false,
        }
    }

    /// 标记该事件为用户主动触发反馈。
    pub fn is_user_initiated(mut self) -> Self {
        self.is_user_initiated = true;
        self
    }
}

/// 当前 toast 展示使用的实体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopToast {
    /// toast id（按写入顺序递增）。
    pub id: u64,
    /// toast 等级。
    pub level: DesktopToastLevel,
    /// 提示文案。
    pub message: String,
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
    ///
    /// 这是渲染快照，不应在 UI 层原地修改后再写回状态机。
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorViewportMutation {
    Scroll,
    Resize,
    WrapWidthChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorViewportUpdate {
    pub first_visible_line: u32,
    pub visible_line_count: u32,
    pub wrap_column: u32,
    pub mutation: EditorViewportMutation,
}

impl EditorViewportUpdate {
    pub fn new(
        first_visible_line: u32,
        visible_line_count: u32,
        wrap_column: u32,
        mutation: EditorViewportMutation,
    ) -> Self {
        Self {
            first_visible_line,
            visible_line_count: visible_line_count.max(1),
            wrap_column: wrap_column.max(1),
            mutation,
        }
    }
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
    /// 中央窗格状态（标签页集合与活动索引）。
    pub pane: PaneState,
    /// 活跃标签页对应的编辑器状态仓库（key = buffer id）。
    ///
    /// 与 `pane.tabs` 通过 `buffer_id` 关联，二者需保持引用一致。
    pub(crate) editor_states: HashMap<BufferId, EditorState>,
    /// 活跃标签页对应的编辑历史仓库（key = buffer id）。
    pub(crate) editor_histories: HashMap<BufferId, history::EditorHistory>,
    /// 每个 buffer 的视口状态模型（用于产生 ViewportInvalidated 事件）。
    pub(crate) viewport_models: HashMap<BufferId, EditorViewportModel>,
    /// 待上层消费的编辑器事件（当前包含 viewport invalidation）。
    pub(crate) pending_editor_events: Vec<EditorToRuntimeEvent>,
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
    /// 当前展示中的悬浮提示（toast）。
    pub active_toast: Option<DesktopToast>,
    /// 下一帧需要应用的焦点请求（仅应用层内部可写）。
    pub(crate) pending_focus_target: Option<FocusTarget>,
    /// 下一帧需要由 UI 层执行的一次性动作。
    pub(crate) pending_ui_action: Option<DesktopUiAction>,
    /// 下一条 toast id（单调递增）。
    pub(crate) next_toast_id: u64,
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
    ///
    /// 若该面板当前持有焦点，会自动把焦点回退到编辑器，避免焦点悬空。
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

    /// 消费本帧累计的编辑器事件。
    pub fn take_pending_editor_events(&mut self) -> Vec<EditorToRuntimeEvent> {
        std::mem::take(&mut self.pending_editor_events)
    }

    /// 返回当前激活编辑器的只读快照（用于渲染层）。
    ///
    /// 若标签页索引损坏或状态缺失，返回 `None` 让上层回退空态展示。
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

    /// 提取缓冲区结果；找不到时返回 `None`。
    pub(super) fn active_buffer_id(&self) -> Option<BufferId> {
        self.pane.active_tab().map(|tab| tab.buffer_id)
    }

    /// 提取状态结果；找不到时返回 `None`。
    pub(super) fn editor_state(&self, buffer_id: BufferId) -> Option<&EditorState> {
        self.editor_states.get(&buffer_id)
    }

    /// 用新的编辑器状态替换当前活动标签的编辑快照。
    pub(super) fn replace_editor_state(&mut self, buffer_id: BufferId, next_state: EditorState) {
        self.editor_states.insert(buffer_id, next_state);
    }

    /// 取出 `take_editor_state` 结果，并清理内部暂存状态。
    pub(super) fn take_editor_state(&mut self, buffer_id: BufferId) -> Option<EditorState> {
        self.editor_states.remove(&buffer_id)
    }

    /// 移除编辑器状态并同步相关状态。
    pub(super) fn remove_editor_state(&mut self, buffer_id: BufferId) {
        self.editor_states.remove(&buffer_id);
        self.editor_histories.remove(&buffer_id);
        self.viewport_models.remove(&buffer_id);
    }

    /// 清理编辑器并同步相关状态。
    pub(super) fn clear_editor_states(&mut self) {
        self.editor_states.clear();
        self.editor_histories.clear();
        self.viewport_models.clear();
        self.pending_editor_events.clear();
    }

    /// 根据当前活动编辑器状态更新视口，并在变化时生成协议事件。
    pub fn dispatch_active_editor_viewport_update(&mut self, update: EditorViewportUpdate) -> bool {
        let Some(buffer_id) = self.active_buffer_id() else {
            return false;
        };
        let Some(editor_state) = self.editor_states.get(&buffer_id) else {
            return false;
        };

        let version = DocumentVersion::from(editor_state.version().get());
        let line_count = editor_state.line_count();
        let update = ViewportUpdate::new(
            ViewportState::new(update.first_visible_line, update.visible_line_count),
            update.wrap_column,
            match update.mutation {
                EditorViewportMutation::Scroll => ViewportMutation::Scroll,
                EditorViewportMutation::Resize => ViewportMutation::Resize,
                EditorViewportMutation::WrapWidthChanged => ViewportMutation::WrapWidthChanged,
            },
        );

        let model = self
            .viewport_models
            .entry(buffer_id)
            .or_insert_with(EditorViewportModel::new);
        let event = model.apply(version, line_count, update);
        if let Some(event) = event {
            self.pending_editor_events.push(event);
            return true;
        }
        false
    }

    pub(super) fn emit_editor_events_from_state_change(
        &mut self,
        previous: &EditorState,
        next: &EditorState,
    ) {
        if previous.text() != next.text() {
            let dirty_lines = collect_dirty_lines_for_text_change(previous.text(), next.text());
            self.pending_editor_events
                .push(EditorToRuntimeEvent::ViewportInvalidated {
                    version: DocumentVersion::from(next.version().get()),
                    dirty_lines,
                    viewport: None,
                    reason: ViewportInvalidationReason::DocumentChanged,
                });
            return;
        }

        if previous.selection() != next.selection() {
            self.pending_editor_events
                .push(EditorToRuntimeEvent::ViewportInvalidated {
                    version: DocumentVersion::from(next.version().get()),
                    dirty_lines: collect_dirty_lines_for_selection(
                        previous.selection(),
                        next.selection(),
                    ),
                    viewport: None,
                    reason: ViewportInvalidationReason::SelectionChanged,
                });
        }
    }
}

fn collect_dirty_lines_for_selection(previous: Selection, next: Selection) -> Vec<LineRange> {
    let ranges = vec![
        LineRange::new(
            previous.anchor().line,
            previous.anchor().line.saturating_add(1),
        ),
        LineRange::new(
            previous.active().line,
            previous.active().line.saturating_add(1),
        ),
        LineRange::new(next.anchor().line, next.anchor().line.saturating_add(1)),
        LineRange::new(next.active().line, next.active().line.saturating_add(1)),
    ];
    merge_line_ranges(ranges)
}

fn collect_dirty_lines_for_text_change(previous_text: String, next_text: String) -> Vec<LineRange> {
    let previous_lines = previous_text.split('\n').collect::<Vec<_>>();
    let next_lines = next_text.split('\n').collect::<Vec<_>>();

    let mut common_prefix = 0usize;
    while common_prefix < previous_lines.len()
        && common_prefix < next_lines.len()
        && previous_lines[common_prefix] == next_lines[common_prefix]
    {
        common_prefix += 1;
    }

    let mut common_suffix = 0usize;
    while common_suffix < previous_lines.len().saturating_sub(common_prefix)
        && common_suffix < next_lines.len().saturating_sub(common_prefix)
        && previous_lines[previous_lines.len() - 1 - common_suffix]
            == next_lines[next_lines.len() - 1 - common_suffix]
    {
        common_suffix += 1;
    }

    let previous_end = previous_lines.len().saturating_sub(common_suffix);
    let next_end = next_lines.len().saturating_sub(common_suffix);
    let start = u32::try_from(common_prefix).unwrap_or(u32::MAX);
    let end = u32::try_from(previous_end.max(next_end)).unwrap_or(u32::MAX);
    vec![LineRange::new(start, end.max(start.saturating_add(1)))]
}

fn merge_line_ranges(mut ranges: Vec<LineRange>) -> Vec<LineRange> {
    if ranges.is_empty() {
        return ranges;
    }
    ranges.sort_by_key(|range| (range.start_line, range.end_line_exclusive));
    let mut merged: Vec<LineRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && range.start_line <= last.end_line_exclusive
        {
            last.end_line_exclusive = last.end_line_exclusive.max(range.end_line_exclusive);
            continue;
        }
        merged.push(range);
    }
    merged
}
