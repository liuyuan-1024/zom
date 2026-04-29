//! 文件树面板视图与状态同步逻辑。

use gpui::{
    AnyElement, App, Context, Entity, FocusHandle, Focusable, InteractiveElement, ParentElement,
    Render, ScrollHandle, Styled, Window, div, prelude::*, px, rgb,
};
use zom_runtime::state::{FileTreeNode, FileTreeNodeKind};

use super::row;
use crate::components::panel::shell::PanelShell;
use crate::root_view::store::AppStore;
use crate::theme::{color, size};

pub struct FileTreePanel {
    store: Entity<AppStore>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    should_scroll_to_selection: bool,
    previous_selected_path: Option<String>,
}

impl FileTreePanel {
    pub fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        cx.observe(&store, |this, store, cx| {
            let roots = store.read(cx).select_file_tree_state().roots;
            let next_selected_path = selected_path(&roots).map(ToOwned::to_owned);
            if this.previous_selected_path != next_selected_path {
                this.should_scroll_to_selection = true;
                this.previous_selected_path = next_selected_path;
            }
            cx.notify();
        })
        .detach();

        let roots = store.read(cx).select_file_tree_state().roots;
        Self {
            store,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            should_scroll_to_selection: true,
            previous_selected_path: selected_path(&roots).map(ToOwned::to_owned),
        }
    }
}

impl Focusable for FileTreePanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FileTreePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        let state = store.select_file_tree_state();
        let is_panel_focused = store.select_focused_target() == zom_protocol::FocusTarget::FileTreePanel;
        let visible_rows = collect_visible_rows(&state.roots);
        let selected_row_index = visible_rows.iter().position(|row| row.node.is_selected);

        if self.should_scroll_to_selection {
            if let Some(selected_row_index) = selected_row_index {
                self.scroll_handle.scroll_to_item(selected_row_index);
            }
            self.should_scroll_to_selection = false;
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
                    .map(|row| render_visible_row(row, is_panel_focused)),
            );

        PanelShell::new("file-tree-container")
            .track_focus(&self.focus_handle)
            .child(tree_content)
    }
}

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

fn render_visible_row(row: &VisibleRow<'_>, is_panel_focused: bool) -> AnyElement {
    let node = row.node;
    let node_id = gpui::SharedString::from(format!("tree-node-{}", node.path));
    div()
        .id(node_id)
        .child(row::render(node, row.depth, is_panel_focused))
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
