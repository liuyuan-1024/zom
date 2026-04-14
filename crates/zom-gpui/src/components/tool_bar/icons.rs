//! 底部状态栏专属的图标定义与渲染。

use gpui::{Hsla, div, prelude::*, px, svg};
use zom_app::StatusBarIcon;

/// 状态栏内部使用的图标类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StatusIcon {
    /// 文件树入口。
    Files,
    /// Git 入口。
    GitBranch,
    /// 结构 入口。
    Outline,
    /// 搜索入口。
    Search,
    /// LSP 入口。
    LanguageServer,
    /// 终端入口。
    Terminal,
    /// 调试入口。
    Debug,
    /// 通知入口。
    Notifications,
}

impl StatusIcon {
    /// 返回图标对应的 SVG 资源路径。
    fn path(self) -> &'static str {
        match self {
            Self::Files => "icons/tool_bar/tool_file_tree.svg",
            Self::GitBranch => "icons/tool_bar/tool_git_branch_alt.svg",
            Self::Outline => "icons/tool_bar/tool_list_tree.svg",
            Self::Search => "icons/tool_bar/tool_search.svg",
            Self::LanguageServer => "icons/tool_bar/tool_bolt_outlined.svg",
            Self::Terminal => "icons/tool_bar/tool_terminal.svg",
            Self::Debug => "icons/tool_bar/tool_debug.svg",
            Self::Notifications => "icons/tool_bar/tool_notification.svg",
        }
    }
}

impl From<StatusBarIcon> for StatusIcon {
    fn from(value: StatusBarIcon) -> Self {
        match value {
            StatusBarIcon::Files => Self::Files,
            StatusBarIcon::GitBranch => Self::GitBranch,
            StatusBarIcon::Outline => Self::Outline,
            StatusBarIcon::Search => Self::Search,
            StatusBarIcon::LanguageServer => Self::LanguageServer,
            StatusBarIcon::Terminal => Self::Terminal,
            StatusBarIcon::Debug => Self::Debug,
            StatusBarIcon::Notifications => Self::Notifications,
        }
    }
}

/// 渲染状态栏中的单色 SVG 图标。
pub(super) fn render(icon: StatusIcon, size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    let color = color.into();

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(icon.path()).size(px(size)).text_color(color))
}
