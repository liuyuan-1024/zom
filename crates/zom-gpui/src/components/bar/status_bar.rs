//! 底部状态栏视图渲染。

use gpui::{div, prelude::*, px, rgb};
use zom_input::shortcut_hint;
use zom_protocol::{CommandInvocation, FocusTarget, WorkspaceAction};
use zom_runtime::{
    projection::{command_is_active, cursor_text},
    state::DesktopAppState,
};

use super::bar_shell::BarShell;
use crate::components::chip::Chip;
use crate::icon::AppIcon;
use crate::theme::{color, size};

#[derive(Debug, Clone, Copy)]
struct PanelChipSpec {
    id: &'static str,
    target: FocusTarget,
    icon: AppIcon,
    label: &'static str,
}

const LEFT_PANEL_MAIN_GROUP: [PanelChipSpec; 2] = [
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
];

const LEFT_PANEL_AUX_GROUP: [PanelChipSpec; 3] = [
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

const LEFT_PANEL_GROUPS: [&[PanelChipSpec]; 2] = [&LEFT_PANEL_MAIN_GROUP, &LEFT_PANEL_AUX_GROUP];

const RIGHT_PANEL_MAIN_GROUP: [PanelChipSpec; 2] = [
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
        label: "调试",
    },
];

const RIGHT_PANEL_AUX_GROUP: [PanelChipSpec; 1] = [PanelChipSpec {
    id: "status-panel-shortcut",
    target: FocusTarget::ShortcutPanel,
    icon: AppIcon::Keyboard,
    label: "快捷键",
}];

const RIGHT_PANEL_GROUPS: [&[PanelChipSpec]; 2] = [&RIGHT_PANEL_MAIN_GROUP, &RIGHT_PANEL_AUX_GROUP];

/// 渲染底部状态栏。
pub(crate) fn render(state: &DesktopAppState) -> impl IntoElement {
    let mut shell = BarShell::new(false);

    // 装配左侧分组区域
    for (group_index, group) in LEFT_PANEL_GROUPS.iter().enumerate() {
        if group_index > 0 {
            shell = shell.left(render_group_divider());
        }

        for spec in group.iter().copied() {
            shell = shell.left(render_panel_chip(state, spec));
        }
    }

    // 光标位置
    let cursor = cursor_text(state.tool_bar.cursor);
    // 语言
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

    // 右侧分组区域（如：终端/调试 与 快捷键）
    for (group_index, group) in RIGHT_PANEL_GROUPS.iter().enumerate() {
        if group_index > 0 {
            shell = shell.right(render_group_divider());
        }

        for spec in group.iter().copied() {
            shell = shell.right(render_panel_chip(state, spec));
        }
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

fn render_value_chip(
    id: &'static str,
    value: impl Into<String>,
    tooltip: &str,
) -> impl IntoElement {
    Chip::new(id)
        .label(value.into())
        .tooltip_hint(tooltip, Option::<String>::None)
}

/// 渲染同侧不同组之间的分隔线。
fn render_group_divider() -> impl IntoElement {
    div()
        .w(px(1.0))
        .h(px(size::ICON_MD))
        .bg(rgb(color::COLOR_BORDER))
}

/// 把面板目标映射为对应的聚焦命令。
fn panel_command(target: FocusTarget) -> CommandInvocation {
    CommandInvocation::from(WorkspaceAction::FocusPanel(target))
}
