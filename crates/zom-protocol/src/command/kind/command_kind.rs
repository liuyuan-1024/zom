//! 命令语义族定义。

use crate::{FocusTarget, OverlayTarget};

/// 命令的稳定语义族（无 UI / 输入细节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKind {
    // --- Editor: 插入与缩进 ---
    /// 编辑器插入动态文本。
    EditorInsertText,
    /// 编辑器插入换行。
    EditorInsertNewline,
    /// 编辑器插入缩进。
    EditorInsertIndent,
    /// 编辑器反缩进。
    EditorOutdent,

    // --- Editor: 光标移动 ---
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

    // --- Editor: 选区 ---
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

    // --- Editor: 删除 ---
    /// 编辑器向后删除一个字符。
    EditorDeleteBackward,
    /// 编辑器向前删除一个字符。
    EditorDeleteForward,
    /// 编辑器向后删除一个单词。
    EditorDeleteWordBackward,
    /// 编辑器向前删除一个单词。
    EditorDeleteWordForward,

    // --- Editor: 剪贴板与历史 ---
    /// 编辑器复制选区。
    EditorCopy,
    /// 编辑器剪切选区。
    EditorCut,
    /// 编辑器粘贴内容。
    EditorPaste,
    /// 编辑器撤销。
    EditorUndo,
    /// 编辑器重做。
    EditorRedo,

    // --- Editor: 查找替换 ---
    /// 打开当前活动编辑器的查找替换条。
    EditorOpenFindReplace,
    /// 查找条切换大小写匹配。
    EditorToggleFindCaseSensitive,
    /// 查找条切换整词匹配。
    EditorToggleFindWholeWord,
    /// 查找条切换正则模式。
    EditorToggleFindRegex,
    /// 编辑器查找下一个匹配项。
    EditorFindNext,
    /// 编辑器查找上一个匹配项。
    EditorFindPrev,
    /// 编辑器替换下一个匹配项。
    EditorReplaceNext,
    /// 编辑器替换全部匹配项。
    EditorReplaceAll,

    // --- Workspace: 顶层动作 ---
    /// 退出应用。
    WorkspaceQuitApp,
    /// 最小化当前窗口。
    WorkspaceMinimizeWindow,

    /// 打开项目选择器。
    WorkspaceOpenProjectPicker,
    /// 保存当前活动标签页。
    WorkspaceSaveActiveBuffer,
    /// 关闭当前聚焦组件。
    WorkspaceCloseFocused,

    // --- Workspace: 聚焦 ---
    /// 显示并聚焦指定面板。
    WorkspaceFocusPanel(FocusTarget),
    /// 显示并聚焦指定悬浮层。
    WorkspaceFocusOverlay(OverlayTarget),

    // --- Workspace.FileTree ---
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

    // --- Workspace.Tab ---
    /// 关闭当前活动标签页。
    WorkspaceTabCloseActive,
    /// 激活上一个标签页。
    WorkspaceTabActivatePrev,
    /// 激活下一个标签页。
    WorkspaceTabActivateNext,
}
