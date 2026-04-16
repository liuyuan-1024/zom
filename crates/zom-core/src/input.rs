//! 输入协议模型。
//! 这里只描述输入世界长什么样，具体解析逻辑放在 `zom-input`。

use crate::Command;

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
    pub fn new(ctrl: bool, alt: bool, shift: bool, meta: bool) -> Self {
        Self {
            has_ctrl: ctrl,
            has_alt: alt,
            has_shift: shift,
            has_meta: meta,
        }
    }

    /// 判断当前是否没有任何修饰键。
    pub fn is_empty(self) -> bool {
        !self.has_ctrl && !self.has_alt && !self.has_shift && !self.has_meta
    }
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

impl Keystroke {
    /// 用按键和修饰键构造一次按键输入。
    pub fn new(key: KeyCode, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }
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
    pub fn with_text_input(mut self, in_text_input: bool) -> Self {
        self.is_in_text_input = in_text_input;
        self
    }

    /// 标记命令面板的打开状态。
    pub fn with_command_palette_open(mut self, command_palette_open: bool) -> Self {
        self.is_command_palette_open = command_palette_open;
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
    pub fn new(editable: bool, read_only: bool, has_selection: bool) -> Self {
        Self {
            is_editable: editable,
            is_read_only: read_only,
            has_selection,
        }
    }
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

impl InputResolution {
    /// 用命令结果构造输入解析结果。
    pub fn command(command: Command) -> Self {
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

#[cfg(test)]
mod tests {
    use crate::{Command, EditorCommand};

    use super::{
        EditorInputContext, FocusTarget, InputContext, InputResolution, KeyCode, Keystroke,
        Modifiers,
    };

    #[test]
    fn modifiers_can_report_whether_any_modifier_is_pressed() {
        assert!(Modifiers::default().is_empty());
        assert!(!Modifiers::new(true, false, false, false).is_empty());
    }

    #[test]
    fn input_context_builder_helpers_keep_it_as_data_model() {
        let editor = EditorInputContext::new(true, false, true);
        let context = InputContext::new(FocusTarget::Editor)
            .with_editor(editor.clone())
            .with_text_input(true)
            .with_command_palette_open(true);

        assert_eq!(context.focus, FocusTarget::Editor);
        assert!(context.is_in_text_input);
        assert!(context.is_command_palette_open);
        assert_eq!(context.editor, Some(editor));
    }

    #[test]
    fn keystroke_and_input_resolution_have_convenient_constructors() {
        let keystroke = Keystroke::new(KeyCode::Enter, Modifiers::default());
        let resolution = InputResolution::command(Command::from(EditorCommand::Undo));
        let text = InputResolution::insert_text("x");

        assert_eq!(keystroke.key, KeyCode::Enter);
        assert_eq!(keystroke.modifiers, Modifiers::default());
        assert!(!resolution.is_noop());
        assert_eq!(text, InputResolution::InsertText("x".into()));
    }
}
