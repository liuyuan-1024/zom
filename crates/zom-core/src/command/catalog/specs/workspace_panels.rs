use crate::FocusTarget;
use crate::command::catalog::{
    CommandKey, CommandShortcut, CommandSpec, ShortcutScope,
    types::{ctrl_char, meta_char, meta_shift_char},
};
use crate::{Command, command::WorkspaceCommand};

pub const SPECS: &[CommandSpec] = &[
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::Editor),
        "workspace.focus_panel.editor",
        "Focus Editor",
        "Move focus to the editor pane.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::Editor,
            )))
        },
        &[],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::Palette),
        "workspace.focus_panel.palette",
        "Focus Command Palette",
        "Move focus to the command palette.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::Palette,
            )))
        },
        &[],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::FileTreePanel),
        "workspace.focus_panel.file_tree",
        "Focus File Tree Panel",
        "Show file tree panel and move focus to it.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::FileTreePanel,
            )))
        },
        &[CommandShortcut::new(ShortcutScope::Global, meta_char('b')).with_priority(100)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::GitPanel),
        "workspace.focus_panel.git",
        "Focus Git Panel",
        "Show Git panel and move focus to it.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::GitPanel,
            )))
        },
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('g')).with_priority(80)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::OutlinePanel),
        "workspace.focus_panel.outline",
        "Focus Outline Panel",
        "Show outline panel and move focus to it.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::OutlinePanel,
            )))
        },
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('o')).with_priority(80)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::ProjectSearch),
        "workspace.focus_panel.project_search",
        "Focus Project Search Panel",
        "Show project search panel and move focus to it.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::ProjectSearch,
            )))
        },
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('f')).with_priority(80)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::LanguageServers),
        "workspace.focus_panel.language_servers",
        "Focus Language Servers Panel",
        "Show language servers panel and move focus to it.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::LanguageServers,
            )))
        },
        &[],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::Terminal),
        "workspace.focus_panel.terminal",
        "Focus Terminal Panel",
        "Show terminal panel and move focus to it.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::Terminal,
            )))
        },
        &[CommandShortcut::new(ShortcutScope::Global, ctrl_char('`')).with_priority(80)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::DebugPanel),
        "workspace.focus_panel.debug",
        "Focus Debug Panel",
        "Show debug panel and move focus to it.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::DebugPanel,
            )))
        },
        &[],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceFocusPanel(FocusTarget::Notification),
        "workspace.focus_panel.notification",
        "Focus Notification Panel",
        "Show notification panel and move focus to it.",
        || {
            Some(Command::from(WorkspaceCommand::FocusPanel(
                FocusTarget::Notification,
            )))
        },
        &[CommandShortcut::new(ShortcutScope::Global, meta_shift_char('n')).with_priority(80)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::Editor),
        "workspace.toggle_panel.editor",
        "Toggle Editor",
        "Toggle editor visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::Editor,
            )))
        },
        &[],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::Palette),
        "workspace.toggle_panel.palette",
        "Toggle Command Palette",
        "Toggle command palette visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::Palette,
            )))
        },
        &[],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::FileTreePanel),
        "workspace.toggle_panel.file_tree",
        "Toggle File Tree Panel",
        "Toggle file tree panel visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::FileTreePanel,
            )))
        },
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::FileTreePanel),
            meta_char('w'),
        )
        .with_priority(120)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::GitPanel),
        "workspace.toggle_panel.git",
        "Toggle Git Panel",
        "Toggle Git panel visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::GitPanel,
            )))
        },
        &[
            CommandShortcut::new(ShortcutScope::Focus(FocusTarget::GitPanel), meta_char('w'))
                .with_priority(120),
        ],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::OutlinePanel),
        "workspace.toggle_panel.outline",
        "Toggle Outline Panel",
        "Toggle outline panel visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::OutlinePanel,
            )))
        },
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::OutlinePanel),
            meta_char('w'),
        )
        .with_priority(120)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::ProjectSearch),
        "workspace.toggle_panel.project_search",
        "Toggle Project Search Panel",
        "Toggle project search panel visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::ProjectSearch,
            )))
        },
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::ProjectSearch),
            meta_char('w'),
        )
        .with_priority(120)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::LanguageServers),
        "workspace.toggle_panel.language_servers",
        "Toggle Language Servers Panel",
        "Toggle language servers panel visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::LanguageServers,
            )))
        },
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::LanguageServers),
            meta_char('w'),
        )
        .with_priority(120)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::Terminal),
        "workspace.toggle_panel.terminal",
        "Toggle Terminal Panel",
        "Toggle terminal panel visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::Terminal,
            )))
        },
        &[
            CommandShortcut::new(ShortcutScope::Focus(FocusTarget::Terminal), meta_char('w'))
                .with_priority(120),
        ],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::DebugPanel),
        "workspace.toggle_panel.debug",
        "Toggle Debug Panel",
        "Toggle debug panel visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::DebugPanel,
            )))
        },
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::DebugPanel),
            meta_char('w'),
        )
        .with_priority(120)],
    ),
    CommandSpec::new(
        CommandKey::WorkspaceTogglePanel(FocusTarget::Notification),
        "workspace.toggle_panel.notification",
        "Toggle Notification Panel",
        "Toggle notification panel visibility.",
        || {
            Some(Command::from(WorkspaceCommand::TogglePanel(
                FocusTarget::Notification,
            )))
        },
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Notification),
            meta_char('w'),
        )
        .with_priority(120)],
    ),
];
