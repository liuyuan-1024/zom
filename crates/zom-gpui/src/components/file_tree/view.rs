//! 文件树组件视图。

use gpui::{
    AnyElement, App, Context, CursorStyle, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Render, Styled, Window, div, prelude::*, px, rgb,
};
use zom_app::state::{FileTreeNode, FileTreeNodeKind, FileTreeState};
use zom_core::{Command, command::FileTreeCommand};

use super::{FILE_TREE_INDENT_STEP, row};
use crate::theme::{color, size};

/// 文件树面板视图。
pub struct FileTreePanel {
    state: FileTreeState,
    width: f32,
    is_dragging: bool,
    focus_handle: FocusHandle,
}

impl EventEmitter<Command> for FileTreePanel {}

impl FileTreePanel {
    /// 创建一个新的文件树面板。
    pub fn new(state: FileTreeState, cx: &mut Context<Self>) -> Self {
        Self {
            state,
            width: size::PANEL_WIDTH,
            is_dragging: false,
            focus_handle: cx.focus_handle(),
        }
    }

    /// 更新文件树展示状态（例如选中态、展开态）。
    pub fn set_state(&mut self, state: FileTreeState, cx: &mut Context<Self>) {
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
            .tab_index(0)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, _window, cx| {
                if has_any_modifier(&event.keystroke.modifiers) {
                    return;
                }

                let command = match event.keystroke.key.as_str() {
                    "up" => Command::from(FileTreeCommand::SelectPrev),
                    "down" => Command::from(FileTreeCommand::SelectNext),
                    "right" => Command::from(FileTreeCommand::ExpandOrDescend),
                    "left" => Command::from(FileTreeCommand::CollapseOrAscend),
                    "enter" => Command::from(FileTreeCommand::ActivateSelection),
                    _ => return,
                };

                cx.emit(command);
                cx.stop_propagation();
                cx.notify();
            }));

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
            .px(px(size::GAP_1))
            .children(self.state.roots.iter().map(render_node));

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
                    // TODO: 消除硬编码
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
        .ml(px(size::GAP_1))
        .pl(px(FILE_TREE_INDENT_STEP))
        .border_l_1()
        .border_color(rgb(color::COLOR_BORDER))
        .children(children.iter().map(render_node))
}

/// 递归渲染文件树节点 (保持为纯渲染逻辑)。
fn render_node(node: &FileTreeNode) -> AnyElement {
    let node_id = gpui::SharedString::from(format!("tree-node-{}", node.path));
    let is_dir = matches!(node.kind, FileTreeNodeKind::Directory);

    let row_view = div().id(node_id).child(row::render(node));

    let container = div().flex().flex_col().child(row_view);

    if is_dir && node.is_expanded {
        container
            .child(render_children(&node.children))
            .into_any_element()
    } else {
        container.into_any_element()
    }
}

fn has_any_modifier(modifiers: &gpui::Modifiers) -> bool {
    modifiers.control
        || modifiers.alt
        || modifiers.shift
        || modifiers.platform
        || modifiers.function
}
