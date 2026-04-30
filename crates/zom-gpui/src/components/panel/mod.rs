//! 面板组件聚合入口。

mod debug;
mod file_tree;
mod git;
mod language_servers;
mod outline;
mod placeholder;
mod project_search;
mod shortcut;
mod shell;
mod terminal;

pub(crate) use debug::DebugPanel;
pub(crate) use file_tree::FileTreePanel;
pub(crate) use git::GitPanel;
pub(crate) use language_servers::LanguageServersPanel;
pub(crate) use outline::OutlinePanel;
pub(crate) use project_search::ProjectSearchPanel;
pub(crate) use shortcut::ShortcutPanel;
pub(crate) use terminal::TerminalPanel;
