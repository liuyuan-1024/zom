//! 文件树组件视图。

use gpui::{
    AnyElement, Context, CursorStyle, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Render, Window, div, prelude::*, px, rgb,
};
use zom_app::state::{FileTreeNode, FileTreeNodeKind, FileTreeState};

use super::{FILE_TREE_INDENT_STEP, row};
use crate::theme::{
    color,
    size::{self, SPACE_1},
};

/// 文件树面板视图。
pub struct FileTreePanel {
    state: FileTreeState,
    width: f32,
    is_dragging: bool,
}

impl FileTreePanel {
    /// 创建一个新的文件树面板。
    pub fn new(state: FileTreeState) -> Self {
        Self {
            state,
            width: size::PANEL_WIDTH,
            is_dragging: false,
        }
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
            .flex_row();

        // 左侧实际文件树内容
        let tree_content = div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(rgb(color::COLOR_BG_PANEL))
            .border_r_1()
            .border_color(rgb(color::COLOR_BORDER))
            .px(px(SPACE_1))
            .children(self.state.roots.iter().map(render_node));

        // 右侧分割线：绝对定位，悬浮于边框之上，不占任何宽度
        let splitter = div()
            .id("splitter")
            .absolute()
            .top(px(0.0))
            .right(px(-1.0))
            .w(px(2.0))
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
                    .top(px(-2000.0))
                    .left(px(-2000.0))
                    .w(px(10000.0))
                    .h(px(10000.0))
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

/// 渲染子树容器。
fn render_children(children: &[FileTreeNode]) -> impl IntoElement {
    div()
        .ml(px(SPACE_1))
        .pl(px(FILE_TREE_INDENT_STEP))
        .border_l_1()
        .border_color(rgb(color::COLOR_BORDER))
        .children(children.iter().map(render_node))
}

/// 递归渲染文件树节点 (保持为纯渲染逻辑)。
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
