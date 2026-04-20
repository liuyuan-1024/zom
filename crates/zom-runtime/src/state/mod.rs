//! 应用状态模块聚合与导出入口。

mod desktop_app;
mod file_tree;
mod pane;
mod panel_layout;
mod title_bar;
mod tool_bar;
pub use desktop_app::{DesktopAppState, DesktopUiAction};
pub use file_tree::{FileTreeNode, FileTreeNodeKind, FileTreeState};
pub use pane::{PaneState, TabState};
pub use panel_layout::{PanelDock, dock_targets, panel_dock};
pub use title_bar::{TitleBarAction, TitleBarState};
pub use tool_bar::{ToolBarEntry, ToolBarState};
