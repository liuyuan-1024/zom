use std::collections::HashMap;

use zom_core::{Command, FocusTarget, InputContext, InputResolution, Keystroke};

#[derive(Debug, Clone, Default)]
pub struct Keymap {
    global: HashMap<Keystroke, InputResolution>,
    editor_only: HashMap<Keystroke, InputResolution>,
}

impl Keymap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_global(&mut self, key: Keystroke, resolution: InputResolution) {
        self.global.insert(key, resolution);
    }

    pub fn bind_editor(&mut self, key: Keystroke, resolution: InputResolution) {
        self.editor_only.insert(key, resolution);
    }

    pub fn resolve(&self, input: &Keystroke, context: &InputContext) -> InputResolution {
        if context.focus == FocusTarget::Editor {
            if let Some(resolution) = self.editor_only.get(input) {
                return resolution.clone();
            }
        }

        self.global.get(input).cloned().unwrap_or(InputResolution::Noop)
    }
}

pub fn command(command: Command) -> InputResolution {
    InputResolution::Command(command)
}

#[cfg(test)]
mod tests {
    use super::{command, Keymap};
    use zom_core::{
        EditorCommand, EditorInputContext, FocusTarget, InputContext, InputResolution, KeyCode,
        Keystroke, Modifiers,
    };

    fn editor_context() -> InputContext {
        InputContext {
            focus: FocusTarget::Editor,
            in_text_input: false,
            command_palette_open: false,
            editor: Some(EditorInputContext {
                editable: true,
                read_only: false,
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

        assert_eq!(keymap.resolve(&key, &editor_context()), InputResolution::Noop);
    }
}
