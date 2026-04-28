//! 根视图渲染入口与层级拼装。

mod overlay;
mod splitters;
mod workspace;

use gpui::{
    Context, CursorStyle, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
    div, rgb,
};
use zom_protocol::OverlayTarget;

use super::{ActiveDockDrag, ZomRootView};
use crate::{
    components::{title_bar, tool_bar},
    theme::color,
};

impl Render for ZomRootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(action) = self.state.take_pending_ui_action() {
            self.apply_ui_action(action, window, cx);
        }

        self.schedule_pending_toast_auto_clear(window, cx);

        if let Some(target) = self.state.take_pending_focus_target() {
            self.apply_focus_target(target, window, cx);
        }

        let workspace_row = self.render_workspace_row(window, cx);

        let drag_cursor = match self.active_dock_drag {
            Some(ActiveDockDrag::Bottom { .. }) => CursorStyle::ResizeUpDown,
            Some(ActiveDockDrag::Left | ActiveDockDrag::Right) => CursorStyle::ResizeLeftRight,
            None => CursorStyle::Arrow,
        };

        let mut root = div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .cursor(drag_cursor)
            .capture_key_down(cx.listener(|this, event, _window, cx| {
                if this.handle_shortcut_keydown(event, cx) {
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .capture_any_mouse_up(cx.listener(|this, _event, _window, cx| {
                if this.active_dock_drag.is_some() {
                    this.active_dock_drag = None;
                    cx.notify();
                }
            }))
            .on_mouse_move(cx.listener(|this, event, window, cx| {
                let workspace_width: f32 = window.viewport_size().width.into();
                let viewport_height: f32 = window.viewport_size().height.into();
                let workspace_height = ZomRootView::workspace_height_from_viewport(viewport_height);
                this.on_drag_mouse_move(event, workspace_width, workspace_height, cx);
            }))
            .on_key_down(cx.listener(|this, event, _window, cx| {
                if this.handle_shortcut_keydown(event, cx) {
                    cx.stop_propagation();
                    cx.notify();
                }
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
            .child(tool_bar::render(&self.state));

        if matches!(self.state.active_overlay, Some(OverlayTarget::Settings)) {
            root = root.child(self.render_settings_overlay(cx));
        }
        if let Some(toast_layer) = self.render_notification_toast_layer() {
            root = root.child(toast_layer);
        }

        root
    }
}
