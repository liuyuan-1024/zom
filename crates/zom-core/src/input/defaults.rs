//! 默认快捷键注册表构建逻辑。

use crate::{
    CommandInvocation,
    command::{CommandShortcut, default_shortcut_bindings},
};

use super::{ShortcutBinding, ShortcutBindingSpec, ShortcutRegistry, command as resolve_command};

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
    command_invocation: CommandInvocation,
    shortcut: CommandShortcut,
) {
    let spec = ShortcutBindingSpec::new(command_invocation, shortcut.keystroke)
        .with_priority(shortcut.priority);
    let resolution = resolve_command(spec.command.clone());
    registry.register(ShortcutBinding::from_spec(spec, shortcut.scope, resolution));
}
