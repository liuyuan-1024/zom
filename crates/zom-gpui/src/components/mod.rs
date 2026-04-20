//! UI 组件总入口与聚合导出。

pub(crate) mod bar;
pub(crate) mod chip;
pub(crate) mod overlay;
pub(crate) mod pane;
pub(crate) mod panel;

pub(crate) use bar::{title_bar, tool_bar};
pub(crate) use overlay::settings as settings_overlay;
pub(crate) use pane::PaneView;
pub(crate) use panel::{
    DebugPanel, FileTreePanel, GitPanel, LanguageServersPanel, NotificationPanel, OutlinePanel,
    ProjectSearchPanel, TerminalPanel,
};
