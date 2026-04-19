//! 文件树组件视图。

use gpui::{
    AnyElement, App, Context, CursorStyle, FocusHandle, Focusable, InteractiveElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Render, ScrollHandle,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px, rgb,
};
use zom_app::state::{FileTreeNode, FileTreeNodeKind, FileTreeState};

use super::row;
use crate::theme::{color, size};

/// 文件树面板视图。
pub struct FileTreePanel {
    state: FileTreeState,
    width: f32,
    is_dragging: bool,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    pending_scroll_to_selection: bool,
}

impl FileTreePanel {
    /// 创建一个新的文件树面板。
    pub fn new(state: FileTreeState, cx: &mut Context<Self>) -> Self {
        Self {
            state,
            width: size::PANEL_WIDTH,
            is_dragging: false,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            pending_scroll_to_selection: true,
        }
    }

    /// 更新文件树展示状态（例如选中态、展开态）。
    pub fn set_state(&mut self, state: FileTreeState, cx: &mut Context<Self>) {
        let previous_selected_path = selected_path(&self.state.roots).map(ToOwned::to_owned);
        let next_selected_path = selected_path(&state.roots).map(ToOwned::to_owned);
        if previous_selected_path != next_selected_path {
            self.pending_scroll_to_selection = true;
        }
        self.state = state;
        cx.notify();
    }
}

impl Focusable for FileTreePanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FileTreePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 基础容器
        let mut container = div()
            .id("file-tree-container")
            .relative()
            .w(px(self.width))
            .h_full()
            .flex()
            .flex_row()
            .track_focus(&self.focus_handle)
            .tab_index(0);

        // 左侧实际文件树内容
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
            .h_full()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_scroll()
            .track_scroll(&self.scroll_handle)
            .bg(rgb(color::COLOR_BG_PANEL))
            .border_r_1()
            .border_color(rgb(color::COLOR_BORDER))
            .px(px(size::GAP_1))
            .children(visible_rows.iter().map(render_visible_row));

        // 右侧分割线：绝对定位，悬浮于边框之上，不占任何宽度
        let splitter = div()
            .id("splitter")
            .absolute()
            .right(px(-(size::GAP_0_5)))
            .w(px(size::GAP_1))
            .h_full()
            .cursor(CursorStyle::ResizeLeftRight)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                    this.is_dragging = true;
                    cx.notify();
                }),
            );

        // 装配内容
        container = container.child(tree_content).child(splitter);

        // 拖拽时的全局事件捕获网
        if self.is_dragging {
            container = container.child(
                div()
                    .absolute()
                    .top(px(-size::DRAG_CAPTURE_OFFSET))
                    .left(px(-size::DRAG_CAPTURE_OFFSET))
                    .w(px(size::DRAG_CAPTURE_SPAN))
                    .h(px(size::DRAG_CAPTURE_SPAN))
                    .cursor(CursorStyle::ResizeLeftRight)
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                        let mut new_width: f32 = event.position.x.into();
                        if new_width < size::PANEL_WIDTH_MIN {
                            new_width = size::PANEL_WIDTH_MIN;
                        }
                        if new_width > size::PANEL_WIDTH_MAX {
                            new_width = size::PANEL_WIDTH_MAX;
                        }
                        this.width = new_width;
                        cx.notify();
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                            this.is_dragging = false;
                            cx.notify();
                        }),
                    ),
            );
        }

        container
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

fn render_visible_row(row: &VisibleRow<'_>) -> AnyElement {
    let node = row.node;
    let node_id = gpui::SharedString::from(format!("tree-node-{}", node.path));
    div()
        .id(node_id)
        .child(row::render(node, row.depth))
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
