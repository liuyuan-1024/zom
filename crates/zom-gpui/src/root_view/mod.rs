//! 根视图状态编排、焦点调度与拖拽尺寸控制。

mod render;

use gpui::{
    App, AppContext, Application, Bounds, Context, Entity, KeyDownEvent, MouseMoveEvent,
    PathPromptOptions, TitlebarOptions, Window, WindowBounds, WindowOptions, px, size,
};
use zom_protocol::{CommandInvocation, FocusTarget, WorkspaceAction};
use zom_runtime::state::{DesktopAppState, DesktopUiAction, PanelDock};

use crate::{
    assets,
    components::{
        DebugPanel, FileTreePanel, GitPanel, LanguageServersPanel, NotificationPanel, OutlinePanel,
        PaneView, ProjectSearchPanel, TerminalPanel, title_bar,
    },
    theme::size,
};

pub(super) const DEFAULT_BOTTOM_PANEL_HEIGHT: f32 = 240.0;

#[derive(Debug, Clone, Copy, PartialEq)]
/// 当前激活的面板分割线拖拽类型。
pub(super) enum ActiveDockDrag {
    Left,
    Right,
    Bottom { origin_y: f32, origin_height: f32 },
}

/// 启动桌面界面。
pub fn run() {
    Application::new()
        .with_assets(assets::ZomAssets::new())
        .run(|cx: &mut App| {
            let bounds = Bounds::centered(
                None,
                size(px(size::WINDOW_WIDTH), px(size::WINDOW_HEIGHT)),
                cx,
            );
            let state = DesktopAppState::from_current_workspace();

            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Zom".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(title_bar::traffic_lights::position()),
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                move |_, cx| cx.new(|cx| ZomRootView::new(state, cx)),
            )
            .unwrap();

            cx.activate(true);
        });
}

/// 根视图，负责拼装最外层界面布局。
pub(super) struct ZomRootView {
    /// 用于展示的应用状态。
    state: DesktopAppState,
    /// 文件树
    file_tree_panel: Entity<FileTreePanel>,
    /// Git 面板
    git_panel: Entity<GitPanel>,
    /// 大纲面板
    outline_panel: Entity<OutlinePanel>,
    /// 搜索面板
    project_search_panel: Entity<ProjectSearchPanel>,
    /// 语言服务器面板
    language_servers_panel: Entity<LanguageServersPanel>,
    /// 终端面板
    terminal_panel: Entity<TerminalPanel>,
    /// 调试面板
    debug_panel: Entity<DebugPanel>,
    /// 通知面板
    notification_panel: Entity<NotificationPanel>,
    /// Pane 视图
    pane_view: Entity<PaneView>,
    /// 左侧面板列宽度。
    left_dock_width: f32,
    /// 右侧面板列宽度。
    right_dock_width: f32,
    /// 底部面板高度。
    bottom_panel_height: f32,
    /// 当前正在拖拽的停靠分割线。
    active_dock_drag: Option<ActiveDockDrag>,
}

impl ZomRootView {
    /// 用应用状态创建根视图。
    pub(super) fn new(state: DesktopAppState, cx: &mut Context<Self>) -> Self {
        let file_tree_panel = cx.new(|cx| {
            FileTreePanel::new(
                state.file_tree.clone(),
                state.focused_target == FocusTarget::FileTreePanel,
                cx,
            )
        });
        let git_panel = cx.new(GitPanel::new);
        let outline_panel = cx.new(OutlinePanel::new);
        let project_search_panel = cx.new(ProjectSearchPanel::new);
        let language_servers_panel = cx.new(LanguageServersPanel::new);
        let terminal_panel = cx.new(TerminalPanel::new);
        let debug_panel = cx.new(DebugPanel::new);
        let notification_panel = cx.new(NotificationPanel::new);
        let pane_view = cx.new(|cx| PaneView::new(state.pane.clone(), state.tool_bar.cursor, cx));

        Self {
            state,
            file_tree_panel,
            git_panel,
            outline_panel,
            project_search_panel,
            language_servers_panel,
            terminal_panel,
            debug_panel,
            notification_panel,
            pane_view,
            left_dock_width: size::PANEL_WIDTH,
            right_dock_width: size::PANEL_WIDTH,
            bottom_panel_height: DEFAULT_BOTTOM_PANEL_HEIGHT,
            active_dock_drag: None,
        }
    }

    /// 按应用层焦点目标把键盘焦点下发到对应 GPUI 视图。
    pub(super) fn apply_focus_target(
        &mut self,
        target: FocusTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match target {
            FocusTarget::FileTreePanel
                if self.state.is_panel_visible(FocusTarget::FileTreePanel) =>
            {
                cx.focus_view(&self.file_tree_panel, window);
            }
            FocusTarget::GitPanel if self.state.is_panel_visible(FocusTarget::GitPanel) => {
                cx.focus_view(&self.git_panel, window);
            }
            FocusTarget::OutlinePanel if self.state.is_panel_visible(FocusTarget::OutlinePanel) => {
                cx.focus_view(&self.outline_panel, window);
            }
            FocusTarget::ProjectSearchPanel
                if self.state.is_panel_visible(FocusTarget::ProjectSearchPanel) =>
            {
                cx.focus_view(&self.project_search_panel, window);
            }
            FocusTarget::LanguageServersPanel
                if self
                    .state
                    .is_panel_visible(FocusTarget::LanguageServersPanel) =>
            {
                cx.focus_view(&self.language_servers_panel, window);
            }
            FocusTarget::TerminalPanel
                if self.state.is_panel_visible(FocusTarget::TerminalPanel) =>
            {
                cx.focus_view(&self.terminal_panel, window);
            }
            FocusTarget::DebugPanel if self.state.is_panel_visible(FocusTarget::DebugPanel) => {
                cx.focus_view(&self.debug_panel, window);
            }
            FocusTarget::NotificationPanel
                if self.state.is_panel_visible(FocusTarget::NotificationPanel) =>
            {
                cx.focus_view(&self.notification_panel, window);
            }
            FocusTarget::Editor => {
                cx.focus_view(&self.pane_view, window);
            }
            _ => {}
        }
    }

    /// 执行应用层产生的一次性 UI 动作。
    pub(super) fn apply_ui_action(
        &mut self,
        action: DesktopUiAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            DesktopUiAction::QuitApp => cx.quit(),
            DesktopUiAction::MinimizeWindow => window.minimize_window(),
            DesktopUiAction::OpenProjectPicker => self.open_project_from_title_bar(window, cx),
        }
    }

    /// 将最新应用状态同步到文件树和窗格视图。
    pub(super) fn sync_child_views(&mut self, cx: &mut Context<Self>) {
        let file_tree_state = self.state.file_tree.clone();
        let file_tree_is_focused = self.state.focused_target == FocusTarget::FileTreePanel;
        let pane_state = self.state.pane.clone();
        let cursor = self.state.tool_bar.cursor;

        self.file_tree_panel.update(cx, |this, cx| {
            this.set_state(file_tree_state, file_tree_is_focused, cx);
        });
        self.pane_view.update(cx, |this, cx| {
            this.set_state(pane_state, cursor, cx);
        });
        cx.notify();
    }

    /// 从标题栏打开项目目录
    pub(super) fn open_project_from_title_bar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let picked_paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Open Project Folder".into()),
        });

        let this = cx.weak_entity();
        window
            .spawn(cx, async move |cx| {
                let Ok(selection_result) = picked_paths.await else {
                    return;
                };
                let Ok(Some(paths)) = selection_result else {
                    return;
                };
                let Some(project_root) = paths.into_iter().next() else {
                    return;
                };

                this.update(cx, |this, cx| {
                    this.state.switch_project(project_root);
                    this.state.handle_command(CommandInvocation::from(
                        WorkspaceAction::FocusPanel(FocusTarget::Editor),
                    ));
                    this.sync_child_views(cx);
                })
                .ok();
            })
            .detach();
    }

    /// 归一化左右面板宽度，确保宽度非负且不会越过中心保留区。
    pub(super) fn normalize_dock_widths(
        &mut self,
        workspace_width: f32,
        left_visible: bool,
        right_visible: bool,
    ) {
        self.left_dock_width = self.left_dock_width.max(0.0);
        self.right_dock_width = self.right_dock_width.max(0.0);

        let mut max_left = Self::max_dock_width(workspace_width);
        let mut max_right = Self::max_dock_width(workspace_width);
        if left_visible && right_visible {
            max_left = (workspace_width - self.right_dock_width - dock_gap()).max(0.0);
            max_right = (workspace_width - self.left_dock_width - dock_gap()).max(0.0);
        }
        self.left_dock_width = self.left_dock_width.min(max_left);
        self.right_dock_width = self.right_dock_width.min(max_right);
    }

    /// 归一化底部面板高度，保证高度在合法拖拽区间内。
    pub(super) fn normalize_bottom_panel_height(&mut self, workspace_height: f32) {
        self.bottom_panel_height = self
            .bottom_panel_height
            .clamp(0.0, Self::max_bottom_panel_height(workspace_height));
    }

    /// 根据鼠标位置更新左侧面板宽度，并保持与右侧面板不重叠。
    pub(super) fn update_left_dock_width_from_cursor(
        &mut self,
        cursor_x: f32,
        workspace_width: f32,
        right_visible: bool,
    ) {
        let max_left = if right_visible {
            (workspace_width - self.right_dock_width - dock_gap()).max(0.0)
        } else {
            Self::max_dock_width(workspace_width)
        };
        self.left_dock_width = cursor_x.clamp(0.0, max_left);
    }

    /// 根据鼠标位置更新右侧面板宽度，并保持与左侧面板不重叠。
    pub(super) fn update_right_dock_width_from_cursor(
        &mut self,
        cursor_x: f32,
        workspace_width: f32,
        left_visible: bool,
    ) {
        let max_right = if left_visible {
            (workspace_width - self.left_dock_width - dock_gap()).max(0.0)
        } else {
            Self::max_dock_width(workspace_width)
        };
        let raw_width = workspace_width - cursor_x;
        self.right_dock_width = raw_width.clamp(0.0, max_right);
    }

    /// 根据窗口可视高度计算工作区高度（剔除上下 bar 占用）。
    pub(super) fn workspace_height_from_viewport(viewport_height: f32) -> f32 {
        // root 结构是: title bar + workspace + tool bar
        (viewport_height - size::BAR_HEIGHT * 2.0).max(0.0)
    }

    /// 计算单侧面板的最大宽度，始终给边界保留一个可拖拽 gap。
    fn max_dock_width(workspace_width: f32) -> f32 {
        (workspace_width - dock_gap()).max(0.0)
    }

    /// 计算底部面板与上边界之间的最小保留间距。
    fn bottom_max_gap() -> f32 {
        dock_gap() + size::GAP_1
    }

    /// 计算底部面板的最大高度，保证顶部始终留有拖拽与识别空间。
    fn max_bottom_panel_height(workspace_height: f32) -> f32 {
        (workspace_height - Self::bottom_max_gap()).max(0.0)
    }

    /// 计算“触发隐藏面板”时使用的边界命中阈值。
    fn hide_boundary_threshold_px() -> f32 {
        splitter_hit_size() * 0.5
    }

    /// 判断当前拖拽是否到达隐藏边界（左/右贴边或底部收至阈值）。
    fn reached_hide_boundary(&self, event: &MouseMoveEvent, workspace_width: f32) -> bool {
        let edge = Self::hide_boundary_threshold_px();
        match self.active_dock_drag {
            Some(ActiveDockDrag::Left) => {
                let cursor_x: f32 = event.position.x.into();
                cursor_x <= edge
            }
            Some(ActiveDockDrag::Right) => {
                let cursor_x: f32 = event.position.x.into();
                cursor_x >= (workspace_width - edge)
            }
            Some(ActiveDockDrag::Bottom {
                origin_y,
                origin_height,
            }) => {
                let cursor_y: f32 = event.position.y.into();
                let delta = cursor_y - origin_y;
                let next_height = (origin_height - delta).max(0.0);
                next_height <= edge
            }
            None => false,
        }
    }

    /// 隐藏当前正在拖拽所属 dock 的可见面板，并清空对应尺寸。
    fn hide_active_dock_panel(&mut self) {
        match self.active_dock_drag {
            Some(ActiveDockDrag::Left) => {
                self.left_dock_width = 0.0;
                self.state.hide_visible_panel_in_dock(PanelDock::Left);
            }
            Some(ActiveDockDrag::Right) => {
                self.right_dock_width = 0.0;
                self.state.hide_visible_panel_in_dock(PanelDock::Right);
            }
            Some(ActiveDockDrag::Bottom { .. }) => {
                self.bottom_panel_height = 0.0;
                self.state.hide_visible_panel_in_dock(PanelDock::Bottom);
            }
            None => {}
        }
    }

    /// 处理 dock 分割线拖拽过程，更新尺寸并处理贴边隐藏。
    pub(super) fn on_drag_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        workspace_width: f32,
        workspace_height: f32,
        cx: &mut Context<Self>,
    ) {
        if self.active_dock_drag.is_none() {
            return;
        }

        if self.reached_hide_boundary(event, workspace_width) {
            self.hide_active_dock_panel();
            self.active_dock_drag = None;
            cx.notify();
            return;
        }

        // If the mouse button was released outside the app window, GPUI may miss MouseUp.
        // As soon as we receive a non-dragging move again, end the dock drag state.
        if !event.dragging() {
            self.active_dock_drag = None;
            cx.notify();
            return;
        }

        match self.active_dock_drag {
            Some(ActiveDockDrag::Left) => {
                let cursor_x: f32 = event.position.x.into();
                let right_visible = self.state.visible_panel_in_dock(PanelDock::Right).is_some();
                self.update_left_dock_width_from_cursor(cursor_x, workspace_width, right_visible);
            }
            Some(ActiveDockDrag::Right) => {
                let cursor_x: f32 = event.position.x.into();
                let left_visible = self.state.visible_panel_in_dock(PanelDock::Left).is_some();
                self.update_right_dock_width_from_cursor(cursor_x, workspace_width, left_visible);
            }
            Some(ActiveDockDrag::Bottom {
                origin_y,
                origin_height,
            }) => {
                let cursor_y: f32 = event.position.y.into();
                let delta = cursor_y - origin_y;
                let next_height = (origin_height - delta).max(0.0);
                self.bottom_panel_height =
                    next_height.min(Self::max_bottom_panel_height(workspace_height));
            }
            None => {}
        }
        cx.notify();
    }

    /// 处理快捷键按下事件并委派给应用层命令系统。
    pub(super) fn handle_shortcut_keydown(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(keystroke) = crate::input::to_core_keystroke(event) else {
            return false;
        };
        // debug时才会出发
        // TODO：后期改成悬浮提示框
        let debug_keys = std::env::var_os("ZOM_DEBUG_KEYS").is_some();
        if debug_keys {
            eprintln!(
                "[zom-shortcut] key={:?} focus_before={:?}",
                keystroke, self.state.focused_target
            );
        }
        if !self.state.handle_keystroke(&keystroke) {
            if debug_keys {
                eprintln!("[zom-shortcut] ignored");
            }
            return false;
        }
        self.sync_child_views(cx);
        if debug_keys {
            eprintln!(
                "[zom-shortcut] handled focus_after={:?}",
                self.state.focused_target
            );
        }
        true
    }
}

/// 返回布局层统一使用的 dock 最小间隔。
pub(super) fn dock_gap() -> f32 {
    size::GAP_1
}

/// 返回分割线的可命中热区尺寸。
pub(super) fn splitter_hit_size() -> f32 {
    size::GAP_1
}
