//! 文件树面板视图与状态同步逻辑。

use gpui::{
    AnyElement, App, Context, FocusHandle, Focusable, InteractiveElement, ParentElement, Render,
    ScrollHandle, Styled, Window, div, prelude::*, px, rgb,
};
use zom_runtime::state::{FileTreeNode, FileTreeNodeKind, FileTreeState};

use super::row;
use crate::components::panel::shell;
use crate::theme::{color, size};

/// 文件树面板视图。
pub struct FileTreePanel {
    state: FileTreeState,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    pending_scroll_to_selection: bool,
    is_logically_focused: bool,
}

impl FileTreePanel {
    /// 创建一个新的文件树面板。
    pub fn new(state: FileTreeState, is_logically_focused: bool, cx: &mut Context<Self>) -> Self {
        Self {
            state,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            pending_scroll_to_selection: true,
            is_logically_focused,
        }
    }

    /// 更新文件树展示状态（例如选中态、展开态）。
    pub fn set_state(
        &mut self,
        state: FileTreeState,
        is_logically_focused: bool,
        cx: &mut Context<Self>,
    ) {
        let previous_selected_path = selected_path(&self.state.roots).map(ToOwned::to_owned);
        let next_selected_path = selected_path(&state.roots).map(ToOwned::to_owned);
        if previous_selected_path != next_selected_path {
            self.pending_scroll_to_selection = true;
        }
        self.state = state;
        self.is_logically_focused = is_logically_focused;
        cx.notify();
    }
}

impl Focusable for FileTreePanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FileTreePanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let panel_has_focus = self.is_logically_focused;
        let visible_rows = collect_visible_rows(&self.state.roots);
        let selected_row_index = visible_rows.iter().position(|row| row.node.is_selected);

        if self.pending_scroll_to_selection {
            if let Some(selected_row_index) = selected_row_index {
                self.scroll_handle.scroll_to_item(selected_row_index);
            }
            self.pending_scroll_to_selection = false;
        }

        let tree_content = div()
            .id("file-tree-scroll")
            .size_full()
            .flex()
            .flex_col()
            .overflow_scroll()
            .track_scroll(&self.scroll_handle)
            .bg(rgb(color::COLOR_BG_PANEL))
            .px(px(size::GAP_1))
            .children(
                visible_rows
                    .iter()
                    .map(|row| render_visible_row(row, panel_has_focus)),
            );

        shell::render_shell("file-tree-container", &self.focus_handle, tree_content)
    }
}

/// 可见文件树行（平铺后便于滚动到指定选中项）。
struct VisibleRow<'a> {
    node: &'a FileTreeNode,
    depth: usize,
}

fn collect_visible_rows<'a>(roots: &'a [FileTreeNode]) -> Vec<VisibleRow<'a>> {
    let mut rows = Vec::new();
    collect_visible_rows_inner(roots, 0, &mut rows);
    rows
}

fn collect_visible_rows_inner<'a>(
    nodes: &'a [FileTreeNode],
    depth: usize,
    rows: &mut Vec<VisibleRow<'a>>,
) {
    for node in nodes {
        rows.push(VisibleRow { node, depth });
        if matches!(node.kind, FileTreeNodeKind::Directory) && node.is_expanded {
            collect_visible_rows_inner(&node.children, depth + 1, rows);
        }
    }
}

fn render_visible_row(row: &VisibleRow<'_>, panel_has_focus: bool) -> AnyElement {
    let node = row.node;
    let node_id = gpui::SharedString::from(format!("tree-node-{}", node.path));
    div()
        .id(node_id)
        .child(row::render(node, row.depth, panel_has_focus))
        .into_any_element()
}

fn selected_path(nodes: &[FileTreeNode]) -> Option<&str> {
    nodes.iter().find_map(selected_path_in_node)
}

fn selected_path_in_node(node: &FileTreeNode) -> Option<&str> {
    if node.is_selected {
        return Some(node.path.as_str());
    }
    node.children.iter().find_map(selected_path_in_node)
}
