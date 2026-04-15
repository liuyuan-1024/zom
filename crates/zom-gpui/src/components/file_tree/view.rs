//! 文件树组件视图。

use gpui::{AnyElement, div, prelude::*, px, rgb};
use zom_app::state::{FileTreeNode, FileTreeNodeKind, FileTreeState};

use super::{FILE_TREE_GUIDE_COLOR, FILE_TREE_INDENT_STEP, row};
use crate::spacing::SPACE_1;

/// 文件树面板的固定宽度。
const FILE_TREE_PANEL_WIDTH: f32 = 260.0;

/// 渲染完整文件树面板。
pub(crate) fn render(state: &FileTreeState) -> impl IntoElement {
    div()
        .w(px(FILE_TREE_PANEL_WIDTH))
        .h_full()
        .flex()
        .flex_col()
        .bg(rgb(0x0d1117))
        .border_r_1()
        .border_color(rgb(0x222938))
        .px(px(SPACE_1))
        .pb(px(SPACE_1))
        .gap(px(SPACE_1))
        .children(state.roots.iter().map(render_node))
}

/// 递归渲染文件树节点。
fn render_node(node: &FileTreeNode) -> AnyElement {
    let container = div().flex().flex_col().child(row::render(node));

    if matches!(node.kind, FileTreeNodeKind::Directory) && node.is_expanded {
        container
            .child(render_children(&node.children))
            .into_any_element()
    } else {
        container.into_any_element()
    }
}

/// 渲染某个目录节点的子树容器，并由容器自己提供连续导线。
fn render_children(children: &[FileTreeNode]) -> impl IntoElement {
    div()
        .ml(px(SPACE_1))
        .pl(px(FILE_TREE_INDENT_STEP))
        .border_l_1()
        .border_color(rgb(FILE_TREE_GUIDE_COLOR))
        .children(children.iter().map(render_node))
}
