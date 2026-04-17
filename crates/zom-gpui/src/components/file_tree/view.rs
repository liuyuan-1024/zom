//! 文件树组件视图。

use gpui::{
    AnyElement, Context, CursorStyle, EventEmitter, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Render, Window, div, prelude::*, px, rgb,
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

/// 文件树节点点击事件。
#[derive(Debug, Clone)]
pub struct FileTreeNodeClicked {
    pub relative_path: String,
    pub kind: FileTreeNodeKind,
}

impl EventEmitter<FileTreeNodeClicked> for FileTreePanel {}

impl FileTreePanel {
    /// 创建一个新的文件树面板。
    pub fn new(state: FileTreeState) -> Self {
        Self {
            state,
            width: size::PANEL_WIDTH,
            is_dragging: false,
        }
    }

    /// 更新文件树展示状态（例如选中态、展开态）。
    pub fn set_state(&mut self, state: FileTreeState, cx: &mut Context<Self>) {
        self.state = state;
        cx.notify();
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
            .h_full()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(rgb(color::COLOR_BG_PANEL))
            .border_r_1()
            .border_color(rgb(color::COLOR_BORDER))
            .px(px(SPACE_1))
            .children(self.state.roots.iter().map(|node| render_node(node, cx)));

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
fn render_children(children: &[FileTreeNode], cx: &mut Context<FileTreePanel>) -> impl IntoElement {
    div()
        .ml(px(SPACE_1))
        .pl(px(FILE_TREE_INDENT_STEP))
        .border_l_1()
        .border_color(rgb(color::COLOR_BORDER))
        .children(children.iter().map(|node| render_node(node, cx)))
}

/// 递归渲染文件树节点 (保持为纯渲染逻辑)。
fn render_node(node: &FileTreeNode, cx: &mut Context<FileTreePanel>) -> AnyElement {
    let node_id = gpui::SharedString::from(format!("tree-node-{}", node.path));
    let is_dir = matches!(node.kind, FileTreeNodeKind::Directory);
    let clicked_path = node.path.clone();
    let clicked_kind = node.kind;

    // 给行容器增加 id 和点击事件
    let row_view = div()
        .id(node_id)
        .child(row::render(node))
        .on_click(cx.listener(move |_this, _event, _window, cx| {
            cx.emit(FileTreeNodeClicked {
                relative_path: clicked_path.clone(),
                kind: clicked_kind,
            });
            cx.notify();
        }));

    let container = div().flex().flex_col().child(row_view);

    if is_dir && node.is_expanded {
        container
            .child(render_children(&node.children, cx))
            .into_any_element()
    } else {
        container.into_any_element()
    }
}
