//! 工作台领域命令调用载荷定义。

use crate::{FocusTarget, OverlayTarget};

/// 标签页动作语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabAction {
    // --- 激活 ---
    /// 激活前一个标签页。
    ActivatePrevTab,
    /// 激活下一个标签页。
    ActivateNextTab,
    // --- 关闭 ---
    /// 关闭当前激活标签页。
    CloseActiveTab,
}

/// 文件树动作语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileTreeAction {
    /// 选中上一条可见节点。
    SelectPrev,
    /// 选中下一条可见节点。
    SelectNext,
    /// 目录展开，或进入已展开目录的第一个子节点。
    ExpandOrDescend,
    /// 目录折叠，或回到父节点。
    CollapseOrAscend,
    /// 激活当前选中节点（文件打开，目录切换展开态）。
    ActivateSelection,
}

/// 工作台动作语义。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkspaceAction {
    // --- 窗口与项目 ---
    /// 退出应用。
    QuitApp,
    /// 最小化当前窗口。
    MinimizeWindow,
    /// 打开项目目录选择器。
    OpenProjectPicker,

    // --- 文档与关闭 ---
    /// 保存当前活动标签页。
    SaveActiveBuffer,
    /// 关闭当前聚焦组件（如悬浮层、面板、标签页等）。
    ///
    /// 具体“谁来处理关闭”由当前焦点目标决定。
    CloseFocused,

    // --- 焦点切换 ---
    /// 聚焦到并显示指定面板。
    FocusPanel(FocusTarget),
    /// 聚焦到并显示指定悬浮层。
    FocusOverlay(OverlayTarget),

    // --- 面板内域动作 ---
    /// 作用于文件树的动作。
    FileTree(FileTreeAction),
    /// 作用于标签页的动作。
    Tab(TabAction),
}

impl From<FileTreeAction> for WorkspaceAction {
    /// 文件树动作提升到工作台动作，复用统一分发通道。
    fn from(action: FileTreeAction) -> Self {
        Self::FileTree(action)
    }
}

impl From<TabAction> for WorkspaceAction {
    fn from(action: TabAction) -> Self {
        Self::Tab(action)
    }
}
