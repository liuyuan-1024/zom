//! GPUI 响应式应用状态仓库。

use std::path::PathBuf;

use zom_protocol::{
    CommandInvocation, EditorInvocation, EditorToRuntimeEvent, FindReplaceAction,
    FindReplaceRequest, FocusTarget, Keystroke, OverlayTarget, WorkspaceAction,
};
use zom_runtime::state::{
    ActiveEditorSnapshot, DesktopAppState, DesktopToastEvent, DesktopToastLevel, DesktopUiAction,
    EditorViewportUpdate, FileTreeState, PaneState, PanelDock,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 查找替换浮层当前激活输入框。
pub(crate) enum FindReplaceField {
    Find,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// 查找替换浮层的可编辑状态：查询词、替换词以及匹配选项。
pub(crate) struct FindReplaceOverlayState {
    /// 查找关键字（literal 或 regex 模式由 `use_regex` 控制）。
    pub(crate) query: String,
    /// 替换文本（仅替换动作使用）。
    pub(crate) replacement: String,
    /// 是否区分大小写。
    pub(crate) case_sensitive: bool,
    /// 是否整词匹配。
    pub(crate) whole_word: bool,
    /// 是否按正则解释 `query`。
    pub(crate) use_regex: bool,
    /// 当前键盘输入应写入哪个字段。
    pub(crate) active_field: FindReplaceField,
}

impl Default for FindReplaceOverlayState {
    /// 查找替换浮层默认态：空查询、空替换、全部匹配开关关闭。
    fn default() -> Self {
        Self {
            query: String::new(),
            replacement: String::new(),
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            active_field: FindReplaceField::Find,
        }
    }
}

#[derive(Debug)]
/// 与运行时同步的核心状态容器，持有唯一的 `DesktopAppState`。
pub(crate) struct CoreState {
    pub(crate) app: DesktopAppState,
}

#[derive(Debug, Default)]
/// 仅供 UI 层使用的短生命周期状态，不写回核心运行时。
pub(crate) struct UiEphemeralState {
    pub(crate) find_replace: FindReplaceOverlayState,
    pub(crate) pending_toast_auto_clear_id: Option<u64>,
    pub(crate) pending_editor_event: Option<EditorToRuntimeEvent>,
}

#[derive(Debug, Clone)]
/// 查找替换浮层的 UI 层动作。
pub(crate) enum FindReplaceUiAction {
    OpenOverlay,
    CycleField,
    Backspace,
    AppendChar(char),
    ToggleCase,
    ToggleWord,
    ToggleRegex,
    Submit(FindReplaceAction),
}

#[derive(Debug, Clone)]
/// 根视图可分发的 UI 动作集合。
pub(crate) enum UiAction {
    DispatchCommand(CommandInvocation),
    DispatchKeystroke(Keystroke),
    DispatchViewportUpdate(EditorViewportUpdate),
    FindReplace(FindReplaceUiAction),
    HidePanelInDock(PanelDock),
    ClearActiveToast,
    SwitchProject(PathBuf),
    PushUserToast {
        level: DesktopToastLevel,
        message: String,
    },
    PushDebugToast {
        level: DesktopToastLevel,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 动作分发返回值：用于把“是否消费”这类信号回传给调用方。
pub(crate) enum UiActionOutput {
    None,
    Bool(bool),
}

#[derive(Debug)]
/// 应用级状态仓库：统一承载核心状态、UI 瞬态与动作分发入口。
pub(crate) struct AppStore {
    core: CoreState,
    ui: UiEphemeralState,
}

impl AppStore {
    /// 创建 UI Store，绑定核心应用状态与临时 UI 态容器。
    /// 初始时不会产生副作用，仅完成状态装配。
    pub(crate) fn new(core: DesktopAppState) -> Self {
        Self {
            core: CoreState { app: core },
            ui: UiEphemeralState::default(),
        }
    }

    /// 分发 UI 动作到核心状态机或本地瞬态状态。
    ///
    /// 该入口是根视图唯一写路径，便于统一副作用审计和调试。
    pub(crate) fn dispatch(&mut self, action: UiAction) -> UiActionOutput {
        match action {
            UiAction::DispatchCommand(command) => {
                self.core.app.dispatch_command(command);
                self.sync_pending_editor_event();
                UiActionOutput::None
            }
            UiAction::DispatchKeystroke(keystroke) => {
                let handled = self.core.app.dispatch_keystroke(&keystroke);
                self.sync_pending_editor_event();
                UiActionOutput::Bool(handled)
            }
            UiAction::DispatchViewportUpdate(update) => {
                let emitted = self.core.app.dispatch_active_editor_viewport_update(update);
                self.sync_pending_editor_event();
                UiActionOutput::Bool(emitted)
            }
            UiAction::FindReplace(action) => {
                self.dispatch_find_replace_action(action);
                UiActionOutput::None
            }
            UiAction::HidePanelInDock(dock) => {
                UiActionOutput::Bool(self.core.app.hide_visible_panel_in_dock(dock))
            }
            UiAction::ClearActiveToast => {
                self.core.app.clear_active_toast();
                UiActionOutput::None
            }
            UiAction::SwitchProject(project_root) => {
                self.core.app.switch_project(project_root);
                self.core.app.dispatch_command(CommandInvocation::from(
                    WorkspaceAction::FocusPanel(FocusTarget::Editor),
                ));
                UiActionOutput::None
            }
            UiAction::PushUserToast { level, message } => {
                self.push_user_toast(level, message);
                UiActionOutput::None
            }
            UiAction::PushDebugToast { level, message } => {
                self.push_debug_toast(level, message);
                UiActionOutput::None
            }
        }
    }

    /// 返回核心应用状态只读引用，供视图层按需派生展示数据。
    /// 调用方不得直接修改该引用指向的内部状态。
    pub(crate) fn select_core(&self) -> &DesktopAppState {
        &self.core.app
    }

    /// 返回用于根框架渲染的整页状态快照副本。
    pub(crate) fn select_root_chrome_state(&self) -> DesktopAppState {
        self.core.app.clone()
    }

    /// 返回文件树状态快照副本，供面板渲染读取。
    pub(crate) fn select_file_tree_state(&self) -> FileTreeState {
        self.core.app.file_tree.clone()
    }

    /// 返回窗格状态快照副本（标签页列表与活动索引）。
    pub(crate) fn select_pane_state(&self) -> PaneState {
        self.core.app.pane.clone()
    }

    /// 返回活动编辑器快照；无活动标签时返回 `None`。
    pub(crate) fn select_active_editor_snapshot(&self) -> Option<ActiveEditorSnapshot> {
        self.core.app.active_editor_snapshot()
    }

    /// 查询某面板当前是否可见（用于按钮高亮与布局分支）。
    pub(crate) fn select_is_panel_visible(&self, target: FocusTarget) -> bool {
        self.core.app.is_panel_visible(target)
    }

    /// 返回当前焦点目标，供键盘与样式系统统一判定焦点归属。
    /// 该查询是只读操作，不改变任何状态。
    pub(crate) fn select_focused_target(&self) -> FocusTarget {
        self.core.app.focused_target
    }

    /// 查询当前激活浮层；无浮层时返回 `None`。
    pub(crate) fn select_active_overlay(&self) -> Option<OverlayTarget> {
        self.core.app.active_overlay
    }

    /// 查询查找替换瞬态状态（不持久化到 runtime）。
    pub(crate) fn select_find_replace_overlay(&self) -> &FindReplaceOverlayState {
        &self.ui.find_replace
    }

    /// 取出待消费的编辑器事件（若存在）。
    pub(crate) fn take_pending_editor_event(&mut self) -> Option<EditorToRuntimeEvent> {
        self.ui.pending_editor_event.take()
    }

    /// 查询指定停靠区当前可见面板（若有）。
    pub(crate) fn select_visible_panel_in_dock(&self, dock: PanelDock) -> Option<FocusTarget> {
        self.core.app.visible_panel_in_dock(dock)
    }

    /// 取出 `take_pending_focus_target` 结果，并清理内部暂存状态。
    pub(crate) fn take_pending_focus_target(&mut self) -> Option<FocusTarget> {
        self.core.app.take_pending_focus_target()
    }

    /// 取出 `take_pending_ui_action` 结果，并清理内部暂存状态。
    pub(crate) fn take_pending_ui_action(&mut self) -> Option<DesktopUiAction> {
        self.core.app.take_pending_ui_action()
    }

    /// 取出 `take_pending_toast_auto_clear_id` 结果，并清理内部暂存状态。
    pub(crate) fn take_pending_toast_auto_clear_id(&mut self) -> Option<u64> {
        self.ui.pending_toast_auto_clear_id.take()
    }

    /// 执行查找替换浮层动作。
    ///
    /// 仅操作 UI 瞬态字段；真正文档查找/替换通过 `Submit` 下发到核心命令层。
    fn dispatch_find_replace_action(&mut self, action: FindReplaceUiAction) {
        match action {
            FindReplaceUiAction::OpenOverlay => {
                self.ui.find_replace.active_field = FindReplaceField::Find;
                self.core.app.dispatch_command(CommandInvocation::from(
                    WorkspaceAction::FocusOverlay(OverlayTarget::FindReplace),
                ));
            }
            FindReplaceUiAction::CycleField => {
                self.ui.find_replace.active_field = match self.ui.find_replace.active_field {
                    FindReplaceField::Find => FindReplaceField::Replace,
                    FindReplaceField::Replace => FindReplaceField::Find,
                };
            }
            FindReplaceUiAction::Backspace => match self.ui.find_replace.active_field {
                FindReplaceField::Find => {
                    self.ui.find_replace.query.pop();
                }
                FindReplaceField::Replace => {
                    self.ui.find_replace.replacement.pop();
                }
            },
            FindReplaceUiAction::AppendChar(ch) => match self.ui.find_replace.active_field {
                FindReplaceField::Find => self.ui.find_replace.query.push(ch),
                FindReplaceField::Replace => self.ui.find_replace.replacement.push(ch),
            },
            FindReplaceUiAction::ToggleCase => {
                self.ui.find_replace.case_sensitive = !self.ui.find_replace.case_sensitive;
            }
            FindReplaceUiAction::ToggleWord => {
                self.ui.find_replace.whole_word = !self.ui.find_replace.whole_word;
            }
            FindReplaceUiAction::ToggleRegex => {
                self.ui.find_replace.use_regex = !self.ui.find_replace.use_regex;
            }
            FindReplaceUiAction::Submit(action) => {
                let request = FindReplaceRequest::new(
                    self.ui.find_replace.query.clone(),
                    self.ui.find_replace.replacement.clone(),
                    action,
                    self.ui.find_replace.case_sensitive,
                    self.ui.find_replace.whole_word,
                    self.ui.find_replace.use_regex,
                );
                self.core.app.dispatch_command(CommandInvocation::from(
                    EditorInvocation::find_replace(request),
                ));
            }
        }
    }

    /// 追加“用户触发”的toast，并记录待自动清除的 toast id。
    ///
    /// 用户toast会打上 `is_user_initiated`，以便 runtime 决定是否弹 toast。
    fn push_user_toast(&mut self, level: DesktopToastLevel, message: impl Into<String>) {
        let message = message.into();
        let event = DesktopToastEvent::new(level, message).is_user_initiated();
        let toast_id = self.core.app.publish_toast_event(event);
        self.ui.pending_toast_auto_clear_id = toast_id;
    }

    /// 追加调试toast（通常不自动弹窗），用于快捷键链路诊断。
    fn push_debug_toast(&mut self, level: DesktopToastLevel, message: impl Into<String>) {
        let message = message.into();
        let event = DesktopToastEvent::new(level, message);
        let toast_id = self.core.app.publish_toast_event(event);
        self.ui.pending_toast_auto_clear_id = toast_id;
    }

    fn sync_pending_editor_event(&mut self) {
        let events = self.core.app.take_pending_editor_events();
        self.ui.pending_editor_event = events.last().cloned();
    }
}

#[cfg(test)]
mod tests {
    use super::{AppStore, FindReplaceField, FindReplaceUiAction, UiAction, UiActionOutput};
    use zom_protocol::{EditorToRuntimeEvent, FocusTarget};
    use zom_runtime::state::{
        DesktopAppState, DesktopToastLevel, EditorViewportMutation, EditorViewportUpdate,
        FileTreeNodeKind, PanelDock,
    };

    fn make_store() -> AppStore {
        AppStore::new(DesktopAppState::from_current_workspace())
    }

    #[test]
    /// 查找替换 UI 动作只应修改瞬态字段，不直接改 runtime 核心状态。
    fn find_replace_ui_actions_only_mutate_ephemeral_state() {
        let mut store = make_store();
        store.dispatch(UiAction::FindReplace(FindReplaceUiAction::CycleField));
        store.dispatch(UiAction::FindReplace(FindReplaceUiAction::AppendChar('x')));
        store.dispatch(UiAction::FindReplace(FindReplaceUiAction::ToggleCase));
        store.dispatch(UiAction::FindReplace(FindReplaceUiAction::ToggleWord));
        store.dispatch(UiAction::FindReplace(FindReplaceUiAction::ToggleRegex));
        store.dispatch(UiAction::FindReplace(FindReplaceUiAction::Backspace));

        let overlay = store.select_find_replace_overlay();
        assert_eq!(overlay.active_field, FindReplaceField::Replace);
        assert_eq!(overlay.replacement, "");
        assert!(overlay.case_sensitive);
        assert!(overlay.whole_word);
        assert!(overlay.use_regex);
    }

    #[test]
    /// 切换项目后应请求焦点回到编辑器，保持后续键盘输入一致性。
    fn switch_project_dispatch_sets_pending_editor_focus() {
        let mut store = make_store();
        let project_root = std::env::temp_dir().join("zom-gpui-store-test-project");
        std::fs::create_dir_all(&project_root).expect("create temp project root");

        store.dispatch(UiAction::SwitchProject(project_root.clone()));

        let expected_root =
            std::fs::canonicalize(project_root).expect("canonicalize expected root");
        let actual_root = std::fs::canonicalize(&store.select_core().project_root)
            .expect("canonicalize actual project root");
        assert_eq!(actual_root, expected_root);
        assert_eq!(store.take_pending_focus_target(), Some(FocusTarget::Editor));
    }

    #[test]
    /// 用户toast应触发 toast 自动清理标记。
    fn push_user_toast_sets_toast_pending_marker() {
        let mut store = make_store();
        store.dispatch(UiAction::PushUserToast {
            level: DesktopToastLevel::Info,
            message: "hello".to_string(),
        });

        assert!(store.select_core().active_toast.is_some());
        assert!(store.take_pending_toast_auto_clear_id().is_some());
    }

    #[test]
    /// 隐藏停靠区动作应返回布尔结果，供调用方判断是否真正执行了隐藏。
    fn hide_panel_action_returns_bool_result() {
        let mut store = make_store();
        let output = store.dispatch(UiAction::HidePanelInDock(PanelDock::Left));
        assert!(matches!(output, UiActionOutput::Bool(_)));
    }

    #[test]
    fn viewport_update_action_emits_runtime_event_snapshot() {
        let project_root = std::env::temp_dir().join("zom-gpui-store-viewport-project");
        std::fs::create_dir_all(&project_root).expect("create temp project root");
        std::fs::write(project_root.join("main.rs"), "a\nb\nc\n").expect("write main.rs");
        let mut state = DesktopAppState::from_current_workspace();
        state.switch_project(project_root.clone());
        state.activate_file_tree_node("main.rs", FileTreeNodeKind::File);
        assert!(state.active_editor_snapshot().is_some());

        let mut store = AppStore::new(state);

        let output = store.dispatch(UiAction::DispatchViewportUpdate(EditorViewportUpdate::new(
            0,
            2,
            80,
            EditorViewportMutation::Scroll,
        )));
        assert!(matches!(output, UiActionOutput::Bool(true)));
        assert!(matches!(
            store.take_pending_editor_event(),
            Some(EditorToRuntimeEvent::ViewportInvalidated { .. })
        ));

        let _ = std::fs::remove_dir_all(project_root);
    }
}
