//! 命令规范目录使用的核心类型定义。

use std::fmt;

use crate::{CommandInvocation, FocusTarget, KeyCode, Keystroke, Modifiers, OverlayTarget};

/// 命令的稳定语义族（无 UI / 输入细节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKind {
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
    WorkspaceFocusOverlay(OverlayTarget),
    WorkspaceCloseFocused,

    WorkspaceOpenProjectPicker,

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

/// 命令语义族的稳定字符串 ID，供跨层引用与文档检索。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandKindId(pub &'static str);

impl fmt::Display for CommandKindId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// 命令元信息（纯描述，不含行为）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandMeta {
    /// 稳定 ID，供跨层引用与文档检索。
    pub id: CommandKindId,
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

/// 由语义族声明反向构造运行时调用的函数签名。
pub type InvocationBuilder = fn() -> CommandInvocation;

/// 语义族是否可从 Kind 直接构造运行时调用。
#[derive(Clone, Copy)]
pub enum Buildability {
    /// 可直接构造（无动态参数）。
    Static(InvocationBuilder),
    /// 需要动态参数，不能直接构造。
    RequiresArgs,
}

impl fmt::Debug for Buildability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(_) => f.write_str("Static(..)"),
            Self::RequiresArgs => f.write_str("RequiresArgs"),
        }
    }
}

/// 命令语义族统一声明结构。
#[derive(Debug, Clone, Copy)]
pub struct CommandKindSpec {
    /// 稳定语义族。
    pub kind: CommandKind,
    /// 只读元信息（UI 文案 / 文档引用）。
    pub meta: CommandMeta,
    /// 该语义族的构造能力。
    pub buildability: Buildability,
    /// 默认快捷键集合（可为空）。
    pub default_shortcuts: &'static [CommandShortcut],
}

impl CommandKindSpec {
    /// 创建一条命令语义族声明。
    pub const fn new(
        kind: CommandKind,
        id: &'static str,
        title: &'static str,
        description: &'static str,
        buildability: Buildability,
        default_shortcuts: &'static [CommandShortcut],
    ) -> Self {
        Self {
            kind,
            meta: CommandMeta {
                id: CommandKindId(id),
                title,
                description,
            },
            buildability,
            default_shortcuts,
        }
    }
}

/// 一条“命令调用 + 默认快捷键”绑定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultShortcutBinding {
    /// 命令调用。
    pub command: CommandInvocation,
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
