//! 标题栏状态与图标语义定义。

use zom_protocol::CommandInvocation;

/// 标题栏动作条目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleBarAction {
    /// 该入口对应的命令语义。
    pub command: CommandInvocation,
}

/// 标题栏展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleBarState {
    /// 标题栏右侧的工具入口（当前设计仅右侧，便于后续渐进扩展左侧区域）。
    pub right_actions: Vec<TitleBarAction>,
}
