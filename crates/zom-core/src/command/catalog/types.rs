use std::fmt;

use crate::{FocusTarget, KeyCode, Keystroke, Modifiers};

/// 命令的稳定语义键（无 UI / 输入细节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKey {
    EditorInsertText,
    EditorInsertNewline,
    EditorMoveLeft,
    EditorMoveRight,
    EditorMoveUp,
    EditorMoveDown,
    EditorMoveToStart,
    EditorMoveToEnd,
    EditorMovePageUp,
    EditorMovePageDown,
    EditorDeleteBackward,
    EditorDeleteForward,
    EditorDeleteWordBackward,
    EditorDeleteWordForward,
    EditorUndo,
    EditorRedo,
    EditorSelectAll,

    WorkspaceFocusPanel(FocusTarget),
    WorkspaceCloseFocused,
    WorkspaceOpenProjectPicker,
    WorkspaceOpenSettings,
    WorkspaceOpenCodeActions,
    WorkspaceStartDebugging,
    WorkspaceFileTreeSelectPrev,
    WorkspaceFileTreeSelectNext,
    WorkspaceFileTreeExpandOrDescend,
    WorkspaceFileTreeCollapseOrAscend,
    WorkspaceFileTreeActivateSelection,
    WorkspaceTabCloseActive,
    WorkspaceTabActivatePrev,
    WorkspaceTabActivateNext,
}

/// 命令的稳定字符串 ID，供跨层引用与文档检索。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandId(pub &'static str);

impl fmt::Display for CommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// 命令的基础元信息（纯描述，不含行为）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandMeta {
    /// 稳定 ID，供跨层引用与文档检索。
    pub id: CommandId,
    /// 简短标题。
    pub title: &'static str,
    /// 语义说明。
    pub description: &'static str,
}

/// 默认快捷键作用域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutScope {
    /// 全局快捷键。
    Global,
    /// 仅在指定焦点下生效。
    Focus(FocusTarget),
}

/// 命令目录里定义的一条默认快捷键。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandShortcut {
    /// 作用域。
    pub scope: ShortcutScope,
    /// 按键定义。
    pub keystroke: Keystroke,
    /// 优先级（越大越优先）。
    pub priority: u8,
}

impl CommandShortcut {
    /// 创建一条默认快捷键。
    pub const fn new(scope: ShortcutScope, keystroke: Keystroke) -> Self {
        Self {
            scope,
            keystroke,
            priority: 0,
        }
    }

    /// 设置优先级。
    pub const fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// 从命令声明反向构造可执行命令的函数签名。
pub type CommandFactory = fn() -> Option<super::Command>;

/// 命令统一声明结构。
#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    /// 稳定语义键。
    pub key: CommandKey,
    /// 只读元信息（UI 文案 / 文档引用）。
    pub meta: CommandMeta,
    /// 由声明投影出来的“Key -> Command”构造函数。
    pub factory: CommandFactory,
    /// 默认快捷键集合（可为空）。
    pub default_shortcuts: &'static [CommandShortcut],
}

impl CommandSpec {
    /// 创建一条命令声明。
    pub const fn new(
        key: CommandKey,
        id: &'static str,
        title: &'static str,
        description: &'static str,
        factory: CommandFactory,
        default_shortcuts: &'static [CommandShortcut],
    ) -> Self {
        Self {
            key,
            meta: CommandMeta {
                id: CommandId(id),
                title,
                description,
            },
            factory,
            default_shortcuts,
        }
    }
}

/// 一条“命令 + 默认快捷键”绑定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultShortcutBinding {
    /// 命令语义。
    pub command: super::Command,
    /// 对应快捷键。
    pub shortcut: CommandShortcut,
}

pub(crate) const fn plain(key: KeyCode) -> Keystroke {
    Keystroke::new(key, Modifiers::new(false, false, false, false))
}

pub(crate) const fn meta_char(c: char) -> Keystroke {
    Keystroke::new(KeyCode::Char(c), Modifiers::new(false, false, false, true))
}

pub(crate) const fn meta_shift_char(c: char) -> Keystroke {
    Keystroke::new(KeyCode::Char(c), Modifiers::new(false, false, true, true))
}

pub(crate) const fn ctrl_char(c: char) -> Keystroke {
    Keystroke::new(KeyCode::Char(c), Modifiers::new(true, false, false, false))
}
