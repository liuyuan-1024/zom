//! 标题栏图标规格与渲染辅助。

use gpui::{Hsla, div, prelude::*, px, svg};
use zom_protocol::{CommandInvocation, OverlayTarget, WorkspaceAction};
use zom_runtime::projection::shortcut_hint;
use zom_runtime::state::TitleBarAction;

/// 标题栏图标的展示规格。
pub(super) struct TitleBarIconSpec {
    /// SVG 资源路径。
    pub path: &'static str,
    /// 悬停时显示的名称。
    pub label: &'static str,
    /// 悬停时显示的快捷键。
    pub shortcut: Option<String>,
}

/// 将应用层语义映射为标题栏自身维护的展示规格。
pub(super) fn spec(action: &TitleBarAction) -> TitleBarIconSpec {
    let (path, label) = match &action.command {
        CommandInvocation::Workspace(WorkspaceAction::FocusOverlay(OverlayTarget::Settings)) => {
            ("icons/title_bar/title_settings.svg", "Settings")
        }
        _ => ("icons/keyboard.svg", "Action"),
    };

    TitleBarIconSpec {
        path,
        label,
        shortcut: shortcut_hint(&action.command),
    }
}

/// 渲染标题栏中的单色 SVG 图标。
pub(super) fn render(
    action: &TitleBarAction,
    size: f32,
    color: impl Into<Hsla>,
) -> impl IntoElement {
    let color = color.into();
    let spec = spec(action);

    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .child(svg().path(spec.path).size(px(size)).text_color(color))
}
