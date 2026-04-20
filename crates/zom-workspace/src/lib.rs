//! `zom-workspace` 的公共入口。
//! 负责承载工作区领域状态（文件树、窗格、面板布局）。

mod file_tree;
mod pane;
mod panel_layout;

pub use file_tree::{FileTreeNode, FileTreeNodeKind, FileTreeState};
pub use pane::{PaneState, TabState};
pub use panel_layout::{PanelDock, dock_targets, panel_dock};
