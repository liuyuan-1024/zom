use zom_protocol::{
    CommandInvocation, CommandKindId, EditorAction, FileTreeAction, FocusTarget, KeyCode,
    Keystroke, Modifiers, NotificationAction, OverlayTarget, TabAction, WorkspaceAction,
};

use crate::{InputResolution, ShortcutBinding, ShortcutRegistry, ShortcutScope};

#[derive(Clone, Copy)]
struct DefaultShortcutSpec {
    /// 协议层命令 ID；用于把静态表映射到具体 `CommandInvocation`。
    command_id: CommandKindId,
    /// 绑定作用域（全局或焦点限定）。
    scope: ShortcutScope,
    /// 触发按键。
    keystroke: Keystroke,
    /// 冲突优先级（保留给上层裁决策略）。
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

pub(crate) fn build_default_shortcut_registry() -> ShortcutRegistry {
    // 默认快捷键表是“声明式规格 -> 可执行绑定”的一次性编译过程。
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
    // 需要运行时载荷的命令（如 insert_text）不在静态快捷键表直接构造。
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
        "editor.copy" => Some(CommandInvocation::from(EditorAction::Copy)),
        "editor.cut" => Some(CommandInvocation::from(EditorAction::Cut)),
        "editor.paste" => Some(CommandInvocation::from(EditorAction::Paste)),
        "editor.undo" => Some(CommandInvocation::from(EditorAction::Undo)),
        "editor.redo" => Some(CommandInvocation::from(EditorAction::Redo)),
        "editor.open_find_replace" => Some(CommandInvocation::from(EditorAction::OpenFindReplace)),
        "editor.find_replace.toggle_case_sensitive" => Some(CommandInvocation::from(
            EditorAction::ToggleFindCaseSensitive,
        )),
        "editor.find_replace.toggle_whole_word" => {
            Some(CommandInvocation::from(EditorAction::ToggleFindWholeWord))
        }
        "editor.find_replace.toggle_regex" => {
            Some(CommandInvocation::from(EditorAction::ToggleFindRegex))
        }
        "editor.find_prev" => Some(CommandInvocation::from(EditorAction::FindPrev)),
        "editor.find_next" => Some(CommandInvocation::from(EditorAction::FindNext)),
        "editor.replace_next" => Some(CommandInvocation::from(EditorAction::ReplaceNext)),
        "editor.replace_all" => Some(CommandInvocation::from(EditorAction::ReplaceAll)),
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
        "workspace.focus_panel.shortcut" => Some(CommandInvocation::from(
            WorkspaceAction::FocusPanel(FocusTarget::ShortcutPanel),
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
        CommandKindId("editor.copy"),
        ShortcutScope::Focus(FocusTarget::Editor),
        primary_char('c'),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.cut"),
        ShortcutScope::Focus(FocusTarget::Editor),
        primary_char('x'),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.paste"),
        ShortcutScope::Focus(FocusTarget::Editor),
        primary_char('v'),
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
        CommandKindId("editor.open_find_replace"),
        ShortcutScope::Global,
        primary_char('f'),
        120,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.find_replace.toggle_case_sensitive"),
        ShortcutScope::Focus(FocusTarget::FindReplaceOverlay),
        secondary_char('c'),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.find_replace.toggle_whole_word"),
        ShortcutScope::Focus(FocusTarget::FindReplaceOverlay),
        secondary_char('w'),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.find_replace.toggle_regex"),
        ShortcutScope::Focus(FocusTarget::FindReplaceOverlay),
        secondary_char('r'),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.find_prev"),
        ShortcutScope::Focus(FocusTarget::FindReplaceOverlay),
        shift(KeyCode::Enter),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.find_next"),
        ShortcutScope::Focus(FocusTarget::FindReplaceOverlay),
        plain(KeyCode::Enter),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.replace_next"),
        ShortcutScope::Focus(FocusTarget::FindReplaceOverlay),
        secondary(KeyCode::Enter),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("editor.replace_all"),
        ShortcutScope::Focus(FocusTarget::FindReplaceOverlay),
        primary_secondary(KeyCode::Enter),
        100,
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
        CommandKindId("workspace.focus_panel.shortcut"),
        ShortcutScope::Global,
        primary_shift_char('k'),
        100,
    ),
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_overlay.settings"),
        ShortcutScope::Global,
        primary_char(','),
        80,
    ),
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
    DefaultShortcutSpec::new(
        CommandKindId("workspace.focus_panel.editor"),
        ShortcutScope::Focus(FocusTarget::TerminalPanel),
        plain(KeyCode::Enter),
        110,
    ),
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

/// 构造“无修饰键”的按键。
///
/// 适用于方向键、回车等纯按键输入，不附带任何逻辑修饰语义。
const fn plain(key: KeyCode) -> Keystroke {
    Keystroke::new(key, Modifiers::new(false, false, false, false))
}

/// 构造“仅 Shift 修饰”的按键。
///
/// `Shift` 在各平台语义一致，因此可直接使用物理位。
const fn shift(key: KeyCode) -> Keystroke {
    Keystroke::new(key, Modifiers::new(false, false, true, false))
}

/// 构造 “Primary + 字符” 的快捷键。
///
/// `Primary` 会在 `with_logical_modifiers` 中映射为：
/// macOS -> `Cmd(Command)`；其它平台 -> `Ctrl`。
const fn primary_char(c: char) -> Keystroke {
    Keystroke::new(
        KeyCode::Char(c),
        with_logical_modifiers(false, true, false, false),
    )
}

/// 构造 “Primary + Shift + 字符” 的快捷键。
///
/// 常用于“主命令的扩展动作”（例如 redo / 打开同类面板）。
const fn primary_shift_char(c: char) -> Keystroke {
    Keystroke::new(
        KeyCode::Char(c),
        with_logical_modifiers(true, true, false, false),
    )
}

/// 构造 “Primary + Secondary + 主键” 的组合快捷键。
///
/// 适用于需要双逻辑修饰键的高级动作。
const fn primary_secondary(key: KeyCode) -> Keystroke {
    Keystroke::new(key, with_logical_modifiers(false, true, true, false))
}

/// 构造 “Secondary + 主键” 的快捷键。
///
/// `Secondary` 会在 `with_logical_modifiers` 中映射为：
/// macOS -> `Ctrl`；其它平台 -> `Alt`。
const fn secondary(key: KeyCode) -> Keystroke {
    Keystroke::new(key, with_logical_modifiers(false, false, true, false))
}

/// 构造 “Secondary + 字符” 的快捷键。
///
/// 与 `secondary` 语义一致，仅主键固定为字符键。
const fn secondary_char(c: char) -> Keystroke {
    Keystroke::new(
        KeyCode::Char(c),
        with_logical_modifiers(false, false, true, false),
    )
}

/// 按逻辑修饰键组合构造平台相关 `Modifiers`。
///
/// 该函数是默认快捷键声明层的核心入口：调用方只描述“逻辑意图”，
/// 由这里统一展开成当前平台的实际修饰键位。
const fn with_logical_modifiers(
    is_shift_pressed: bool,
    has_primary_modifier: bool,
    has_secondary_modifier: bool,
    has_word_nav_modifier: bool,
) -> Modifiers {
    // 逻辑修饰键会在此展开为平台实际组合（macOS 与非 macOS 不同）。
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

/// 合并两组修饰键位（按位 OR）。
///
/// 用于把多个逻辑修饰键结果叠加为一份最终 `Modifiers`。
const fn merge_modifiers(base: Modifiers, extra: Modifiers) -> Modifiers {
    Modifiers::new(
        base.has_ctrl || extra.has_ctrl,
        base.has_alt || extra.has_alt,
        base.has_shift || extra.has_shift,
        base.has_cmd || extra.has_cmd,
    )
}

const fn primary_modifier() -> Modifiers {
    // “主命令键”：macOS 为 Command，其它平台为 Ctrl。
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
    // “次修饰键”：macOS 为 Ctrl，其它平台为 Alt。
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
    // 单词级导航修饰键：macOS 使用 Alt，其它平台使用 Ctrl。
    #[cfg(target_os = "macos")]
    {
        Modifiers::new(false, true, false, false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Modifiers::new(true, false, false, false)
    }
}
