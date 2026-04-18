use zom_core::command::{CommandShortcut, default_shortcut_bindings};

use crate::{ShortcutBinding, ShortcutBindingSpec, ShortcutRegistry};

/// 构造默认快捷键注册表（构建函数）。
pub(crate) fn build_default_shortcut_registry() -> ShortcutRegistry {
    let mut registry = ShortcutRegistry::new();
    register_catalog_shortcuts(&mut registry);
    registry
}

fn register_catalog_shortcuts(registry: &mut ShortcutRegistry) {
    for binding in default_shortcut_bindings() {
        register_shortcut(registry, binding.command, binding.shortcut);
    }
}

fn register_shortcut(
    registry: &mut ShortcutRegistry,
    command: zom_core::Command,
    shortcut: CommandShortcut,
) {
    let spec = ShortcutBindingSpec::new(command, shortcut.keystroke)
        .with_when(shortcut.when)
        .with_platform(shortcut.platform)
        .with_priority(shortcut.priority);
    let resolution = crate::command(spec.command.clone());
    registry.register(ShortcutBinding::from_spec(spec, shortcut.scope, resolution));
}
