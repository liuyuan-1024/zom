//! toast 写入与短时展示同步逻辑。

use super::{DesktopAppState, DesktopToast, DesktopToastEvent, DesktopToastLevel};

impl DesktopAppState {
    /// 依据事件语义决定是否展示 toast。
    /// 返回写入后的 toast id；若事件不应弹 toast 则返回 `None`。
    pub fn publish_toast_event(&mut self, event: DesktopToastEvent) -> Option<u64> {
        if !should_emit_toast(&event) {
            return None;
        }

        let id = self.next_toast_id;
        self.next_toast_id = self.next_toast_id.saturating_add(1);
        self.active_toast = Some(DesktopToast {
            id,
            level: event.level,
            message: event.message,
        });
        Some(id)
    }

    /// 清空当前 toast。
    pub fn clear_active_toast(&mut self) {
        self.active_toast = None;
    }
}

/// 仅保留 toast 通道。
fn should_emit_toast(event: &DesktopToastEvent) -> bool {
    match event.level {
        DesktopToastLevel::Error | DesktopToastLevel::Warning => true,
        DesktopToastLevel::Info => event.is_user_initiated,
    }
}
