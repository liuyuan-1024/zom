//! 输入协议模型。
//! 这里只描述输入世界长什么样，具体解析逻辑放在 `zom-input`。

use crate::Command;

/// 键盘修饰键状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Modifiers {
    /// 是否按下 Control。
    pub ctrl: bool,
    /// 是否按下 Alt。
    pub alt: bool,
    /// 是否按下 Shift。
    pub shift: bool,
    /// 是否按下 Meta 或 Command。
    pub meta: bool,
}

/// 与平台无关的按键编码。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// 可见字符键。
    Char(char),
    /// 回车键。
    Enter,
    /// 退格键。
    Backspace,
    /// 删除键。
    Delete,
    /// Tab 键。
    Tab,
    /// Escape 键。
    Escape,
    /// 左方向键。
    Left,
    /// 右方向键。
    Right,
    /// 上方向键。
    Up,
    /// 下方向键。
    Down,
    /// Home 键。
    Home,
    /// End 键。
    End,
    /// Page Up 键。
    PageUp,
    /// Page Down 键。
    PageDown,
    /// 功能键。
    F(u8),
}

/// 一次完整的按键输入，由按键本体和修饰键组成。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keystroke {
    /// 被触发的主按键。
    pub key: KeyCode,
    /// 本次按键携带的修饰键状态。
    pub modifiers: Modifiers,
}

/// 当前焦点所在的逻辑区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusTarget {
    /// 编辑器区域。
    Editor,
    /// 侧边栏区域。
    Sidebar,
    /// 命令面板区域。
    Palette,
    /// 普通面板区域。
    Panel,
    /// 终端区域。
    Terminal,
    /// 当前没有明确焦点。
    None,
}

/// 输入解析时依赖的上下文信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputContext {
    /// 当前焦点所在区域。
    pub focus: FocusTarget,
    /// 是否处于原生文本输入语境中。
    pub in_text_input: bool,
    /// 命令面板当前是否打开。
    pub command_palette_open: bool,
    /// 编辑器相关上下文，仅在焦点位于编辑器时存在。
    pub editor: Option<EditorInputContext>,
}

/// 编辑器局部输入上下文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorInputContext {
    /// 当前编辑器是否允许编辑。
    pub editable: bool,
    /// 当前缓冲区是否只读。
    pub read_only: bool,
    /// 当前是否已有选区。
    pub has_selection: bool,
}

/// 输入系统解析后的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputResolution {
    /// 解析成一个抽象命令。
    Command(Command),
    /// 解析成直接插入的文本。
    InsertText(String),
    /// 当前输入没有匹配任何行为。
    Noop,
}
