//! 应用状态模块聚合与导出入口。

mod desktop_app;
mod title_bar;
mod tool_bar;
pub use desktop_app::{
    ActiveEditorSnapshot, DesktopAppState, DesktopToast, DesktopToastEvent, DesktopToastLevel,
    DesktopUiAction,
};
pub use title_bar::{TitleBarAction, TitleBarState};
pub use tool_bar::{ToolBarEntry, ToolBarState};
pub use zom_protocol::{PanelDock, dock_targets, panel_dock};
pub use zom_workspace::{FileTreeNode, FileTreeNodeKind, FileTreeState};
pub use zom_workspace::{PaneState, TabState};
