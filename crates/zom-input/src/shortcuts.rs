use zom_protocol::{CommandInvocation, FocusTarget, InputResolution, KeyCode, Keystroke};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutScope {
    /// 全局作用域：不依赖当前焦点面板。
    Global,
    /// 焦点作用域：仅当焦点落在目标区域时生效。
    Focus(FocusTarget),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBindingSpec {
    /// 目标命令（不含运行时载荷）。
    pub command: CommandInvocation,
    /// 触发该命令的按键组合。
    pub keystroke: Keystroke,
    /// 冲突解决优先级，数值越大越优先（由上层注册策略解释）。
    pub priority: u8,
}

impl ShortcutBindingSpec {
    pub fn new(command: CommandInvocation, keystroke: Keystroke) -> Self {
        Self {
            command,
            keystroke,
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBinding {
    /// 绑定目标命令。
    pub command: CommandInvocation,
    /// 生效作用域（全局或焦点限定）。
    pub scope: ShortcutScope,
    /// 原始按键输入描述。
    pub keystroke: Keystroke,
    /// 预留冲突优先级（用于未来排序/裁决策略）。
    pub priority: u8,
    /// 解析结果缓存，避免命中后再二次构造 `InputResolution`。
    pub resolution: InputResolution,
}

#[derive(Debug, Clone, Default)]
pub struct ShortcutRegistry {
    /// 按注册顺序存储绑定；顺序会影响 `shortcut_hint` 的首个命中结果。
    bindings: Vec<ShortcutBinding>,
}

impl ShortcutRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, binding: ShortcutBinding) {
        self.bindings.push(binding);
    }

    pub fn bindings(&self) -> &[ShortcutBinding] {
        &self.bindings
    }

    /// 返回命令的首个快捷键提示。
    ///
    /// 当前策略是“按注册顺序取第一个匹配项”，适合提供稳定 UI hint，
    /// 但不负责完整冲突裁决。
    pub fn shortcut_hint(&self, command: &CommandInvocation) -> Option<String> {
        self.bindings
            .iter()
            .find(|binding| &binding.command == command)
            .map(|binding| format_keystroke(&binding.keystroke))
    }
}

fn format_keystroke(keystroke: &Keystroke) -> String {
    // 统一平台无关展示格式：修饰键前缀 + 主键，方便状态栏/菜单复用。
    let mut parts = Vec::new();
    if keystroke.modifiers.has_meta {
        parts.push("Cmd".to_string());
    }
    if keystroke.modifiers.has_ctrl {
        parts.push("Ctrl".to_string());
    }
    if keystroke.modifiers.has_alt {
        parts.push("Alt".to_string());
    }
    if keystroke.modifiers.has_shift {
        parts.push("Shift".to_string());
    }

    let key = match keystroke.key {
        KeyCode::Char(c) => c.to_ascii_uppercase().to_string(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Delete => "Delete".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Escape => "Esc".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "PageUp".into(),
        KeyCode::PageDown => "PageDown".into(),
        KeyCode::F(index) => format!("F{index}"),
    };
    parts.push(key);

    parts.join("+")
}
