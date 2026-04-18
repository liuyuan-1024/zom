use zom_core::{
    Command, InputResolution, KeyCode, Keystroke,
    command::{
        ShortcutPlatform as CoreShortcutPlatform, ShortcutScope as CoreShortcutScope,
        ShortcutWhen as CoreShortcutWhen,
    },
};

/// 快捷键作用域（源自 `zom-core::command::ShortcutScope`）。
pub type ShortcutScope = CoreShortcutScope;
/// 快捷键触发条件（源自 `zom-core::command::ShortcutWhen`）。
pub type ShortcutWhen = CoreShortcutWhen;
/// 快捷键平台（源自 `zom-core::command::ShortcutPlatform`）。
pub type ShortcutPlatform = CoreShortcutPlatform;

/// 快捷键绑定契约（不含运行时解析结果）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBindingSpec {
    /// 语义命令。
    pub command: Command,
    /// 按键定义。
    pub keystroke: Keystroke,
    /// 额外触发条件。
    pub when: ShortcutWhen,
    /// 适用平台。
    pub platform: ShortcutPlatform,
    /// 冲突处理优先级（越大越优先）。
    pub priority: u8,
}

impl ShortcutBindingSpec {
    /// 创建一个默认绑定契约。
    pub fn new(command: Command, keystroke: Keystroke) -> Self {
        Self {
            command,
            keystroke,
            when: ShortcutWhen::Always,
            platform: ShortcutPlatform::Any,
            priority: 0,
        }
    }

    /// 设置额外触发条件。
    pub fn with_when(mut self, when: ShortcutWhen) -> Self {
        self.when = when;
        self
    }

    /// 设置适用平台。
    pub fn with_platform(mut self, platform: ShortcutPlatform) -> Self {
        self.platform = platform;
        self
    }

    /// 设置绑定优先级。
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// 统一快捷键绑定定义（按键 + 作用域 + 语义命令）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBinding {
    /// 语义命令。
    pub command: Command,
    /// 作用域。
    pub scope: ShortcutScope,
    /// 按键定义。
    pub keystroke: Keystroke,
    /// 额外触发条件。
    pub when: ShortcutWhen,
    /// 适用平台。
    pub platform: ShortcutPlatform,
    /// 冲突处理优先级（越大越优先）。
    pub priority: u8,
    /// 解析后的执行结果。
    pub resolution: InputResolution,
}

impl ShortcutBinding {
    pub(crate) fn from_spec(
        spec: ShortcutBindingSpec,
        scope: ShortcutScope,
        resolution: InputResolution,
    ) -> Self {
        Self {
            command: spec.command,
            scope,
            keystroke: spec.keystroke,
            when: spec.when,
            platform: spec.platform,
            priority: spec.priority,
            resolution,
        }
    }
}

/// 默认快捷键注册表（单一事实源）。
#[derive(Debug, Clone, Default)]
pub struct ShortcutRegistry {
    bindings: Vec<ShortcutBinding>,
}

impl ShortcutRegistry {
    /// 创建空注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一条快捷键绑定。
    pub fn register(&mut self, binding: ShortcutBinding) {
        self.bindings.push(binding);
    }

    /// 读取全部快捷键定义。
    pub fn bindings(&self) -> &[ShortcutBinding] {
        &self.bindings
    }

    /// 读取某个命令对应的默认快捷键文案。
    pub fn shortcut_hint(&self, command: &Command) -> Option<String> {
        self.bindings
            .iter()
            .find(|binding| &binding.command == command)
            .map(|binding| format_keystroke(&binding.keystroke))
    }
}

fn format_keystroke(keystroke: &Keystroke) -> String {
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
