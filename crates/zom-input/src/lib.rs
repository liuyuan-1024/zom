//! 输入策略层：默认键位注册、键位解析与快捷键提示。

mod defaults;
mod keymap;
mod shortcuts;

use std::sync::LazyLock;

use defaults::build_default_shortcut_registry;
use zom_protocol::{CommandInvocation, FocusTarget};

pub use keymap::Keymap;
pub use shortcuts::{ShortcutBinding, ShortcutBindingSpec, ShortcutRegistry, ShortcutScope};
pub use zom_protocol::keyboard::{
    EditorInputContext, InputContext, InputResolution, KeyCode, Keystroke, Modifiers,
};

/// 读取默认快捷键注册表（单例）。
pub fn default_shortcut_registry() -> &'static ShortcutRegistry {
    &DEFAULT_SHORTCUT_REGISTRY
}

/// 读取某个命令对应的默认快捷键文案。
pub fn shortcut_hint(command: &CommandInvocation) -> Option<String> {
    default_shortcut_registry().shortcut_hint(command)
}

/// 基于默认注册表构建默认键位映射。
pub fn default_keymap() -> Keymap {
    Keymap::from_shortcut_registry(default_shortcut_registry())
}

static DEFAULT_SHORTCUT_REGISTRY: LazyLock<ShortcutRegistry> =
    LazyLock::new(build_default_shortcut_registry);
static DEFAULT_KEYMAP: LazyLock<Keymap> = LazyLock::new(default_keymap);

/// 使用默认键位方案解析一次输入。
pub fn resolve_default(input: &Keystroke, context: &InputContext) -> InputResolution {
    let resolution = DEFAULT_KEYMAP.resolve(input, context);
    if !resolution.is_noop() {
        return resolution;
    }

    resolve_editor_text_fallback(input, context)
}

fn resolve_editor_text_fallback(input: &Keystroke, context: &InputContext) -> InputResolution {
    if context.focus != FocusTarget::Editor {
        return InputResolution::Noop;
    }

    if input.modifiers.has_ctrl || input.modifiers.has_alt || input.modifiers.has_meta {
        return InputResolution::Noop;
    }

    match input.key {
        KeyCode::Char(c) => {
            let ch = if input.modifiers.has_shift && c.is_ascii_alphabetic() {
                c.to_ascii_uppercase()
            } else {
                c
            };
            InputResolution::insert_text(ch.to_string())
        }
        _ => InputResolution::Noop,
    }
}

#[cfg(test)]
mod tests {
    use zom_protocol::{
        CommandInvocation, EditorAction, FocusTarget, KeyCode, Keystroke, Modifiers,
        NotificationAction, WorkspaceAction,
    };

    use super::{InputContext, InputResolution, default_shortcut_registry, resolve_default};

    #[test]
    fn default_registry_contains_expected_workspace_focus_binding() {
        let command =
            CommandInvocation::from(WorkspaceAction::FocusPanel(FocusTarget::FileTreePanel));
        let has_file_tree_focus = default_shortcut_registry()
            .bindings()
            .iter()
            .any(|binding| binding.command == command);
        assert!(has_file_tree_focus);
    }

    #[test]
    fn resolve_default_falls_back_to_plain_text_insert_in_editor_focus() {
        let key = Keystroke::new(KeyCode::Char('x'), Modifiers::default());
        assert_eq!(
            resolve_default(&key, &InputContext::new(FocusTarget::Editor)),
            InputResolution::InsertText("x".into())
        );
    }

    #[test]
    fn resolve_default_does_not_insert_plain_text_outside_editor_focus() {
        let key = Keystroke::new(KeyCode::Char('x'), Modifiers::default());
        assert_eq!(
            resolve_default(&key, &InputContext::new(FocusTarget::FileTreePanel)),
            InputResolution::Noop
        );
    }

    #[test]
    fn resolve_default_maps_tab_and_shift_tab_to_editor_indent_commands() {
        let tab = Keystroke::new(KeyCode::Tab, Modifiers::default());
        assert_eq!(
            resolve_default(&tab, &InputContext::new(FocusTarget::Editor)),
            InputResolution::command(CommandInvocation::from(EditorAction::InsertIndent))
        );

        let shift_tab = Keystroke::new(KeyCode::Tab, Modifiers::new(false, false, true, false));
        assert_eq!(
            resolve_default(&shift_tab, &InputContext::new(FocusTarget::Editor)),
            InputResolution::command(CommandInvocation::from(EditorAction::Outdent))
        );
    }

    #[test]
    fn resolve_default_maps_enter_to_newline_command() {
        let enter = Keystroke::new(KeyCode::Enter, Modifiers::default());
        assert_eq!(
            resolve_default(&enter, &InputContext::new(FocusTarget::Editor)),
            InputResolution::command(CommandInvocation::from(EditorAction::InsertNewline))
        );
    }

    #[test]
    fn resolve_default_maps_primary_s_to_save_active_buffer() {
        let modifiers = if cfg!(target_os = "macos") {
            Modifiers::new(false, false, false, true)
        } else {
            Modifiers::new(true, false, false, false)
        };
        let save = Keystroke::new(KeyCode::Char('s'), modifiers);
        assert_eq!(
            resolve_default(&save, &InputContext::new(FocusTarget::Editor)),
            InputResolution::command(CommandInvocation::from(WorkspaceAction::SaveActiveBuffer))
        );
    }

    #[test]
    fn same_keystroke_maps_to_different_commands_by_focus_target() {
        let enter = Keystroke::new(KeyCode::Enter, Modifiers::default());
        assert_eq!(
            resolve_default(&enter, &InputContext::new(FocusTarget::FileTreePanel)),
            InputResolution::command(CommandInvocation::from(
                zom_protocol::FileTreeAction::ActivateSelection
            ))
        );
        assert_eq!(
            resolve_default(&enter, &InputContext::new(FocusTarget::NotificationPanel)),
            InputResolution::command(CommandInvocation::from(
                NotificationAction::MarkSelectedRead
            ))
        );
        assert_eq!(
            resolve_default(&enter, &InputContext::new(FocusTarget::TerminalPanel)),
            InputResolution::command(CommandInvocation::from(WorkspaceAction::FocusPanel(
                FocusTarget::Editor
            )))
        );
    }
}
