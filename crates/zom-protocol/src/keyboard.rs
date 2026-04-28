//! 键盘协议模型（按键、上下文、解析结果）。

use crate::{CommandInvocation, FocusTarget};

/// 键盘修饰键状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Modifiers {
    /// 是否按下 Control。
    pub has_ctrl: bool,
    /// 是否按下 Alt。
    pub has_alt: bool,
    /// 是否按下 Shift。
    pub has_shift: bool,
    /// 是否按下 Meta 或 Command。
    pub has_meta: bool,
}

impl Modifiers {
    /// 构造一组显式的修饰键状态。
    pub const fn new(has_ctrl: bool, has_alt: bool, has_shift: bool, has_meta: bool) -> Self {
        Self {
            has_ctrl,
            has_alt,
            has_shift,
            has_meta,
        }
    }

    /// 判断当前是否没有任何修饰键。
    pub fn is_empty(self) -> bool {
        !self.has_ctrl && !self.has_alt && !self.has_shift && !self.has_meta
    }
}

/// 与平台无关的按键编码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keystroke {
    /// 被触发的主按键。
    pub key: KeyCode,
    /// 本次按键携带的修饰键状态。
    pub modifiers: Modifiers,
}

impl Keystroke {
    /// 用按键和修饰键构造一次按键输入。
    pub const fn new(key: KeyCode, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }
}

/// 输入解析时依赖的上下文信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputContext {
    /// 当前焦点所在区域。
    pub focus: FocusTarget,
    /// 是否处于原生文本输入语境中。
    pub is_in_text_input: bool,
    /// 命令面板当前是否打开。
    pub is_command_palette_open: bool,
    /// 编辑器相关上下文，仅在焦点位于编辑器时存在。
    pub editor: Option<EditorInputContext>,
}

impl InputContext {
    /// 构造一个基础输入上下文。
    pub fn new(focus: FocusTarget) -> Self {
        Self {
            focus,
            is_in_text_input: false,
            is_command_palette_open: false,
            editor: None,
        }
    }

    /// 用编辑器上下文补全当前输入上下文。
    pub fn with_editor(mut self, editor: EditorInputContext) -> Self {
        self.editor = Some(editor);
        self
    }

    /// 标记当前处于文本输入语境。
    pub fn with_text_input(mut self, is_in_text_input: bool) -> Self {
        self.is_in_text_input = is_in_text_input;
        self
    }

    /// 标记命令面板的打开状态。
    pub fn with_command_palette_open(mut self, is_command_palette_open: bool) -> Self {
        self.is_command_palette_open = is_command_palette_open;
        self
    }
}

/// 编辑器局部输入上下文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorInputContext {
    /// 当前编辑器是否允许编辑。
    pub is_editable: bool,
    /// 当前缓冲区是否只读。
    pub is_read_only: bool,
    /// 当前是否已有选区。
    pub has_selection: bool,
}

impl EditorInputContext {
    /// 构造一份编辑器局部输入上下文。
    pub fn new(is_editable: bool, is_read_only: bool, has_selection: bool) -> Self {
        Self {
            is_editable,
            is_read_only,
            has_selection,
        }
    }
}

/// 输入系统解析后的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputResolution {
    /// 解析成一个抽象命令。
    Command(CommandInvocation),
    /// 解析成直接插入的文本。
    InsertText(String),
    /// 当前输入没有匹配任何行为。
    Noop,
}

impl InputResolution {
    /// 用命令结果构造输入解析结果。
    pub fn command(command: CommandInvocation) -> Self {
        Self::Command(command)
    }

    /// 用文本插入结果构造输入解析结果。
    pub fn insert_text(text: impl Into<String>) -> Self {
        Self::InsertText(text.into())
    }

    /// 判断当前结果是否为空操作。
    pub fn is_noop(&self) -> bool {
        matches!(self, Self::Noop)
    }
}

/// 把命令语义包装为输入解析结果。
pub fn command(command: CommandInvocation) -> InputResolution {
    InputResolution::Command(command)
}

#[cfg(test)]
mod tests {
    use crate::{CommandInvocation, EditorAction};

    use super::{InputResolution, command};

    #[test]
    fn input_resolution_command_constructor_wraps_invocation() {
        let invocation = CommandInvocation::from(EditorAction::Undo);
        assert_eq!(
            InputResolution::command(invocation.clone()),
            InputResolution::Command(invocation)
        );
    }

    #[test]
    fn command_helper_wraps_invocation_as_resolution() {
        let invocation = CommandInvocation::from(EditorAction::Redo);
        assert_eq!(
            command(invocation.clone()),
            InputResolution::Command(invocation)
        );
    }
}
