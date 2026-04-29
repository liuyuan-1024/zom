//! 底部状态栏视图渲染。

use gpui::prelude::*;
use zom_input::shortcut_hint;
use zom_protocol::{CommandInvocation, FocusTarget, WorkspaceAction};
use zom_runtime::{
    projection::{command_is_active, cursor_text},
    state::{DesktopAppState, DesktopNotificationLevel},
};

use super::bar_shell::BarShell;
use crate::components::chip::Chip;
use crate::icon::AppIcon;

#[derive(Debug, Clone, Copy)]
struct PanelChipSpec {
    id: &'static str,
    target: FocusTarget,
    icon: AppIcon,
    label: &'static str,
}

const LEFT_PANEL_SPECS: [PanelChipSpec; 5] = [
    PanelChipSpec {
        id: "status-panel-file-tree",
        target: FocusTarget::FileTreePanel,
        icon: AppIcon::FileTree,
        label: "文件树",
    },
    PanelChipSpec {
        id: "status-panel-git",
        target: FocusTarget::GitPanel,
        icon: AppIcon::GitBranchAlt,
        label: "Git",
    },
    PanelChipSpec {
        id: "status-panel-outline",
        target: FocusTarget::OutlinePanel,
        icon: AppIcon::ListTree,
        label: "大纲",
    },
    PanelChipSpec {
        id: "status-panel-project-search",
        target: FocusTarget::ProjectSearchPanel,
        icon: AppIcon::Search,
        label: "项目搜索",
    },
    PanelChipSpec {
        id: "status-panel-language-servers",
        target: FocusTarget::LanguageServersPanel,
        icon: AppIcon::BoltOutlined,
        label: "语言服务",
    },
];

const RIGHT_PANEL_SPECS: [PanelChipSpec; 3] = [
    PanelChipSpec {
        id: "status-panel-terminal",
        target: FocusTarget::TerminalPanel,
        icon: AppIcon::Terminal,
        label: "终端",
    },
    PanelChipSpec {
        id: "status-panel-debug",
        target: FocusTarget::DebugPanel,
        icon: AppIcon::Debug,
        label: "Debug",
    },
    PanelChipSpec {
        id: "status-panel-notification",
        target: FocusTarget::NotificationPanel,
        icon: AppIcon::Notification,
        label: "通知中心",
    },
];

/// 渲染底部状态栏。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    let mut shell = BarShell::new(false);

    // 装配左侧区域
    for spec in LEFT_PANEL_SPECS {
        shell = shell.left(render_panel_chip(state, spec));
    }
    shell = shell.left(render_diagnostics_chip(state));

    // 装配右侧区域
    let cursor = cursor_text(state.tool_bar.cursor);
    let language = if state.tool_bar.language.is_empty() {
        "".to_string()
    } else {
        state.tool_bar.language.clone()
    };

    // 光标和语言：仅在存在活动标签页时显示
    if state.pane.active_tab().is_some() {
        shell = shell.right(render_value_chip("status-cursor", cursor, "光标位置"));
        shell = shell.right(render_value_chip("status-language", language, "当前语言"));
    }

    // 终端、Debug、通知中心
    for spec in RIGHT_PANEL_SPECS {
        shell = shell.right(render_panel_chip(state, spec));
    }

    shell
}

/// 渲染面板并组装对应界面节点。
fn render_panel_chip(state: &DesktopAppState, spec: PanelChipSpec) -> impl IntoElement {
    let command = panel_command(spec.target);
    let is_active = command_is_active(state, &command);

    Chip::new(spec.id)
        .icon(spec.icon)
        .tooltip_hint(spec.label, shortcut_hint(&command))
        // 激活时仅改变图标前景色，不切换样式，避免视觉跳动。
        .active(is_active)
}

/// 统计 Warning/Error 条目并渲染诊断芯片；无诊断时显示通过态图标。
fn render_diagnostics_chip(state: &DesktopAppState) -> impl IntoElement {
    let count = diagnostic_count(state);
    if count == 0 {
        Chip::new("status-bar-diagnostics")
            .icon(AppIcon::Check)
            .active(false)
            .tooltip_hint("语言诊断: 0", Option::<String>::None)
    } else {
        Chip::new("status-bar-diagnostics")
            .active(false)
            .label(count.to_string())
            .tooltip_hint(format!("语言诊断: {count}"), Option::<String>::None)
    }
}

fn render_value_chip(
    id: &'static str,
    value: impl Into<String>,
    tooltip: &str,
) -> impl IntoElement {
    Chip::new(id)
        .label(value.into())
        .tooltip_hint(tooltip, Option::<String>::None)
}

/// 把面板目标映射为对应的聚焦命令。
fn panel_command(target: FocusTarget) -> CommandInvocation {
    CommandInvocation::from(WorkspaceAction::FocusPanel(target))
}

/// 统计当前通知列表中的告警与错误数量，供状态栏诊断角标复用。
fn diagnostic_count(state: &DesktopAppState) -> usize {
    state
        .notifications
        .iter()
        .filter(|notification| {
            matches!(
                notification.level,
                DesktopNotificationLevel::Warning | DesktopNotificationLevel::Error
            )
        })
        .count()
}
