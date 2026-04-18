//! `zom-gpui` 负责把应用状态渲染成桌面界面。
//! 当前阶段先提供一个最小可运行的编辑器壳子。

mod assets;
mod chrome;
mod components;
mod input;
mod theme;
use components::{file_tree::FileTreePanel, title_bar, tool_bar};

use gpui::{
    App, Application, Bounds, Context, Entity, InteractiveElement, ParentElement,
    PathPromptOptions, Render, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use zom_app::state::DesktopAppState;
use zom_core::{Command, FocusTarget, command::WorkspaceCommand};

use crate::{
    components::{pane::PaneView, title_bar::traffic_lights},
    theme::{color, size},
};

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
                        traffic_light_position: Some(traffic_lights::position()),
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
pub struct ZomRootView {
    /// 用于展示的应用状态。
    state: DesktopAppState,
    /// 文件树
    file_tree_panel: Entity<FileTreePanel>,
    /// Pane 视图
    pane_view: Entity<PaneView>,
}

impl ZomRootView {
    /// 用应用状态创建根视图。
    pub fn new(state: DesktopAppState, cx: &mut Context<Self>) -> Self {
        let file_tree_panel = cx.new(|cx| FileTreePanel::new(state.file_tree.clone(), cx));
        let pane_view = cx.new(|cx| PaneView::new(state.pane.clone(), cx));

        Self {
            state,
            file_tree_panel,
            pane_view,
        }
    }

    fn apply_focus_target(
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
            FocusTarget::Editor => {
                cx.focus_view(&self.pane_view, window);
            }
            _ => {}
        }
    }

    /// 将最新应用状态同步到文件树和窗格视图。
    fn sync_child_views(&mut self, cx: &mut Context<Self>) {
        let file_tree_state = self.state.file_tree.clone();
        let pane_state = self.state.pane.clone();

        self.file_tree_panel.update(cx, |this, cx| {
            this.set_state(file_tree_state, cx);
        });
        self.pane_view.update(cx, |this, cx| {
            this.set_state(pane_state, cx);
        });
        cx.notify();
    }

    /// 从标题栏打开项目目录
    fn open_project_from_title_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
                    this.state
                        .handle_command(Command::from(WorkspaceCommand::FocusPanel(
                            FocusTarget::Editor,
                        )));
                    this.sync_child_views(cx);
                })
                .ok();
            })
            .detach();
    }
}

impl Render for ZomRootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(target) = self.state.take_pending_focus_target() {
            self.apply_focus_target(target, window, cx);
        }

        let workspace_row = {
            let row = div().flex().flex_1().overflow_hidden();
            let row = if self.state.is_panel_visible(FocusTarget::FileTreePanel) {
                row.child(self.file_tree_panel.clone())
            } else {
                row
            };
            row.child(self.pane_view.clone())
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .on_key_down(cx.listener(|this, event, _window, cx| {
                let Some(keystroke) = input::to_core_keystroke(event) else {
                    return;
                };
                if !this.state.handle_keystroke(&keystroke) {
                    return;
                }
                this.sync_child_views(cx);
                cx.stop_propagation();
                cx.notify();
            }))
            .bg(rgb(color::COLOR_BG_APP))
            .text_color(rgb(color::COLOR_FG_PRIMARY))
            .child(title_bar::render(
                &self.state,
                cx.listener(|this, _event, window, cx| {
                    this.open_project_from_title_bar(window, cx);
                }),
            ))
            .child(workspace_row)
            .child(tool_bar::render(&self.state))
    }
}
