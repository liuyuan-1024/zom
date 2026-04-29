use std::collections::HashMap;

use zom_protocol::{FocusTarget, InputContext, InputResolution, Keystroke};

use crate::{ShortcutBinding, ShortcutRegistry, ShortcutScope};

#[derive(Debug, Clone, Default)]
pub struct Keymap {
    /// 全局键位映射：在任意焦点下都可命中（除非被作用域键覆盖）。
    global: HashMap<Keystroke, InputResolution>,
    /// 按焦点分桶的局部映射：用于同键在不同面板映射不同命令。
    scoped: HashMap<FocusTarget, HashMap<Keystroke, InputResolution>>,
}

impl Keymap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_shortcut(&mut self, binding: &ShortcutBinding) {
        // 注册时直接按 scope 拆入两层表，解析阶段可 O(1) 命中。
        match binding.scope {
            ShortcutScope::Global => {
                self.bind_global(binding.keystroke, binding.resolution.clone())
            }
            ShortcutScope::Focus(target) => {
                self.bind_for_focus(target, binding.keystroke, binding.resolution.clone())
            }
        }
    }

    pub fn from_shortcut_registry(registry: &ShortcutRegistry) -> Self {
        let mut keymap = Self::new();
        for binding in registry.bindings() {
            keymap.bind_shortcut(binding);
        }
        keymap
    }

    pub fn bind_global(&mut self, key: Keystroke, resolution: InputResolution) {
        // 同键重复绑定时后写覆盖前写，调用方可通过注册顺序实现重载。
        self.global.insert(key, resolution);
    }

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

    pub fn bind_editor(&mut self, key: Keystroke, resolution: InputResolution) {
        self.bind_for_focus(FocusTarget::Editor, key, resolution);
    }

    pub fn bind_file_tree(&mut self, key: Keystroke, resolution: InputResolution) {
        self.bind_for_focus(FocusTarget::FileTreePanel, key, resolution);
    }

    pub fn resolve(&self, input: &Keystroke, context: &InputContext) -> InputResolution {
        // 同键位冲突时，焦点作用域优先于全局作用域。
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
