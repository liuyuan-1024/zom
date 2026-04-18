use zom_core::{FocusTarget, InputResolution, KeyCode, Keystroke};

/// 快捷键代表的语义动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    /// 显示并聚焦文件树。
    FocusFileTreePanel,
    /// 显示并聚焦 Git 面板。
    FocusGitPanel,
    /// 显示并聚焦 Outline 面板。
    FocusOutlinePanel,
    /// 显示并聚焦全局搜索面板。
    FocusProjectSearchPanel,
    /// 显示并聚焦终端面板。
    FocusTerminalPanel,
    /// 从标题栏打开项目选择。
    OpenProjectFromTitleBar,
    /// 从标题栏打开设置。
    OpenSettingsFromTitleBar,
    /// 关闭当前已聚焦面板。
    HideFocusedPanel,
    /// 文件树选择上一项。
    FileTreeSelectPrev,
    /// 文件树选择下一项。
    FileTreeSelectNext,
    /// 文件树展开目录或下探到子项。
    FileTreeExpandOrDescend,
    /// 文件树折叠目录或回到父项。
    FileTreeCollapseOrAscend,
    /// 文件树激活当前选中项。
    FileTreeActivateSelection,
}

/// 快捷键的作用域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutScope {
    /// 全局快捷键。
    Global,
    /// 仅在指定焦点下生效。
    Focus(FocusTarget),
}

/// 统一快捷键绑定定义（按键 + 作用域 + 语义命令）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutBinding {
    /// 语义动作标识。
    pub action: ShortcutAction,
    /// 作用域。
    pub scope: ShortcutScope,
    /// 按键定义。
    pub keystroke: Keystroke,
    /// 解析后的执行结果。
    pub resolution: InputResolution,
}

impl ShortcutBinding {
    pub(crate) fn new(
        action: ShortcutAction,
        scope: ShortcutScope,
        keystroke: Keystroke,
        resolution: InputResolution,
    ) -> Self {
        Self {
            action,
            scope,
            keystroke,
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

    /// 读取某个语义动作对应的默认快捷键文案。
    pub fn shortcut_hint(&self, action: ShortcutAction) -> Option<String> {
        self.bindings
            .iter()
            .find(|binding| binding.action == action)
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
