mod defaults;
mod keymap;
mod shortcuts;

use std::sync::LazyLock;

use defaults::build_default_shortcut_registry;
pub use keymap::Keymap;
pub use shortcuts::{ShortcutAction, ShortcutBinding, ShortcutRegistry, ShortcutScope};
use zom_core::{Command, InputContext, InputResolution, Keystroke};

pub fn command(command: Command) -> InputResolution {
    InputResolution::Command(command)
}

/// 读取默认快捷键注册表（单例）。
pub fn default_shortcut_registry() -> &'static ShortcutRegistry {
    &DEFAULT_SHORTCUT_REGISTRY
}

/// 读取某个语义动作对应的默认快捷键文案。
pub fn shortcut_hint(action: ShortcutAction) -> Option<String> {
    default_shortcut_registry().shortcut_hint(action)
}

pub fn default_keymap() -> Keymap {
    Keymap::from_shortcut_registry(default_shortcut_registry())
}

static DEFAULT_SHORTCUT_REGISTRY: LazyLock<ShortcutRegistry> =
    LazyLock::new(build_default_shortcut_registry);
static DEFAULT_KEYMAP: LazyLock<Keymap> = LazyLock::new(default_keymap);

/// 使用默认键位方案解析一次输入。
pub fn resolve_default(input: &Keystroke, context: &InputContext) -> InputResolution {
    DEFAULT_KEYMAP.resolve(input, context)
}

#[cfg(test)]
mod tests {
    use super::{
        Keymap, ShortcutAction, ShortcutScope, command, default_keymap, default_shortcut_registry,
        shortcut_hint,
    };
    use zom_core::{
        Command, EditorCommand, EditorInputContext, FocusTarget, InputContext, InputResolution,
        KeyCode, Keystroke, Modifiers,
        command::{FileTreeCommand, WorkspaceCommand},
    };

    fn editor_context() -> InputContext {
        InputContext {
            focus: FocusTarget::Editor,
            is_in_text_input: false,
            is_command_palette_open: false,
            editor: Some(EditorInputContext {
                is_editable: true,
                is_read_only: false,
                has_selection: false,
            }),
        }
    }

    #[test]
    fn resolves_editor_binding_first() {
        let mut keymap = Keymap::new();
        let key = Keystroke {
            key: KeyCode::Char('x'),
            modifiers: Modifiers::default(),
        };

        keymap.bind_global(key.clone(), InputResolution::InsertText("global".into()));
        keymap.bind_editor(
            key.clone(),
            command(zom_core::Command::Editor(EditorCommand::DeleteBackward)),
        );

        assert_eq!(
            keymap.resolve(&key, &editor_context()),
            InputResolution::Command(zom_core::Command::Editor(EditorCommand::DeleteBackward))
        );
    }

    #[test]
    fn returns_noop_when_no_binding() {
        let keymap = Keymap::new();
        let key = Keystroke {
            key: KeyCode::Escape,
            modifiers: Modifiers::default(),
        };

        assert_eq!(
            keymap.resolve(&key, &editor_context()),
            InputResolution::Noop
        );
    }

    #[test]
    fn default_keymap_resolves_file_tree_scoped_navigation() {
        let keymap = default_keymap();
        let key = Keystroke::new(KeyCode::Down, Modifiers::default());
        let context = InputContext::new(FocusTarget::FileTreePanel);

        assert_eq!(
            keymap.resolve(&key, &context),
            InputResolution::Command(Command::from(FileTreeCommand::SelectNext))
        );
    }

    #[test]
    fn default_keymap_resolves_global_file_tree_focus() {
        let keymap = default_keymap();
        let key = Keystroke::new(
            KeyCode::Char('b'),
            Modifiers::new(false, false, false, true),
        );
        let context = InputContext::new(FocusTarget::Editor);

        assert_eq!(
            keymap.resolve(&key, &context),
            InputResolution::Command(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::FileTreePanel,
            )))
        );
    }

    #[test]
    fn default_keymap_resolves_panel_close_shortcut_for_focused_file_tree() {
        let keymap = default_keymap();
        let key = Keystroke::new(
            KeyCode::Char('w'),
            Modifiers::new(false, false, false, true),
        );
        let context = InputContext::new(FocusTarget::FileTreePanel);

        assert_eq!(
            keymap.resolve(&key, &context),
            InputResolution::Command(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::FileTreePanel,
            )))
        );
    }

    #[test]
    fn default_shortcut_registry_contains_file_tree_focus_shortcut() {
        let registry = default_shortcut_registry();
        let binding = registry
            .bindings()
            .iter()
            .find(|binding| binding.action == ShortcutAction::FocusFileTreePanel)
            .expect("file tree focus shortcut should exist");

        assert_eq!(binding.scope, ShortcutScope::Global);
        assert_eq!(
            binding.keystroke,
            Keystroke::new(
                KeyCode::Char('b'),
                Modifiers::new(false, false, false, true),
            )
        );
    }

    #[test]
    fn shortcut_hint_uses_registry_definition() {
        assert_eq!(
            shortcut_hint(ShortcutAction::FocusFileTreePanel),
            Some("Cmd+B".to_string())
        );
        assert_eq!(
            shortcut_hint(ShortcutAction::FocusGitPanel),
            Some("Cmd+Shift+G".to_string())
        );
        assert_eq!(
            shortcut_hint(ShortcutAction::FocusOutlinePanel),
            Some("Cmd+Shift+O".to_string())
        );
        assert_eq!(
            shortcut_hint(ShortcutAction::FocusProjectSearchPanel),
            Some("Cmd+Shift+F".to_string())
        );
        assert_eq!(
            shortcut_hint(ShortcutAction::FocusTerminalPanel),
            Some("Ctrl+`".to_string())
        );
        assert_eq!(
            shortcut_hint(ShortcutAction::OpenProjectFromTitleBar),
            Some("Cmd+Shift+P".to_string())
        );
        assert_eq!(
            shortcut_hint(ShortcutAction::OpenSettingsFromTitleBar),
            Some("Cmd+,".to_string())
        );
        assert_eq!(
            shortcut_hint(ShortcutAction::HideFocusedPanel),
            Some("Cmd+W".to_string())
        );
    }
}
