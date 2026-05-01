//! 根视图：仅负责窗口级副作用、快捷键入口与全局外层装配。

pub(crate) mod store;

use std::time::Duration;

use gpui::{
    App, AppContext, Application, Bounds, Context, Entity, InteractiveElement, KeyDownEvent,
    ParentElement, PathPromptOptions, Styled, Timer, TitlebarOptions, Window, WindowBounds,
    WindowOptions, px, rgb, size,
};
use zom_protocol::{
    CommandInvocation, EditorInvocation, FindReplaceAction, KeyCode, Modifiers, OverlayTarget,
};
use zom_runtime::state::{DesktopAppState, DesktopToastLevel, DesktopUiAction};

use crate::{
    assets,
    components::{
        WorkspaceView, bar::traffic_lights, settings_overlay, status_bar, title_bar, toast_overlay,
    },
    theme::{color, size},
};

use self::store::{AppStore, FindReplaceUiAction, UiAction, UiActionOutput};

/// `TOAST_AUTO_DISMISS_DELAY` 的时间参数。
const TOAST_AUTO_DISMISS_DELAY: Duration = Duration::from_secs(3);

/// 启动桌面应用入口：创建窗口、注入 Store，并挂载根视图。
/// 该流程会同时注册快捷键分发与 UI 副作用处理链路。
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

            if let Err(error) = cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Zom".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(traffic_lights::position()),
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                move |_, cx| cx.new(|cx| ZomRootView::new(state, cx)),
            ) {
                eprintln!("failed to open main window: {error:?}");
                return;
            }

            cx.activate(true);
        });
}

pub(super) struct ZomRootView {
    store: Entity<AppStore>,
    workspace_view: Entity<WorkspaceView>,
}

impl ZomRootView {
    /// 初始化根视图：创建全局 store、挂载工作区视图，并订阅状态变更触发重绘。
    pub(super) fn new(state: DesktopAppState, cx: &mut Context<Self>) -> Self {
        let store = cx.new(|_| AppStore::new(state));
        let workspace_view = cx.new(|cx| WorkspaceView::new(store.clone(), cx));

        cx.observe(&store, |_this, _, cx| {
            cx.notify();
        })
        .detach();

        Self {
            store,
            workspace_view,
        }
    }

    /// 把 runtime 产出的焦点目标真正应用到各子视图。
    ///
    /// 该步骤与状态更新解耦，避免在 runtime 层直接依赖 GPUI 具体焦点 API。
    fn apply_focus_target(
        &mut self,
        target: zom_protocol::FocusTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.workspace_view.update(cx, |workspace, cx| {
            workspace.focus_target(target, window, cx)
        });
    }

    /// 执行核心层产出的 UI 动作并落到窗口副作用（焦点、剪贴板、窗口行为等）。
    fn apply_ui_action(
        &mut self,
        action: DesktopUiAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            DesktopUiAction::QuitApp => cx.quit(),
            DesktopUiAction::MinimizeWindow => window.minimize_window(),
            DesktopUiAction::OpenProjectPicker => self.open_project_from_title_bar(window, cx),
            DesktopUiAction::OpenFindReplace => {
                self.store.update(cx, |store, cx| {
                    store.dispatch(UiAction::FindReplace(FindReplaceUiAction::OpenOverlay));
                    cx.notify();
                });
            }
            DesktopUiAction::WriteClipboard(text) => {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
            }
            DesktopUiAction::PasteFromClipboard => {
                if let Some(text) = clipboard_text(cx)
                    && !text.is_empty()
                {
                    self.store.update(cx, |store, cx| {
                        store.dispatch(UiAction::DispatchCommand(CommandInvocation::from(
                            EditorInvocation::insert_text(text),
                        )));
                        cx.notify();
                    });
                }
            }
        }
    }

    /// 为当前 toast 安排“延迟自动清除”任务。
    ///
    /// 清除前会二次核对toast id，防止新 toast 被旧定时器误删。
    fn schedule_pending_toast_auto_clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let toast_id = self
            .store
            .update(cx, |store, _cx| store.take_pending_toast_auto_clear_id());

        let Some(toast_id) = toast_id else {
            return;
        };

        let this = cx.weak_entity();
        window
            .spawn(cx, async move |cx| {
                Timer::after(TOAST_AUTO_DISMISS_DELAY).await;
                let _ = this.update(cx, |this, cx| {
                    this.store.update(cx, |store, cx| {
                        let should_clear = store
                            .select_core()
                            .active_toast
                            .as_ref()
                            .map(|toast| toast.id == toast_id)
                            .unwrap_or(false);
                        if should_clear {
                            store.dispatch(UiAction::ClearActiveToast);
                            cx.notify();
                        }
                    });
                });
            })
            .detach();
    }

    /// 处理标题栏的打开项目动作并触发项目切换流程。
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
                    let _ = this.update(cx, |this, cx| {
                        this.store.update(cx, |store, cx| {
                            store.dispatch(UiAction::PushUserToast {
                                level: DesktopToastLevel::Warning,
                                message: "Open project folder dialog failed.".to_string(),
                            });
                            cx.notify();
                        });
                    });
                    return;
                };
                let Ok(Some(paths)) = selection_result else {
                    let _ = this.update(cx, |this, cx| {
                        this.store.update(cx, |store, cx| {
                            store.dispatch(UiAction::PushUserToast {
                                level: DesktopToastLevel::Info,
                                message: "Open project folder canceled.".to_string(),
                            });
                            cx.notify();
                        });
                    });
                    return;
                };
                let Some(project_root) = paths.into_iter().next() else {
                    let _ = this.update(cx, |this, cx| {
                        this.store.update(cx, |store, cx| {
                            store.dispatch(UiAction::PushUserToast {
                                level: DesktopToastLevel::Warning,
                                message: "No project folder selected.".to_string(),
                            });
                            cx.notify();
                        });
                    });
                    return;
                };

                let _ = this.update(cx, |this, cx| {
                    this.store.update(cx, |store, cx| {
                        store.dispatch(UiAction::SwitchProject(project_root));
                        let project_name = store.select_core().project_name.clone();
                        store.dispatch(UiAction::PushUserToast {
                            level: DesktopToastLevel::Info,
                            message: format!("Opened project: {project_name}"),
                        });
                        cx.notify();
                    });
                });
            })
            .detach();
    }

    /// 键盘事件统一入口：先给 overlay 优先处理，再走全局快捷键解析。
    ///
    /// 返回 `true` 表示事件已消费，调用方应停止继续传播。
    fn dispatch_shortcut_keydown(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        if self.handle_find_replace_overlay_keydown(event, cx) {
            return true;
        }
        let Some(keystroke) = crate::input::to_core_keystroke(event) else {
            return false;
        };

        let debug_keys = std::env::var_os("ZOM_DEBUG_KEYS").is_some();
        if debug_keys {
            self.store.update(cx, |store, cx| {
                store.dispatch(UiAction::PushDebugToast {
                    level: DesktopToastLevel::Info,
                    message: format!(
                        "[zom-shortcut] key={:?} focus_before={:?}",
                        keystroke,
                        store.select_core().focused_target
                    ),
                });
                cx.notify();
            });
        }

        let handled = self.store.update(cx, |store, cx| {
            let output = store.dispatch(UiAction::DispatchKeystroke(keystroke));
            let handled = matches!(output, UiActionOutput::Bool(true));
            if handled {
                cx.notify();
            }
            handled
        });

        if !handled {
            if debug_keys {
                self.store.update(cx, |store, cx| {
                    store.dispatch(UiAction::PushDebugToast {
                        level: DesktopToastLevel::Warning,
                        message: "[zom-shortcut] ignored".to_string(),
                    });
                    cx.notify();
                });
            }
            return false;
        }

        if debug_keys {
            self.store.update(cx, |store, cx| {
                store.dispatch(UiAction::PushDebugToast {
                    level: DesktopToastLevel::Info,
                    message: format!(
                        "[zom-shortcut] handled focus_after={:?}",
                        store.select_core().focused_target
                    ),
                });
                cx.notify();
            });
        }

        true
    }

    /// 仅在查找替换浮层激活时拦截按键并更新瞬态状态。
    ///
    /// 行为优先级：Tab/Backspace/Alt 选项切换/Enter 提交/字符输入。
    fn handle_find_replace_overlay_keydown(
        &mut self,
        event: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.store.read(cx).select_active_overlay() != Some(OverlayTarget::FindReplace) {
            return false;
        }

        let Some(key) = crate::input::to_core_keystroke(event).map(|key| key.key) else {
            return false;
        };
        let modifiers = Modifiers::new(
            event.keystroke.modifiers.control,
            event.keystroke.modifiers.alt,
            event.keystroke.modifiers.shift,
            event.keystroke.modifiers.platform,
        );

        if matches!(key, KeyCode::Tab) {
            self.store.update(cx, |store, cx| {
                store.dispatch(UiAction::FindReplace(FindReplaceUiAction::CycleField));
                cx.notify();
            });
            return true;
        }

        if matches!(key, KeyCode::Backspace) && modifiers.is_empty() {
            self.store.update(cx, |store, cx| {
                store.dispatch(UiAction::FindReplace(FindReplaceUiAction::Backspace));
                cx.notify();
            });
            return true;
        }

        if modifiers.has_alt
            && !modifiers.has_ctrl
            && !modifiers.has_cmd
            && let KeyCode::Char(ch) = key
        {
            let matched = match ch {
                'c' => Some(FindReplaceUiAction::ToggleCase),
                'w' => Some(FindReplaceUiAction::ToggleWord),
                'r' => Some(FindReplaceUiAction::ToggleRegex),
                _ => None,
            };
            if let Some(action) = matched {
                self.store.update(cx, |store, cx| {
                    store.dispatch(UiAction::FindReplace(action));
                    cx.notify();
                });
                return true;
            }
        }

        if matches!(key, KeyCode::Enter) {
            let action = if modifiers.has_alt && (modifiers.has_ctrl || modifiers.has_cmd) {
                FindReplaceAction::ReplaceAll
            } else if modifiers.has_alt {
                FindReplaceAction::ReplaceNext
            } else if modifiers.has_shift {
                FindReplaceAction::FindPrev
            } else {
                FindReplaceAction::FindNext
            };
            self.store.update(cx, |store, cx| {
                store.dispatch(UiAction::FindReplace(FindReplaceUiAction::Submit(action)));
                cx.notify();
            });
            return true;
        }

        if let Some(ch) = typed_char(event, modifiers) {
            self.store.update(cx, |store, cx| {
                store.dispatch(UiAction::FindReplace(FindReplaceUiAction::AppendChar(ch)));
                cx.notify();
            });
            return true;
        }

        false
    }

    /// 渲染浮层并组装对应界面节点。
    fn render_settings_overlay(&self) -> gpui::Stateful<gpui::Div> {
        gpui::div()
            .id("settings-overlay-layer")
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .w_full()
            .h_full()
            .child(
                gpui::div()
                    .id("settings-overlay-mask")
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .w_full()
                    .h_full()
                    .bg(rgb(color::COLOR_BG_APP))
                    .opacity(0.72),
            )
            .child(
                gpui::div()
                    .id("settings-overlay-center")
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        gpui::div()
                            .id("settings-overlay-card-container")
                            .child(settings_overlay::panel()),
                    ),
            )
    }
}

impl gpui::Render for ZomRootView {
    /// 根节点渲染入口：先消费待执行 UI 动作，再按当前 overlay/toast 状态装配整页视图。
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        if let Some(action) = self
            .store
            .update(cx, |store, _cx| store.take_pending_ui_action())
        {
            self.apply_ui_action(action, window, cx);
        }

        self.schedule_pending_toast_auto_clear(window, cx);

        if let Some(target) = self
            .store
            .update(cx, |store, _cx| store.take_pending_focus_target())
        {
            self.apply_focus_target(target, window, cx);
        }

        let state = self.store.read(cx).select_root_chrome_state();
        let active_overlay = state.active_overlay;
        let toast = state.active_toast.clone();

        let mut root = gpui::div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .capture_key_down(cx.listener(|this, event, _window, cx| {
                if this.dispatch_shortcut_keydown(event, cx) {
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .on_key_down(cx.listener(|this, event, _window, cx| {
                if this.dispatch_shortcut_keydown(event, cx) {
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .bg(rgb(color::COLOR_BG_APP))
            .text_color(rgb(color::COLOR_FG_PRIMARY))
            .child(title_bar::render(&state))
            .child(self.workspace_view.clone())
            .child(status_bar::render(&state));

        if active_overlay == Some(OverlayTarget::Settings) {
            root = root.child(self.render_settings_overlay());
        }

        if let Some(toast) = toast.as_ref() {
            root = root.child(toast_overlay::layer(toast));
        }

        root
    }
}

/// 从系统剪贴板读取纯文本；无文本或读取失败时返回 `None`。
fn clipboard_text(cx: &mut Context<ZomRootView>) -> Option<String> {
    cx.read_from_clipboard()?.text()
}

/// 从键盘事件中提取可直接插入的字符；带 Ctrl/Meta/Alt 的组合键不参与文本输入。
fn typed_char(event: &KeyDownEvent, modifiers: Modifiers) -> Option<char> {
    if modifiers.has_ctrl || modifiers.has_cmd || modifiers.has_alt {
        return None;
    }
    let key = event.keystroke.key.as_str();
    if key == "space" {
        return Some(' ');
    }
    if key.chars().count() == 1 {
        return key.chars().next();
    }
    None
}
