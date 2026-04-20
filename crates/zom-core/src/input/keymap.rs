//! 按键映射表与作用域解析逻辑。

use std::collections::HashMap;

use crate::FocusTarget;

use super::{
    InputContext, InputResolution, Keystroke, ShortcutBinding, ShortcutRegistry, ShortcutScope,
};

/// 输入键位解析表，按全局与焦点作用域组织快捷键。
#[derive(Debug, Clone, Default)]
pub struct Keymap {
    /// 全局快捷键
    global: HashMap<Keystroke, InputResolution>,
    /// 焦点作用域快捷键
    scoped: HashMap<FocusTarget, HashMap<Keystroke, InputResolution>>,
}

impl Keymap {
    /// 创建空键位映射表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 将统一快捷键绑定应用到解析表中。
    pub fn bind_shortcut(&mut self, binding: &ShortcutBinding) {
        match binding.scope {
            ShortcutScope::Global => {
                self.bind_global(binding.keystroke.clone(), binding.resolution.clone())
            }
            ShortcutScope::Focus(target) => self.bind_for_focus(
                target,
                binding.keystroke.clone(),
                binding.resolution.clone(),
            ),
        }
    }

    /// 从快捷键注册表构建一份解析表。
    pub fn from_shortcut_registry(registry: &ShortcutRegistry) -> Self {
        let mut keymap = Self::new();
        for binding in registry.bindings() {
            keymap.bind_shortcut(binding);
        }
        keymap
    }

    /// 绑定一条全局快捷键。
    pub fn bind_global(&mut self, key: Keystroke, resolution: InputResolution) {
        self.global.insert(key, resolution);
    }

    /// 绑定一条只在指定焦点下生效的快捷键。
    pub fn bind_for_focus(
        &mut self,
        focus: FocusTarget,
        key: Keystroke,
        resolution: InputResolution,
    ) {
        self.scoped
            .entry(focus)
            .or_default()
            .insert(key, resolution);
    }

    /// 绑定编辑器作用域快捷键。
    pub fn bind_editor(&mut self, key: Keystroke, resolution: InputResolution) {
        self.bind_for_focus(FocusTarget::Editor, key, resolution);
    }

    /// 绑定文件树面板作用域快捷键。
    pub fn bind_file_tree(&mut self, key: Keystroke, resolution: InputResolution) {
        self.bind_for_focus(FocusTarget::FileTreePanel, key, resolution);
    }

    /// 解析按键：优先焦点作用域，再回退到全局作用域。
    pub fn resolve(&self, input: &Keystroke, context: &InputContext) -> InputResolution {
        if let Some(by_focus) = self.scoped.get(&context.focus)
            && let Some(resolution) = by_focus.get(input)
        {
            return resolution.clone();
        }

        self.global
            .get(input)
            .cloned()
            .unwrap_or(InputResolution::Noop)
    }
}
