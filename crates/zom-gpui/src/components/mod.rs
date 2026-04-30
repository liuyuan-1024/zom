//! UI 组件总入口与聚合导出。

pub(crate) mod bar;
pub(crate) mod chip;
pub(crate) mod editor;
pub(crate) mod overlay;
pub(crate) mod pane;
pub(crate) mod panel;
pub(crate) mod workspace;

pub(crate) use bar::{status_bar, title_bar};
pub(crate) use overlay::{settings as settings_overlay, toast as toast_overlay};
pub(crate) use pane::PaneView;
pub(crate) use panel::{
    DebugPanel, FileTreePanel, GitPanel, LanguageServersPanel, OutlinePanel, ProjectSearchPanel,
    ShortcutPanel, TerminalPanel,
};
pub(crate) use workspace::WorkspaceView;
