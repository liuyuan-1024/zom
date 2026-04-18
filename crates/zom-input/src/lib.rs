use std::{collections::HashMap, sync::LazyLock};

use zom_core::{
    Command, FocusTarget, InputContext, InputResolution, KeyCode, Keystroke, Modifiers,
    command::{FileTreeCommand, WorkspaceCommand},
};

#[derive(Debug, Clone, Default)]
pub struct Keymap {
    /// 全局快捷键
    global: HashMap<Keystroke, InputResolution>,
    /// 焦点作用域快捷键
    scoped: HashMap<FocusTarget, HashMap<Keystroke, InputResolution>>,
}

impl Keymap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_global(&mut self, key: Keystroke, resolution: InputResolution) {
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

pub fn command(command: Command) -> InputResolution {
    InputResolution::Command(command)
}

pub fn default_keymap() -> Keymap {
    let mut keymap = Keymap::new();
    bind_workspace_defaults(&mut keymap);
    bind_file_tree_defaults(&mut keymap);
    keymap
}

fn bind_workspace_defaults(keymap: &mut Keymap) {
    bind_panel_focus_shortcut(
        keymap,
        FocusTarget::FileTreePanel,
        Keystroke::new(
            KeyCode::Char('e'),
            Modifiers::new(false, false, false, true),
        ),
    );
    bind_panel_close_shortcuts(keymap);
}

fn bind_file_tree_defaults(keymap: &mut Keymap) {
    let file_tree_bindings = [
        (KeyCode::Up, FileTreeCommand::SelectPrev),
        (KeyCode::Down, FileTreeCommand::SelectNext),
        (KeyCode::Right, FileTreeCommand::ExpandOrDescend),
        (KeyCode::Left, FileTreeCommand::CollapseOrAscend),
        (KeyCode::Enter, FileTreeCommand::ActivateSelection),
    ];

    for (key, file_tree_command) in file_tree_bindings {
        keymap.bind_file_tree(
            Keystroke::new(key, Modifiers::default()),
            command(Command::from(file_tree_command)),
        );
    }
}

fn bind_panel_focus_shortcut(keymap: &mut Keymap, target: FocusTarget, key: Keystroke) {
    keymap.bind_global(
        key,
        command(Command::from(WorkspaceCommand::FocusPanel(target))),
    );
}

fn bind_panel_close_shortcuts(keymap: &mut Keymap) {
    let close_shortcut = Keystroke::new(
        KeyCode::Char('w'),
        Modifiers::new(false, false, false, true),
    );

    for panel in FocusTarget::VISIBILITY_MANAGED_PANELS {
        keymap.bind_for_focus(
            panel,
            close_shortcut.clone(),
            command(Command::from(WorkspaceCommand::TogglePanel(panel))),
        );
    }
}

static DEFAULT_KEYMAP: LazyLock<Keymap> = LazyLock::new(default_keymap);

/// 使用默认键位方案解析一次输入。
pub fn resolve_default(input: &Keystroke, context: &InputContext) -> InputResolution {
    DEFAULT_KEYMAP.resolve(input, context)
}

#[cfg(test)]
mod tests {
    use super::{Keymap, command, default_keymap};
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
}
