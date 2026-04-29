//! 焦点/面板显隐逻辑

use zom_protocol::{FocusTarget, OverlayTarget, command::TabAction};

use crate::state::panel_dock;

use super::DesktopAppState;

impl DesktopAppState {
    /// 聚焦到指定面板：若面板当前隐藏，则先显示后聚焦。
    pub(super) fn focus_panel(&mut self, target: FocusTarget) {
        // 同一停靠区只保留一个可见面板，避免侧边栏叠层竞争宽度。
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
        if target == OverlayTarget::FindReplace {
            self.focus_editor();
            return;
        }
        self.focused_target = target.into();
        self.pending_focus_target = Some(self.focused_target);
    }

    /// 关闭当前聚焦组件：优先关闭焦点悬浮层，其次关闭焦点面板，最后关闭当前标签页。
    ///
    /// 该顺序模拟常见 IDE 的 `Esc/Close` 行为，降低用户心智成本。
    pub(super) fn close_focused(&mut self) {
        if self.active_overlay.is_some() {
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
            self.dispatch_tab_action(TabAction::CloseActiveTab);
        }
    }

    /// 将编辑器视图设为当前焦点目标。
    pub(super) fn focus_editor(&mut self) {
        self.focused_target = FocusTarget::Editor;
        self.pending_focus_target = Some(FocusTarget::Editor);
    }

    /// 在面板接收焦点前执行必要的准备动作。
    ///
    /// 比如文件树面板需要保证存在选中项，才能响应方向键和激活动作。
    fn prepare_panel_focus(&mut self, target: FocusTarget) {
        if target == FocusTarget::FileTreePanel {
            self.ensure_file_tree_selection();
        }
    }

    /// 设置面板显隐位。
    ///
    /// 仅作用于“可见性托管面板”；编辑器和 overlay 不受此接口影响。
    pub(super) fn set_panel_visible(&mut self, target: FocusTarget, is_visible: bool) {
        if !target.is_visibility_managed_panel() {
            return;
        }

        if is_visible {
            self.visible_panels.insert(target);
        } else {
            self.visible_panels.remove(&target);
        }
    }

    /// 隐藏与目标处于同一停靠区的其他面板。
    fn hide_panels_in_same_dock(&mut self, target: FocusTarget) {
        let Some(dock) = panel_dock(target) else {
            return;
        };
        self.visible_panels
            .retain(|panel| panel_dock(*panel) != Some(dock));
    }
}
