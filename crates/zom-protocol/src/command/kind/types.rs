//! 命令规范目录使用的核心类型定义。

use std::fmt;

use crate::{CommandInvocation, FocusTarget, KeyCode, Keystroke, Modifiers, OverlayTarget};

/// 命令的稳定语义族（无 UI / 输入细节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKind {
    /// 编辑器插入动态文本。
    EditorInsertText,
    /// 编辑器插入换行。
    EditorInsertNewline,
    /// 编辑器光标左移。
    EditorMoveLeft,
    /// 编辑器光标右移。
    EditorMoveRight,
    /// 编辑器光标上移。
    EditorMoveUp,
    /// 编辑器光标下移。
    EditorMoveDown,
    /// 编辑器光标移动到当前行起点。
    EditorMoveToStart,
    /// 编辑器光标移动到当前行终点。
    EditorMoveToEnd,
    /// 编辑器向上翻页。
    EditorMovePageUp,
    /// 编辑器向下翻页。
    EditorMovePageDown,
    /// 编辑器向左扩展选区。
    EditorSelectLeft,
    /// 编辑器向右扩展选区。
    EditorSelectRight,
    /// 编辑器向上扩展选区。
    EditorSelectUp,
    /// 编辑器向下扩展选区。
    EditorSelectDown,
    /// 编辑器向当前行起点扩展选区。
    EditorSelectToStart,
    /// 编辑器向当前行终点扩展选区。
    EditorSelectToEnd,
    /// 编辑器向上扩展一页选区。
    EditorSelectPageUp,
    /// 编辑器向下扩展一页选区。
    EditorSelectPageDown,
    /// 编辑器全选。
    EditorSelectAll,
    /// 编辑器向后删除一个字符。
    EditorDeleteBackward,
    /// 编辑器向前删除一个字符。
    EditorDeleteForward,
    /// 编辑器向后删除一个单词。
    EditorDeleteWordBackward,
    /// 编辑器向前删除一个单词。
    EditorDeleteWordForward,
    /// 编辑器撤销。
    EditorUndo,
    /// 编辑器重做。
    EditorRedo,

    /// 退出应用。
    WorkspaceQuitApp,
    /// 最小化当前窗口。
    WorkspaceMinimizeWindow,
    /// 打开项目选择器。
    WorkspaceOpenProjectPicker,

    /// 显示并聚焦指定面板。
    WorkspaceFocusPanel(FocusTarget),
    /// 显示并聚焦指定悬浮层。
    WorkspaceFocusOverlay(OverlayTarget),
    /// 关闭当前聚焦组件。
    WorkspaceCloseFocused,

    /// 打开代码操作入口。
    WorkspaceOpenCodeActions,
    /// 启动调试流程。
    WorkspaceStartDebugging,

    /// 文件树选择上一项。
    WorkspaceFileTreeSelectPrev,
    /// 文件树选择下一项。
    WorkspaceFileTreeSelectNext,
    /// 文件树展开目录或下探到子节点。
    WorkspaceFileTreeExpandOrDescend,
    /// 文件树折叠目录或上探到父节点。
    WorkspaceFileTreeCollapseOrAscend,
    /// 激活文件树当前选中项。
    WorkspaceFileTreeActivateSelection,

    /// 关闭当前活动标签页。
    WorkspaceTabCloseActive,
    /// 激活上一个标签页。
    WorkspaceTabActivatePrev,
    /// 激活下一个标签页。
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

pub(crate) const fn shift(key: KeyCode) -> Keystroke {
    Keystroke::new(key, Modifiers::new(false, false, true, false))
}

/// 主命令修饰键（Primary）：
/// macOS 为 Command，Windows/Linux 为 Ctrl。
pub(crate) const fn primary_char(c: char) -> Keystroke {
    Keystroke::new(
        KeyCode::Char(c),
        with_logical_modifiers(false, true, false, false),
    )
}

/// 主命令修饰键 + Shift：
/// macOS 为 Command+Shift，Windows/Linux 为 Ctrl+Shift。
pub(crate) const fn primary_shift_char(c: char) -> Keystroke {
    Keystroke::new(
        KeyCode::Char(c),
        with_logical_modifiers(true, true, false, false),
    )
}

/// 次命令修饰键（Secondary）：
/// macOS 为 Ctrl，Windows/Linux 为 Alt。
#[allow(dead_code)]
pub(crate) const fn secondary_char(c: char) -> Keystroke {
    Keystroke::new(
        KeyCode::Char(c),
        with_logical_modifiers(false, false, true, false),
    )
}

/// 单词导航修饰键（WordNav）：
/// macOS 为 Alt，Windows/Linux 为 Ctrl。
#[allow(dead_code)]
pub(crate) const fn word_nav_char(c: char) -> Keystroke {
    Keystroke::new(
        KeyCode::Char(c),
        with_logical_modifiers(false, false, false, true),
    )
}

const fn with_logical_modifiers(
    shift: bool,
    primary: bool,
    secondary: bool,
    word_nav: bool,
) -> Modifiers {
    let mut modifiers = Modifiers::new(false, false, shift, false);
    if primary {
        modifiers = merge_modifiers(modifiers, primary_modifier());
    }
    if secondary {
        modifiers = merge_modifiers(modifiers, secondary_modifier());
    }
    if word_nav {
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
