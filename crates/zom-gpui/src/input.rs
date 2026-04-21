//! GPUI 按键事件到核心 Keystroke 的转换桥接。
//! `zom-gpui` 输入事件到 `zom-protocol` 输入协议的适配层。
//! 这里只做平台事件字段对齐，不做按键语义到命令语义的映射。

use gpui::KeyDownEvent;
use zom_protocol::{KeyCode, Keystroke, Modifiers};

/// 将 GPUI 的按键事件转换为跨 crate 共享的 `Keystroke`。
pub(crate) fn to_core_keystroke(event: &KeyDownEvent) -> Option<Keystroke> {
    let key = to_core_key_code(event.keystroke.key.as_str())?;
    let modifiers = Modifiers::new(
        event.keystroke.modifiers.control,
        event.keystroke.modifiers.alt,
        event.keystroke.modifiers.shift,
        event.keystroke.modifiers.platform,
    );
    Some(Keystroke::new(key, modifiers))
}

fn to_core_key_code(key: &str) -> Option<KeyCode> {
    match key {
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "enter" => Some(KeyCode::Enter),
        "escape" => Some(KeyCode::Escape),
        "backspace" => Some(KeyCode::Backspace),
        "delete" => Some(KeyCode::Delete),
        "tab" => Some(KeyCode::Tab),
        "space" => Some(KeyCode::Char(' ')),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" => Some(KeyCode::PageUp),
        "pagedown" => Some(KeyCode::PageDown),
        _ => {
            if key.chars().count() == 1 {
                return key
                    .chars()
                    .next()
                    .map(|c| KeyCode::Char(c.to_ascii_lowercase()));
            }
            None
        }
    }
}
