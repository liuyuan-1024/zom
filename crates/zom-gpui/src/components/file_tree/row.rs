//! 文件树单行节点视图。

use gpui::{AnyElement, div, prelude::*, px, rgb, svg};
use zom_app::state::{FileTreeNode, FileTreeNodeKind};

use crate::spacing::SPACE_1;

/// 文件树每一行的统一高度。
const FILE_TREE_ROW_HEIGHT: f32 = 28.0;
/// 节点图标区域的固定宽度。
const FILE_TREE_KIND_BADGE_WIDTH: f32 = 18.0;
/// 文件树节点图标尺寸。
const FILE_TREE_NODE_ICON_SIZE: f32 = 15.0;
/// 文件树图标的统一中性色，只作用于图标本身。
const FILE_TREE_ICON_COLOR: u32 = 0x97a4bb;
/// 文件树节点进入普通选中态时的整行背景色。
const FILE_TREE_SELECTION_BG: u32 = 0x1b2330;
/// 文件树节点对应当前活动文件时的整行背景色。
const FILE_TREE_ACTIVE_BG: u32 = 0x232b38;

/// 渲染单个文件树节点行。
pub(super) fn render(node: &FileTreeNode) -> AnyElement {
    let row = div()
        .w_full()
        .h(px(FILE_TREE_ROW_HEIGHT))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(SPACE_1))
        .pr(px(SPACE_1))
        .child(render_kind_badge(node))
        .child(render_label(node));

    if node.is_active {
        row.bg(rgb(FILE_TREE_ACTIVE_BG)).into_any_element()
    } else if node.is_selected {
        row.bg(rgb(FILE_TREE_SELECTION_BG)).into_any_element()
    } else {
        row.into_any_element()
    }
}

/// 渲染节点名称。
fn render_label(node: &FileTreeNode) -> impl IntoElement {
    div().flex_1().text_sm().child(node.name.clone())
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
        .w(px(FILE_TREE_KIND_BADGE_WIDTH))
        .h(px(18.0))
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path(icon_path)
                .size(px(FILE_TREE_NODE_ICON_SIZE))
                .text_color(rgb(FILE_TREE_ICON_COLOR)),
        )
}

/// 渲染文件图标。
fn render_file_icon() -> impl IntoElement {
    div()
        .w(px(FILE_TREE_KIND_BADGE_WIDTH))
        .h(px(18.0))
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path("icons/file_tree/file.svg")
                .size(px(FILE_TREE_NODE_ICON_SIZE))
                .text_color(rgb(FILE_TREE_ICON_COLOR)),
        )
}
