mod defaults;
mod keymap;
mod shortcuts;

use std::sync::LazyLock;

use defaults::build_default_shortcut_registry;
pub use keymap::Keymap;
pub use shortcuts::{
    ShortcutBinding, ShortcutBindingSpec, ShortcutPlatform, ShortcutRegistry, ShortcutScope,
    ShortcutWhen,
};
use zom_core::{Command, InputContext, InputResolution, Keystroke};

pub fn command(command: Command) -> InputResolution {
    InputResolution::Command(command)
}

/// 读取默认快捷键注册表（单例）。
pub fn default_shortcut_registry() -> &'static ShortcutRegistry {
    &DEFAULT_SHORTCUT_REGISTRY
}

/// 读取某个命令对应的默认快捷键文案。
pub fn shortcut_hint(command: &Command) -> Option<String> {
    default_shortcut_registry().shortcut_hint(command)
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
        Keymap, ShortcutPlatform, ShortcutScope, ShortcutWhen, command, default_keymap,
        default_shortcut_registry, shortcut_hint,
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

    fn focus_panel_command(target: FocusTarget) -> Command {
        Command::from(WorkspaceCommand::FocusPanel(target))
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
            InputResolution::Command(focus_panel_command(FocusTarget::FileTreePanel))
        );
    }

    #[test]
    fn default_keymap_resolves_open_project_shortcut() {
        let keymap = default_keymap();
        let key = Keystroke::new(KeyCode::Char('p'), Modifiers::new(false, false, true, true));
        let context = InputContext::new(FocusTarget::Editor);

        assert_eq!(
            keymap.resolve(&key, &context),
            InputResolution::Command(Command::from(WorkspaceCommand::OpenProjectPicker))
        );
    }

    #[test]
    fn default_keymap_resolves_open_settings_shortcut() {
        let keymap = default_keymap();
        let key = Keystroke::new(
            KeyCode::Char(','),
            Modifiers::new(false, false, false, true),
        );
        let context = InputContext::new(FocusTarget::Editor);

        assert_eq!(
            keymap.resolve(&key, &context),
            InputResolution::Command(Command::from(WorkspaceCommand::OpenSettings))
        );
    }

    #[test]
    fn default_keymap_resolves_start_debugging_shortcut() {
        let keymap = default_keymap();
        let key = Keystroke::new(KeyCode::F(5), Modifiers::default());
        let context = InputContext::new(FocusTarget::Editor);

        assert_eq!(
            keymap.resolve(&key, &context),
            InputResolution::Command(Command::from(WorkspaceCommand::StartDebugging))
        );
    }

    #[test]
    fn default_keymap_resolves_notification_focus_shortcut() {
        let keymap = default_keymap();
        let key = Keystroke::new(KeyCode::Char('n'), Modifiers::new(false, false, true, true));
        let context = InputContext::new(FocusTarget::Editor);

        assert_eq!(
            keymap.resolve(&key, &context),
            InputResolution::Command(focus_panel_command(FocusTarget::Notification))
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
            .find(|binding| binding.command == focus_panel_command(FocusTarget::FileTreePanel))
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
            shortcut_hint(&focus_panel_command(FocusTarget::FileTreePanel)),
            Some("Cmd+B".to_string())
        );
        assert_eq!(
            shortcut_hint(&focus_panel_command(FocusTarget::GitPanel)),
            Some("Cmd+Shift+G".to_string())
        );
        assert_eq!(
            shortcut_hint(&focus_panel_command(FocusTarget::OutlinePanel)),
            Some("Cmd+Shift+O".to_string())
        );
        assert_eq!(
            shortcut_hint(&focus_panel_command(FocusTarget::ProjectSearch)),
            Some("Cmd+Shift+F".to_string())
        );
        assert_eq!(
            shortcut_hint(&focus_panel_command(FocusTarget::Terminal)),
            Some("Ctrl+`".to_string())
        );
        assert_eq!(
            shortcut_hint(&Command::from(WorkspaceCommand::OpenProjectPicker)),
            Some("Cmd+Shift+P".to_string())
        );
        assert_eq!(
            shortcut_hint(&Command::from(WorkspaceCommand::OpenSettings)),
            Some("Cmd+,".to_string())
        );
        assert_eq!(
            shortcut_hint(&Command::from(WorkspaceCommand::OpenCodeActions)),
            Some("Cmd+.".to_string())
        );
        assert_eq!(
            shortcut_hint(&Command::from(WorkspaceCommand::StartDebugging)),
            Some("F5".to_string())
        );
        assert_eq!(
            shortcut_hint(&focus_panel_command(FocusTarget::Notification)),
            Some("Cmd+Shift+N".to_string())
        );
        assert_eq!(
            shortcut_hint(&Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::FileTreePanel,
            ))),
            Some("Cmd+W".to_string())
        );
    }

    #[test]
    fn default_binding_metadata_is_structured() {
        let registry = default_shortcut_registry();
        let file_tree_focus = registry
            .bindings()
            .iter()
            .find(|binding| binding.command == focus_panel_command(FocusTarget::FileTreePanel))
            .expect("file tree focus binding should exist");

        assert_eq!(file_tree_focus.when, ShortcutWhen::Always);
        assert_eq!(file_tree_focus.platform, ShortcutPlatform::Any);
        assert_eq!(file_tree_focus.priority, 100);
    }
}
