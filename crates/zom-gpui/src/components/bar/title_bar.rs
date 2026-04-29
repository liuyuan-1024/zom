//! 标题栏视图渲染。

use gpui::{div, prelude::*, px};
use zom_input::shortcut_hint;
use zom_protocol::{CommandInvocation, OverlayTarget, WorkspaceAction};
use zom_runtime::state::{DesktopAppState, TitleBarAction};

use super::bar_shell::BarShell;
use super::traffic_lights;
use crate::components::chip::Chip;
use crate::icon::AppIcon;

struct TitleBarActionIconSpec {
    /// 图标资源语义。
    icon: AppIcon,
    /// tooltip 主文案。
    label: &'static str,
    /// 可选快捷键提示文本。
    shortcut: Option<String>,
}

/// 渲染顶栏
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    let mut shell = BarShell::new(true);

    // 左侧：红绿灯避让区 + 项目名称指示器
    let project_picker_cmd = CommandInvocation::from(WorkspaceAction::OpenProjectPicker);
    shell = shell.left(div().w(px(traffic_lights::slot_width())));

    shell = shell.left(
        Chip::new("title-bar-project_name")
            .label(state.project_name.clone())
            .tooltip_hint("选择项目", shortcut_hint(&project_picker_cmd)),
    );

    // 右侧：全局操作与设置
    for (index, action) in state.title_bar.right_actions.iter().enumerate() {
        shell = shell.right(render_action_button(index, action));
    }

    shell
}

/// 渲染标题栏右侧系统设置按钮。
fn render_action_button(index: usize, action: &TitleBarAction) -> impl IntoElement {
    let spec = title_bar_action_icon_spec(action);

    Chip::new(("title-bar-item", index))
        .icon(spec.icon)
        .tooltip_hint(spec.label, spec.shortcut)
}

/// 将标题栏动作映射为图标与提示文案规格。
fn title_bar_action_icon_spec(action: &TitleBarAction) -> TitleBarActionIconSpec {
    let (icon, label) = match &action.command {
        CommandInvocation::Workspace(WorkspaceAction::FocusOverlay(OverlayTarget::Settings)) => {
            (AppIcon::Settings, "设置")
        }
        _ => (AppIcon::Keyboard, "操作"),
    };

    TitleBarActionIconSpec {
        icon,
        label,
        shortcut: shortcut_hint(&action.command),
    }
}
