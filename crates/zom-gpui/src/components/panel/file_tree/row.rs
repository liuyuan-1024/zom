//! 文件树行渲染与视觉状态规则。

use gpui::{AnyElement, div, prelude::*, px, rgb, svg, transparent_black};
use zom_runtime::state::{FileTreeNode, FileTreeNodeKind};

use super::FILE_TREE_INDENT_STEP;
use crate::theme::{color, size};

/// 渲染单个文件树节点行。
pub(super) fn render(node: &FileTreeNode, depth: usize, is_panel_focused: bool) -> AnyElement {
    let has_focus_emphasis = focus_emphasis_visible(node, is_panel_focused);
    let mut row = div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        // 常态保留透明边框，避免焦点态出现时导致行高抖动。
        .border_1()
        .border_color(transparent_black())
        .pl(px(FILE_TREE_INDENT_STEP * depth as f32))
        .child(render_kind_badge(node))
        .child(render_label(node));

    if has_focus_emphasis {
        row = row.border_color(rgb(color::COLOR_FG_PRIMARY));
    }

    if let Some(background) = row_background_color(node) {
        row.bg(rgb(background)).into_any_element()
    } else {
        row.into_any_element()
    }
}

/// 焦点强调可见
fn focus_emphasis_visible(node: &FileTreeNode, is_panel_focused: bool) -> bool {
    node.is_selected && is_panel_focused
}

/// 行背景色
fn row_background_color(node: &FileTreeNode) -> Option<u32> {
    // 当前打开文件始终高亮；导航的单一选中不再使用背景色，由焦点外框表达。
    if node.is_active {
        Some(color::COLOR_BG_ACTIVE)
    } else {
        None
    }
}

/// 渲染节点名称。
fn render_label(node: &FileTreeNode) -> impl IntoElement {
    div()
        .flex_1()
        .text_sm()
        .text_color(rgb(color::COLOR_FG_PRIMARY))
        .overflow_hidden()
        .whitespace_nowrap()
        .child(node.name.clone())
}

/// 渲染目录图标或文件占位。
fn render_kind_badge(node: &FileTreeNode) -> impl IntoElement {
    match node.kind {
        FileTreeNodeKind::Directory => render_folder_icon(node).into_any_element(),
        FileTreeNodeKind::File => render_file_icon().into_any_element(),
    }
}

/// 渲染目录图标。
fn render_folder_icon(node: &FileTreeNode) -> impl IntoElement {
    let icon_path = if node.is_expanded {
        "icons/file_tree/folder_open.svg"
    } else {
        "icons/file_tree/folder.svg"
    };

    div()
        .w(px(size::GAP_3))
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path(icon_path)
                .size(px(size::ICON_MD))
                .text_color(rgb(color::COLOR_FG_MUTED)),
        )
}

/// 渲染文件图标。
fn render_file_icon() -> impl IntoElement {
    div()
        .w(px(size::GAP_3))
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path("icons/file_tree/file.svg")
                .size(px(size::ICON_MD))
                .text_color(rgb(color::COLOR_FG_MUTED)),
        )
}

#[cfg(test)]
mod tests {
    use zom_runtime::state::{FileTreeNode, FileTreeNodeKind};

    use super::{focus_emphasis_visible, row_background_color};
    use crate::theme::color;

    fn node(is_selected: bool, is_active: bool) -> FileTreeNode {
        FileTreeNode {
            name: "lib.rs".to_string(),
            path: "src/lib.rs".to_string(),
            kind: FileTreeNodeKind::File,
            is_expanded: false,
            is_selected,
            is_active,
            children: Vec::new(),
        }
    }

    #[test]
    fn selected_node_uses_no_background_even_when_panel_has_focus() {
        let selected = node(true, false);

        assert_eq!(row_background_color(&selected), None);
    }

    #[test]
    fn active_node_stays_highlighted_without_panel_focus() {
        let active = node(true, true);

        assert_eq!(row_background_color(&active), Some(color::COLOR_BG_ACTIVE));
    }

    #[test]
    fn focus_emphasis_tracks_selected_focus_state_even_for_active_node() {
        let active_selected = node(true, true);

        assert!(focus_emphasis_visible(&active_selected, true));
        assert!(!focus_emphasis_visible(&active_selected, false));
    }
}
