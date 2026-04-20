//! 标题栏状态与图标语义定义。

/// 标题栏使用的图标语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleBarIcon {
    /// 系统设置。
    Settings,
}

/// 标题栏展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleBarState {
    /// 标题栏右侧的工具入口。
    pub right_icons: Vec<TitleBarIcon>,
}
