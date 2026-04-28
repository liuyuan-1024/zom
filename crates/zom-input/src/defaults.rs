//! 默认快捷键注册表构建逻辑（按 `CommandKindId` 维护）。

use zom_protocol::{
    CommandInvocation, CommandKindId, EditorAction, FileTreeAction, FocusTarget, KeyCode,
    Keystroke, Modifiers, NotificationAction, OverlayTarget, TabAction, WorkspaceAction,
};

use crate::{InputResolution, ShortcutBinding, ShortcutRegistry, ShortcutScope};

/// 默认快捷键声明（语义 ID + 作用域 + 按键）。
#[derive(Clone, Copy)]
struct DefaultShortcutSpec {
    command_id: CommandKindId,
    scope: ShortcutScope,
    keystroke: Keystroke,
    priority: u8,
}

impl DefaultShortcutSpec {
    const fn new(
        command_id: CommandKindId,
        scope: ShortcutScope,
        keystroke: Keystroke,
        priority: u8,
    ) -> Self {
        Self {
            command_id,
            scope,
            keystroke,
            priority,
        }
    }
}

/// 构造默认快捷键注册表（构建函数）。
pub(crate) fn build_default_shortcut_registry() -> ShortcutRegistry {
    let mut registry = ShortcutRegistry::new();
    for spec in DEFAULT_SHORTCUT_SPECS {
        let Some(command) = command_from_kind_id(spec.command_id) else {
            continue;
        };
        registry.register(ShortcutBinding {
            command: command.clone(),
            scope: spec.scope,
            keystroke: spec.keystroke,
            priority: spec.priority,
            resolution: InputResolution::Command(command),
        });
    }
    registry
}

fn command_from_kind_id(command_id: CommandKindId) -> Option<CommandInvocation> {
    match command_id.0 {
        "editor.insert_text" => None,
        "editor.insert_newline" => Some(CommandInvocation::from(EditorAction::InsertNewline)),
        "editor.insert_indent" => Some(CommandInvocation::from(EditorAction::InsertIndent)),
        "editor.outdent" => Some(CommandInvocation::from(EditorAction::Outdent)),
        "editor.move_left" => Some(CommandInvocation::from(EditorAction::MoveLeft)),
        "editor.move_right" => Some(CommandInvocation::from(EditorAction::MoveRight)),
        "editor.move_up" => Some(CommandInvocation::from(EditorAction::MoveUp)),
        "editor.move_down" => Some(CommandInvocation::from(EditorAction::MoveDown)),
        "editor.move_to_start" => Some(CommandInvocation::from(EditorAction::MoveToStart)),
        "editor.move_to_end" => Some(CommandInvocation::from(EditorAction::MoveToEnd)),
        "editor.page_up" => Some(CommandInvocation::from(EditorAction::MovePageUp)),
        "editor.page_down" => Some(CommandInvocation::from(EditorAction::MovePageDown)),
        "editor.select_left" => Some(CommandInvocation::from(EditorAction::SelectLeft)),
        "editor.select_right" => Some(CommandInvocation::from(EditorAction::SelectRight)),
        "editor.select_up" => Some(CommandInvocation::from(EditorAction::SelectUp)),
        "editor.select_down" => Some(CommandInvocation::from(EditorAction::SelectDown)),
        "editor.select_to_start" => Some(CommandInvocation::from(EditorAction::SelectToStart)),
        "editor.select_to_end" => Some(CommandInvocation::from(EditorAction::SelectToEnd)),
        "editor.select_page_up" => Some(CommandInvocation::from(EditorAction::SelectPageUp)),
        "editor.select_page_down" => Some(CommandInvocation::from(EditorAction::SelectPageDown)),
        "editor.select_all" => Some(CommandInvocation::from(EditorAction::SelectAll)),
        "editor.delete_backward" => Some(CommandInvocation::from(EditorAction::DeleteBackward)),
        "editor.delete_forward" => Some(CommandInvocation::from(EditorAction::DeleteForward)),
        "editor.delete_word_backward" => {
            Some(CommandInvocation::from(EditorAction::DeleteWordBackward))
        }
        "editor.delete_word_forward" => {
            Some(CommandInvocation::from(EditorAction::DeleteWordForward))
        }
        "editor.undo" => Some(CommandInvocation::from(EditorAction::Undo)),
        "editor.redo" => Some(CommandInvocation::from(EditorAction::Redo)),
        "workspace.quit_app" => Some(CommandInvocation::from(WorkspaceAction::QuitApp)),
        "workspace.minimize_window" => {
            Some(CommandInvocation::from(WorkspaceAction::MinimizeWindow))
        }
        "workspace.save_active_buffer" => {
            Some(CommandInvocation::from(WorkspaceAction::SaveActiveBuffer))
        }
        "workspace.open_project_picker" => {
            Some(CommandInvocation::from(WorkspaceAction::OpenProjectPicker))
        }
        "workspace.close_focused" => Some(CommandInvocation::from(WorkspaceAction::CloseFocused)),
        "workspace.focus_panel.editor" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::Editor),
        )),
        "workspace.focus_panel.palette" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::Palette),
        )),
        "workspace.focus_panel.file_tree" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::FileTreePanel),
        )),
        "workspace.focus_panel.git" => Some(CommandInvocation::from(WorkspaceAction::FocusPanel(
            FocusTarget::GitPanel,
        ))),
        "workspace.focus_panel.outline" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::OutlinePanel),
        )),
        "workspace.focus_panel.project_search" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::ProjectSearchPanel),
        )),
        "workspace.focus_panel.language_servers" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::LanguageServersPanel),
        )),
        "workspace.focus_panel.terminal" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::TerminalPanel),
        )),
        "workspace.focus_panel.debug" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::DebugPanel),
        )),
        "workspace.focus_panel.notification" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::NotificationPanel),
        )),
        "workspace.focus_overlay.settings" => Some(CommandInvocation::from(
            WorkspaceAction::FocusOverlay(OverlayTarget::Settings),
        )),
        "workspace.file_tree.select_prev" => {
            Some(CommandInvocation::from(FileTreeAction::SelectPrev))
        }
        "workspace.file_tree.select_next" => {
            Some(CommandInvocation::from(FileTreeAction::SelectNext))
        }
        "workspace.file_tree.expand_or_descend" => {
            Some(CommandInvocation::from(FileTreeAction::ExpandOrDescend))
        }
        "workspace.file_tree.collapse_or_ascend" => {
            Some(CommandInvocation::from(FileTreeAction::CollapseOrAscend))
        }
        "workspace.file_tree.activate_selection" => {
            Some(CommandInvocation::from(FileTreeAction::ActivateSelection))
        }
        "workspace.tab.close_active" => Some(CommandInvocation::from(TabAction::CloseActiveTab)),
        "workspace.tab.activate_prev" => Some(CommandInvocation::from(TabAction::ActivatePrevTab)),
        "workspace.tab.activate_next" => Some(CommandInvocation::from(TabAction::ActivateNextTab)),
        "workspace.notification.mark_selected_read" => Some(CommandInvocation::from(
            NotificationAction::MarkSelectedRead,
        )),
        "workspace.notification.clear_all" => {
            Some(CommandInvocation::from(NotificationAction::ClearAll))
        }
        "workspace.notification.clear_read" => {
            Some(CommandInvocation::from(NotificationAction::ClearRead))
        }
        "workspace.notification.focus_unread_error" => Some(CommandInvocation::from(
            NotificationAction::FocusUnreadError,
        )),
        "workspace.notification.select_prev" => {
            Some(CommandInvocation::from(NotificationAction::SelectPrev))
        }
        "workspace.notification.select_next" => {
            Some(CommandInvocation::from(NotificationAction::SelectNext))
        }
        _ => None,
    }
}

const DEFAULT_SHORTCUT_SPECS: &[DefaultShortcutSpec] = &[
    // editor
    DefaultShortcutSpec::new(
        CommandKindId("editor.insert_newline"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::Enter),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.insert_indent"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::Tab),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.outdent"),
        ShortcutScope::Focus(FocusTarget::Editor),
        shift(KeyCode::Tab),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.move_left"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::Left),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.move_right"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::Right),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.move_up"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::Up),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.move_down"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::Down),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.move_to_start"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::Home),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.move_to_end"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::End),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.page_up"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::PageUp),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.page_down"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::PageDown),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.select_left"),
        ShortcutScope::Focus(FocusTarget::Editor),
        shift(KeyCode::Left),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.select_right"),
        ShortcutScope::Focus(FocusTarget::Editor),
        shift(KeyCode::Right),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.select_up"),
        ShortcutScope::Focus(FocusTarget::Editor),
        shift(KeyCode::Up),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.select_down"),
        ShortcutScope::Focus(FocusTarget::Editor),
        shift(KeyCode::Down),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.select_to_start"),
        ShortcutScope::Focus(FocusTarget::Editor),
        shift(KeyCode::Home),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.select_to_end"),
        ShortcutScope::Focus(FocusTarget::Editor),
        shift(KeyCode::End),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.select_page_up"),
        ShortcutScope::Focus(FocusTarget::Editor),
        shift(KeyCode::PageUp),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.select_page_down"),
        ShortcutScope::Focus(FocusTarget::Editor),
        shift(KeyCode::PageDown),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.select_all"),
        ShortcutScope::Focus(FocusTarget::Editor),
        primary_char('a'),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.delete_backward"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::Backspace),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.delete_forward"),
        ShortcutScope::Focus(FocusTarget::Editor),
        plain(KeyCode::Delete),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.undo"),
        ShortcutScope::Focus(FocusTarget::Editor),
        primary_char('z'),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.redo"),
        ShortcutScope::Focus(FocusTarget::Editor),
        primary_shift_char('z'),
        120,
    ),
    // workspace actions
    DefaultShortcutSpec::new(
        CommandKindId("workspace.quit_app"),
        ShortcutScope::Global,
        primary_char('q'),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.minimize_window"),
        ShortcutScope::Global,
        primary_char('m'),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.open_project_picker"),
        ShortcutScope::Global,
        primary_shift_char('p'),
        80,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.save_active_buffer"),
        ShortcutScope::Global,
        primary_char('s'),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.close_focused"),
        ShortcutScope::Global,
        primary_char('w'),
        120,
    ),
    // workspace pane/panels/overlays
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.editor"),
        ShortcutScope::Global,
        primary_char('e'),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.palette"),
        ShortcutScope::Global,
        primary_char('p'),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.file_tree"),
        ShortcutScope::Global,
        primary_shift_char('e'),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.git"),
        ShortcutScope::Global,
        primary_shift_char('g'),
        80,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.outline"),
        ShortcutScope::Global,
        primary_shift_char('o'),
        80,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.project_search"),
        ShortcutScope::Global,
        primary_shift_char('f'),
        80,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.language_servers"),
        ShortcutScope::Global,
        primary_shift_char('l'),
        80,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.terminal"),
        ShortcutScope::Global,
        primary_char('.'),
        80,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.debug"),
        ShortcutScope::Global,
        primary_shift_char('d'),
        80,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.notification"),
        ShortcutScope::Global,
        primary_shift_char('n'),
        80,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_overlay.settings"),
        ShortcutScope::Global,
        primary_char(','),
        80,
    ),
    // notification
    DefaultShortcutSpec::new(
        CommandKindId("workspace.notification.clear_all"),
        ShortcutScope::Focus(FocusTarget::NotificationPanel),
        primary_char('k'),
        70,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.notification.clear_read"),
        ShortcutScope::Focus(FocusTarget::NotificationPanel),
        primary_char('u'),
        70,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.notification.focus_unread_error"),
        ShortcutScope::Focus(FocusTarget::NotificationPanel),
        primary_char('x'),
        70,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.notification.select_prev"),
        ShortcutScope::Focus(FocusTarget::NotificationPanel),
        plain(KeyCode::Up),
        110,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.notification.select_next"),
        ShortcutScope::Focus(FocusTarget::NotificationPanel),
        plain(KeyCode::Down),
        110,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.notification.mark_selected_read"),
        ShortcutScope::Focus(FocusTarget::NotificationPanel),
        plain(KeyCode::Enter),
        110,
    ),
    // terminal scoped (placeholder)
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.editor"),
        ShortcutScope::Focus(FocusTarget::TerminalPanel),
        plain(KeyCode::Enter),
        110,
    ),
    // file tree scoped
    DefaultShortcutSpec::new(
        CommandKindId("workspace.file_tree.select_prev"),
        ShortcutScope::Focus(FocusTarget::FileTreePanel),
        plain(KeyCode::Up),
        110,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.file_tree.select_next"),
        ShortcutScope::Focus(FocusTarget::FileTreePanel),
        plain(KeyCode::Down),
        110,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.file_tree.expand_or_descend"),
        ShortcutScope::Focus(FocusTarget::FileTreePanel),
        plain(KeyCode::Right),
        110,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.file_tree.collapse_or_ascend"),
        ShortcutScope::Focus(FocusTarget::FileTreePanel),
        plain(KeyCode::Left),
        110,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.file_tree.activate_selection"),
        ShortcutScope::Focus(FocusTarget::FileTreePanel),
        plain(KeyCode::Enter),
        110,
    ),
    // tab
    DefaultShortcutSpec::new(
        CommandKindId("workspace.tab.activate_prev"),
        ShortcutScope::Global,
        primary_char('h'),
        0,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.tab.activate_next"),
        ShortcutScope::Global,
        primary_char('l'),
        0,
    ),
];

const fn plain(key: KeyCode) -> Keystroke {
    Keystroke::new(key, Modifiers::new(false, false, false, false))
}

const fn shift(key: KeyCode) -> Keystroke {
    Keystroke::new(key, Modifiers::new(false, false, true, false))
}

const fn primary_char(c: char) -> Keystroke {
    Keystroke::new(
        KeyCode::Char(c),
        with_logical_modifiers(false, true, false, false),
    )
}

const fn primary_shift_char(c: char) -> Keystroke {
    Keystroke::new(
        KeyCode::Char(c),
        with_logical_modifiers(true, true, false, false),
    )
}

const fn with_logical_modifiers(
    is_shift_pressed: bool,
    has_primary_modifier: bool,
    has_secondary_modifier: bool,
    has_word_nav_modifier: bool,
) -> Modifiers {
    let mut modifiers = Modifiers::new(false, false, is_shift_pressed, false);
    if has_primary_modifier {
        modifiers = merge_modifiers(modifiers, primary_modifier());
    }
    if has_secondary_modifier {
        modifiers = merge_modifiers(modifiers, secondary_modifier());
    }
    if has_word_nav_modifier {
        modifiers = merge_modifiers(modifiers, word_nav_modifier());
    }
    modifiers
}

const fn merge_modifiers(base: Modifiers, extra: Modifiers) -> Modifiers {
    Modifiers::new(
        base.has_ctrl || extra.has_ctrl,
        base.has_alt || extra.has_alt,
        base.has_shift || extra.has_shift,
        base.has_meta || extra.has_meta,
    )
}

const fn primary_modifier() -> Modifiers {
    #[cfg(target_os = "macos")]
    {
        Modifiers::new(false, false, false, true)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Modifiers::new(true, false, false, false)
    }
}

const fn secondary_modifier() -> Modifiers {
    #[cfg(target_os = "macos")]
    {
        Modifiers::new(true, false, false, false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Modifiers::new(false, true, false, false)
    }
}

const fn word_nav_modifier() -> Modifiers {
    #[cfg(target_os = "macos")]
    {
        Modifiers::new(false, true, false, false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Modifiers::new(true, false, false, false)
    }
}
