//! 编辑器领域命令规范声明。

use crate::command::kind::{
    Buildability, CommandKind, CommandKindSpec, CommandShortcut, ShortcutScope,
    types::{meta_char, meta_shift_char, plain, shift},
};
use crate::{CommandInvocation, EditorAction, FocusTarget, KeyCode};

pub const SPECS: &[CommandKindSpec] = &[
    CommandKindSpec::new(
        CommandKind::EditorInsertText,
        "editor.insert_text",
        "插入文本",
        "将提供的文本插入到当前光标位置。",
        Buildability::RequiresArgs,
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::EditorInsertNewline,
        "editor.insert_newline",
        "插入新行",
        "在当前光标位置插入一个新行。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::InsertNewline)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::Enter),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorMoveLeft,
        "editor.move_left",
        "光标左移",
        "将光标向左移动一个字符。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::MoveLeft)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::Left),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorMoveRight,
        "editor.move_right",
        "光标右移",
        "将光标向右移动一个字符。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::MoveRight)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::Right),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorMoveUp,
        "editor.move_up",
        "光标上移",
        "将光标向上移动一行。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::MoveUp)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::Up),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorMoveDown,
        "editor.move_down",
        "光标下移",
        "将光标向下移动一行。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::MoveDown)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::Down),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorMoveToStart,
        "editor.move_to_start",
        "光标移动到行起点",
        "将光标移动到当前行的起点。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::MoveToStart)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::Home),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorMoveToEnd,
        "editor.move_to_end",
        "光标移动到行终点",
        "将光标移动到当前行的终点。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::MoveToEnd)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::End),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorMovePageUp,
        "editor.page_up",
        "向上翻页",
        "将光标向上翻一页。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::MovePageUp)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::PageUp),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorMovePageDown,
        "editor.page_down",
        "向下翻页",
        "将光标向下翻一页。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::MovePageDown)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::PageDown),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorSelectLeft,
        "editor.select_left",
        "向左扩展选区",
        "将选区活动端向左移动一个字符。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::SelectLeft)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            shift(KeyCode::Left),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorSelectRight,
        "editor.select_right",
        "向右扩展选区",
        "将选区活动端向右移动一个字符。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::SelectRight)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            shift(KeyCode::Right),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorSelectUp,
        "editor.select_up",
        "向上扩展选区",
        "将选区活动端向上移动一行。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::SelectUp)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            shift(KeyCode::Up),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorSelectDown,
        "editor.select_down",
        "向下扩展选区",
        "将选区活动端向下移动一行。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::SelectDown)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            shift(KeyCode::Down),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorSelectToStart,
        "editor.select_to_start",
        "向行起点扩展选区",
        "将选区活动端移动到当前行起点。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::SelectToStart)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            shift(KeyCode::Home),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorSelectToEnd,
        "editor.select_to_end",
        "向行终点扩展选区",
        "将选区活动端移动到当前行终点。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::SelectToEnd)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            shift(KeyCode::End),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorSelectPageUp,
        "editor.select_page_up",
        "向上扩展一页选区",
        "将选区活动端向上移动一页。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::SelectPageUp)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            shift(KeyCode::PageUp),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorSelectPageDown,
        "editor.select_page_down",
        "向下扩展一页选区",
        "将选区活动端向下移动一页。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::SelectPageDown)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            shift(KeyCode::PageDown),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorDeleteBackward,
        "editor.delete_backward",
        "删除前一个字符",
        "删除光标前一个字符。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::DeleteBackward)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::Backspace),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorDeleteForward,
        "editor.delete_forward",
        "删除后一个字符",
        "删除光标后一个字符。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::DeleteForward)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            plain(KeyCode::Delete),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorDeleteWordBackward,
        "editor.delete_word_backward",
        "删除前一个单词",
        "删除光标前一个单词。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::DeleteWordBackward)),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::EditorDeleteWordForward,
        "editor.delete_word_forward",
        "删除后一个单词",
        "删除光标后一个单词。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::DeleteWordForward)),
        &[],
    ),
    CommandKindSpec::new(
        CommandKind::EditorUndo,
        "editor.undo",
        "撤销",
        "撤销最近一次编辑。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::Undo)),
        &[
            CommandShortcut::new(ShortcutScope::Focus(FocusTarget::Editor), meta_char('z'))
                .with_priority(120),
        ],
    ),
    CommandKindSpec::new(
        CommandKind::EditorRedo,
        "editor.redo",
        "重做",
        "重做最近一次撤销的编辑。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::Redo)),
        &[CommandShortcut::new(
            ShortcutScope::Focus(FocusTarget::Editor),
            meta_shift_char('z'),
        )
        .with_priority(120)],
    ),
    CommandKindSpec::new(
        CommandKind::EditorSelectAll,
        "editor.select_all",
        "全选",
        "全选当前编辑器中的内容。",
        Buildability::Static(|| CommandInvocation::from(EditorAction::SelectAll)),
        &[
            CommandShortcut::new(ShortcutScope::Focus(FocusTarget::Editor), meta_char('a'))
                .with_priority(120),
        ],
    ),
];
