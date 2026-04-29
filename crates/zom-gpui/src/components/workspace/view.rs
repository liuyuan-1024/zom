//! 工作区主布局实体：承载 panel/pane 组合与分割线拖拽交互。

use gpui::{AppContext, Context, Entity, Render, Window};
use zom_protocol::FocusTarget;

use crate::{
    components::{
        DebugPanel, FileTreePanel, GitPanel, LanguageServersPanel, NotificationPanel, OutlinePanel,
        PaneView, ProjectSearchPanel, ShortcutPanel, TerminalPanel,
    },
    root_view::store::{AppStore, UiAction},
};

#[derive(Debug, Clone, Copy, PartialEq)]
/// 定义 `ActiveDockDrag` 的枚举分支，用于表达离散状态或动作。
pub(super) enum ActiveDockDrag {
    Left,
    Right,
    Bottom { origin_y: f32, origin_height: f32 },
}

/// 工作区根视图状态，统一管理各停靠面板、主窗格及分割拖拽尺寸。
pub(crate) struct WorkspaceView {
    pub(super) store: Entity<AppStore>,
    pub(super) file_tree_panel: Entity<FileTreePanel>,
    pub(super) git_panel: Entity<GitPanel>,
    pub(super) outline_panel: Entity<OutlinePanel>,
    pub(super) project_search_panel: Entity<ProjectSearchPanel>,
    pub(super) language_servers_panel: Entity<LanguageServersPanel>,
    pub(super) terminal_panel: Entity<TerminalPanel>,
    pub(super) debug_panel: Entity<DebugPanel>,
    pub(super) notification_panel: Entity<NotificationPanel>,
    pub(super) shortcut_panel: Entity<ShortcutPanel>,
    pub(super) pane_view: Entity<PaneView>,
    pub(super) left_dock_width: f32,
    pub(super) right_dock_width: f32,
    pub(super) bottom_panel_height: f32,
    pub(super) active_dock_drag: Option<ActiveDockDrag>,
}

impl WorkspaceView {
    /// 创建工作区视图并初始化各个 dock 面板实体。
    /// 同时订阅全局 store，保证布局与面板可见性在状态变更后及时刷新。
    pub(crate) fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        cx.observe(&store, |_this, _, cx| {
            cx.notify();
        })
        .detach();

        let file_tree_panel = cx.new(|cx| FileTreePanel::new(store.clone(), cx));
        let git_panel = cx.new(GitPanel::new);
        let outline_panel = cx.new(OutlinePanel::new);
        let project_search_panel = cx.new(ProjectSearchPanel::new);
        let language_servers_panel = cx.new(LanguageServersPanel::new);
        let terminal_panel = cx.new(TerminalPanel::new);
        let debug_panel = cx.new(DebugPanel::new);
        let notification_panel = cx.new(|cx| NotificationPanel::new(store.clone(), cx));
        let shortcut_panel = cx.new(ShortcutPanel::new);
        let pane_view = cx.new(|cx| PaneView::new(store.clone(), cx));

        Self {
            store,
            file_tree_panel,
            git_panel,
            outline_panel,
            project_search_panel,
            language_servers_panel,
            terminal_panel,
            debug_panel,
            notification_panel,
            shortcut_panel,
            pane_view,
            left_dock_width: crate::theme::size::PANEL_WIDTH,
            right_dock_width: crate::theme::size::PANEL_WIDTH,
            bottom_panel_height: 240.0,
            active_dock_drag: None,
        }
    }

    /// 根据目标类型切换焦点并同步界面状态。
    pub(crate) fn focus_target(
        &mut self,
        target: FocusTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let is_visible = self.store.read(cx).select_is_panel_visible(target);
        match target {
            FocusTarget::FileTreePanel if is_visible => {
                cx.focus_view(&self.file_tree_panel, window)
            }
            FocusTarget::GitPanel if is_visible => cx.focus_view(&self.git_panel, window),
            FocusTarget::OutlinePanel if is_visible => cx.focus_view(&self.outline_panel, window),
            FocusTarget::ProjectSearchPanel if is_visible => {
                cx.focus_view(&self.project_search_panel, window)
            }
            FocusTarget::LanguageServersPanel if is_visible => {
                cx.focus_view(&self.language_servers_panel, window)
            }
            FocusTarget::TerminalPanel if is_visible => cx.focus_view(&self.terminal_panel, window),
            FocusTarget::DebugPanel if is_visible => cx.focus_view(&self.debug_panel, window),
            FocusTarget::NotificationPanel if is_visible => {
                self.store.update(cx, |store, cx| {
                    store.dispatch(UiAction::ClearActiveToast);
                    cx.notify();
                });
                cx.focus_view(&self.notification_panel, window);
            }
            FocusTarget::ShortcutPanel if is_visible => cx.focus_view(&self.shortcut_panel, window),
            FocusTarget::Editor => {
                self.pane_view
                    .update(cx, |pane, cx| pane.focus_editor(window, cx));
            }
            _ => {}
        }
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.render_workspace(window, cx)
    }
}
