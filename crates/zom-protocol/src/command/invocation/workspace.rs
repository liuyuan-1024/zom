//! 工作台领域命令调用载荷定义。

use crate::{FocusTarget, OverlayTarget};

/// 标签页动作语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabAction {
    /// 关闭当前激活标签页。
    CloseActiveTab,
    /// 激活前一个标签页。
    ActivatePrevTab,
    /// 激活下一个标签页。
    ActivateNextTab,
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

/// 通知中心动作语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationAction {
    /// 标记当前选中通知为已读。
    MarkSelectedRead,
    /// 标记全部通知为已读。
    MarkAllRead,
    /// 清空全部通知。
    ClearAll,
    /// 清空已读通知。
    ClearRead,
    /// 聚焦并定位到未读错误。
    FocusUnreadError,
    /// 选择上一条通知（面板内向上）。
    SelectPrev,
    /// 选择下一条通知（面板内向下）。
    SelectNext,
}

/// 工作台动作语义。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorkspaceAction {
    /// 退出应用。
    QuitApp,
    /// 最小化当前窗口。
    MinimizeWindow,

    /// 打开项目目录选择器。
    OpenProjectPicker,
    /// 保存当前活动标签页。
    SaveActiveBuffer,

    /// 聚焦到并显示指定面板。
    FocusPanel(FocusTarget),
    /// 聚焦到并显示指定悬浮层。
    FocusOverlay(OverlayTarget),
    /// 关闭当前聚焦组件（如悬浮层、面板、标签页等）。
    ///
    /// 具体“谁来处理关闭”由当前焦点目标决定。
    CloseFocused,

    /// 作用于标签页的动作。
    Tab(TabAction),
    /// 作用于文件树的动作。
    FileTree(FileTreeAction),
    /// 作用于通知中心的动作。
    Notification(NotificationAction),
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

impl From<NotificationAction> for WorkspaceAction {
    fn from(action: NotificationAction) -> Self {
        Self::Notification(action)
    }
}
