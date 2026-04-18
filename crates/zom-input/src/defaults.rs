use zom_core::{
    Command, FocusTarget, InputResolution, KeyCode, Keystroke, Modifiers,
    command::{FileTreeCommand, WorkspaceCommand},
};

use crate::{ShortcutAction, ShortcutBinding, ShortcutRegistry, ShortcutScope, command};

/// 构造默认快捷键注册表（构建函数）。
pub(crate) fn build_default_shortcut_registry() -> ShortcutRegistry {
    let mut registry = ShortcutRegistry::new();
    register_workspace_shortcuts(&mut registry);
    register_file_tree_shortcuts(&mut registry);
    registry
}

fn register_workspace_shortcuts(registry: &mut ShortcutRegistry) {
    registry.register(ShortcutBinding::new(
        ShortcutAction::FocusFileTreePanel,
        ShortcutScope::Global,
        Keystroke::new(
            KeyCode::Char('b'),
            Modifiers::new(false, false, false, true),
        ),
        command(Command::from(WorkspaceCommand::FocusPanel(
            FocusTarget::FileTreePanel,
        ))),
    ));

    register_tooltip_only_shortcuts(registry);
    register_panel_close_shortcuts(registry);
}

fn register_tooltip_only_shortcuts(registry: &mut ShortcutRegistry) {
    let tooltip_only_bindings = [
        (
            ShortcutAction::FocusGitPanel,
            Keystroke::new(
                KeyCode::Char('g'),
                Modifiers::new(false, false, true, true),
            ),
        ),
        (
            ShortcutAction::FocusOutlinePanel,
            Keystroke::new(
                KeyCode::Char('o'),
                Modifiers::new(false, false, true, true),
            ),
        ),
        (
            ShortcutAction::FocusProjectSearchPanel,
            Keystroke::new(
                KeyCode::Char('f'),
                Modifiers::new(false, false, true, true),
            ),
        ),
        (
            ShortcutAction::FocusTerminalPanel,
            Keystroke::new(
                KeyCode::Char('`'),
                Modifiers::new(true, false, false, false),
            ),
        ),
        (
            ShortcutAction::OpenProjectFromTitleBar,
            Keystroke::new(
                KeyCode::Char('p'),
                Modifiers::new(false, false, true, true),
            ),
        ),
        (
            ShortcutAction::OpenSettingsFromTitleBar,
            Keystroke::new(
                KeyCode::Char(','),
                Modifiers::new(false, false, false, true),
            ),
        ),
    ];

    for (action, keystroke) in tooltip_only_bindings {
        registry.register(ShortcutBinding::new(
            action,
            ShortcutScope::Global,
            keystroke,
            InputResolution::Noop,
        ));
    }
}

fn register_file_tree_shortcuts(registry: &mut ShortcutRegistry) {
    let file_tree_bindings = [
        (
            ShortcutAction::FileTreeSelectPrev,
            KeyCode::Up,
            FileTreeCommand::SelectPrev,
        ),
        (
            ShortcutAction::FileTreeSelectNext,
            KeyCode::Down,
            FileTreeCommand::SelectNext,
        ),
        (
            ShortcutAction::FileTreeExpandOrDescend,
            KeyCode::Right,
            FileTreeCommand::ExpandOrDescend,
        ),
        (
            ShortcutAction::FileTreeCollapseOrAscend,
            KeyCode::Left,
            FileTreeCommand::CollapseOrAscend,
        ),
        (
            ShortcutAction::FileTreeActivateSelection,
            KeyCode::Enter,
            FileTreeCommand::ActivateSelection,
        ),
    ];

    for (action, key, file_tree_command) in file_tree_bindings {
        registry.register(ShortcutBinding::new(
            action,
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            Keystroke::new(key, Modifiers::default()),
            command(Command::from(file_tree_command)),
        ));
    }
}

fn register_panel_close_shortcuts(registry: &mut ShortcutRegistry) {
    let close_shortcut = Keystroke::new(
        KeyCode::Char('w'),
        Modifiers::new(false, false, false, true),
    );

    for panel in FocusTarget::VISIBILITY_MANAGED_PANELS {
        registry.register(ShortcutBinding::new(
            ShortcutAction::HideFocusedPanel,
            ShortcutScope::Focus(panel),
            close_shortcut.clone(),
            command(Command::from(WorkspaceCommand::TogglePanel(panel))),
        ));
    }
}
