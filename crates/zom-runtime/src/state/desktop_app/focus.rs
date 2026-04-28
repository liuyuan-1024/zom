//! 焦点/面板显隐逻辑

use zom_protocol::{FocusTarget, OverlayTarget, command::TabAction};

use crate::state::panel_dock;

use super::DesktopAppState;

impl DesktopAppState {
    /// 聚焦到指定面板：若面板当前隐藏，则先显示后聚焦。
    pub(super) fn focus_panel(&mut self, target: FocusTarget) {
        self.hide_panels_in_same_dock(target);
        self.set_panel_visible(target, true);
        self.active_overlay = None;
        self.focused_target = target;
        self.pending_focus_target = Some(target);
        self.prepare_panel_focus(target);
    }

    /// 聚焦到指定悬浮层：显示并聚焦。
    pub(super) fn focus_overlay(&mut self, target: OverlayTarget) {
        self.active_overlay = Some(target);
        self.focused_target = target.into();
        self.pending_focus_target = Some(self.focused_target);
    }

    /// 关闭当前聚焦组件：优先关闭焦点悬浮层，其次关闭焦点面板，最后关闭当前标签页。
    pub(super) fn close_focused(&mut self) {
        if self.focused_target.is_overlay() && self.active_overlay.is_some() {
            self.active_overlay = None;
            self.focus_editor();
            return;
        }

        if self.focused_target.is_visibility_managed_panel()
            && self.is_panel_visible(self.focused_target)
        {
            self.set_panel_visible(self.focused_target, false);
            self.focus_editor();
            return;
        }

        if self.focused_target == FocusTarget::Editor {
            self.handle_tab_command(TabAction::CloseActiveTab);
        }
    }

    pub(super) fn focus_editor(&mut self) {
        self.focused_target = FocusTarget::Editor;
        self.pending_focus_target = Some(FocusTarget::Editor);
    }

    /// 在面板接收焦点前执行必要的准备动作。
    fn prepare_panel_focus(&mut self, target: FocusTarget) {
        if target == FocusTarget::FileTreePanel {
            self.ensure_file_tree_selection();
        }
    }

    pub(super) fn set_panel_visible(&mut self, target: FocusTarget, visible: bool) {
        if !target.is_visibility_managed_panel() {
            return;
        }

        if visible {
            self.visible_panels.insert(target);
        } else {
            self.visible_panels.remove(&target);
        }
    }

    fn hide_panels_in_same_dock(&mut self, target: FocusTarget) {
        let Some(dock) = panel_dock(target) else {
            return;
        };
        self.visible_panels
            .retain(|panel| panel_dock(*panel) != Some(dock));
    }
}
