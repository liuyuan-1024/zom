//! 输入协议，
//! 具体的输入解析放到 zom-input

use crate::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    Enter,
    Escape,
    Backspace,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    F(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keystroke {
    pub key: KeyCode,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusTarget {
    Editor,
    Sidebar,
    Palette,
    Panel,
    Terminal,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputContext {
    pub focus: FocusTarget,
    pub in_text_input: bool,
    pub command_palette_open: bool,
    pub editor: Option<EditorInputContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorInputContext {
    pub editable: bool,
    pub read_only: bool,
    pub has_selection: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputResolution {
    Command(Command),
    InsertText(String),
    Noop,
}
