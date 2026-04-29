mod defaults;
mod keymap;
mod shortcuts;

use std::sync::LazyLock;

use defaults::build_default_shortcut_registry;
use zom_protocol::{CommandInvocation, FocusTarget};

pub use keymap::Keymap;
pub use shortcuts::{
    ShortcutBinding, ShortcutBindingSpec, ShortcutRegistry, ShortcutScope,
    format_keystroke_for_display, format_scope_for_display,
};
pub use zom_protocol::keyboard::{
    EditorInputContext, InputContext, InputResolution, KeyCode, Keystroke, Modifiers,
};

/// 返回全局默认快捷键注册表（进程内单例）。
pub fn default_shortcut_registry() -> &'static ShortcutRegistry {
    &DEFAULT_SHORTCUT_REGISTRY
}

/// 查询命令对应的默认快捷键提示文本（如 `Cmd+S` / `Ctrl+S`）。
pub fn shortcut_hint(command: &CommandInvocation) -> Option<String> {
    default_shortcut_registry().shortcut_hint(command)
}

/// 基于默认注册表构造 `Keymap` 快照。
pub fn default_keymap() -> Keymap {
    Keymap::from_shortcut_registry(default_shortcut_registry())
}

static DEFAULT_SHORTCUT_REGISTRY: LazyLock<ShortcutRegistry> =
    LazyLock::new(build_default_shortcut_registry);
static DEFAULT_KEYMAP: LazyLock<Keymap> = LazyLock::new(default_keymap);

/// 默认输入解析入口：
/// 1) 先走声明式快捷键映射；2) 未命中时再尝试编辑器文本输入降级。
pub fn resolve_default(input: &Keystroke, context: &InputContext) -> InputResolution {
    let resolution = DEFAULT_KEYMAP.resolve(input, context);
    if !resolution.is_noop() {
        return resolution;
    }

    resolve_editor_text_fallback(input, context)
}

fn resolve_editor_text_fallback(input: &Keystroke, context: &InputContext) -> InputResolution {
    // 仅在编辑器焦点且无 ctrl/alt/cmd 时把字符输入降级为 InsertText，
    // 其余场景交给上层命令解析或保持 Noop。
    if context.focus != FocusTarget::Editor {
        return InputResolution::Noop;
    }

    if input.modifiers.has_ctrl || input.modifiers.has_alt || input.modifiers.has_cmd {
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
        WorkspaceAction,
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
    fn resolve_default_maps_clipboard_shortcuts_to_editor_commands() {
        let modifiers = if cfg!(target_os = "macos") {
            Modifiers::new(false, false, false, true)
        } else {
            Modifiers::new(true, false, false, false)
        };

        assert_eq!(
            resolve_default(
                &Keystroke::new(KeyCode::Char('c'), modifiers),
                &InputContext::new(FocusTarget::Editor)
            ),
            InputResolution::command(CommandInvocation::from(EditorAction::Copy))
        );
        assert_eq!(
            resolve_default(
                &Keystroke::new(KeyCode::Char('x'), modifiers),
                &InputContext::new(FocusTarget::Editor)
            ),
            InputResolution::command(CommandInvocation::from(EditorAction::Cut))
        );
        assert_eq!(
            resolve_default(
                &Keystroke::new(KeyCode::Char('v'), modifiers),
                &InputContext::new(FocusTarget::Editor)
            ),
            InputResolution::command(CommandInvocation::from(EditorAction::Paste))
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
            resolve_default(&enter, &InputContext::new(FocusTarget::TerminalPanel)),
            InputResolution::command(CommandInvocation::from(WorkspaceAction::FocusPanel(
                FocusTarget::Editor
            )))
        );
    }
}
