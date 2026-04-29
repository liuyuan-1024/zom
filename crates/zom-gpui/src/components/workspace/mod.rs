//! Workspace 视图模块：负责布局、拖拽与面板装配。

mod dock_sizing;
mod render_find_replace;
mod render_notification;
mod render_workspace;
mod splitters;
mod view;

pub(crate) use view::WorkspaceView;
