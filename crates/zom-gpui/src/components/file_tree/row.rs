//! 文件树单行节点视图。

use gpui::{AnyElement, div, prelude::*, px, rgb, svg};
use zom_app::state::{FileTreeNode, FileTreeNodeKind};

use crate::theme::{color, size};

/// 渲染单个文件树节点行。
pub(super) fn render(node: &FileTreeNode) -> AnyElement {
    let row = div()
        .w_full()
        .flex()
        .flex_row()
        .items_center()
        .child(render_kind_badge(node))
        .child(render_label(node));

    // 使用统一的激活态和悬停态颜色
    if node.is_active {
        row.bg(rgb(color::COLOR_BG_ACTIVE)).into_any_element()
    } else if node.is_selected {
        row.bg(rgb(color::COLOR_BG_HOVER)).into_any_element()
    } else {
        row.into_any_element()
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
        .w(px(size::SPACE_3))
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
        .w(px(size::SPACE_3))
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
