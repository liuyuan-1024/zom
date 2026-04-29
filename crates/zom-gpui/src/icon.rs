//! 全局图标资产 token 与路径映射。

/// 应用内允许使用的图标语义集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppIcon {
    Keyboard,
    Settings,
    FileTree,
    GitBranchAlt,
    ListTree,
    Search,
    BoltOutlined,
    Check,
    Terminal,
    Debug,
    Notification,
    Close,
    FileTreeFolderOpen,
    FileTreeFolder,
    FileTreeFile,
    ChevronUp,
    ChevronDown,
    FindReplaceCaseSensitive,
    FindReplaceWholeWord,
    FindReplaceRegex,
    FindReplaceNext,
    FindReplaceAll,
}

impl AppIcon {
    pub(crate) const fn asset_path(self) -> &'static str {
        match self {
            AppIcon::Keyboard => "icons/keyboard.svg",
            AppIcon::Settings => "icons/title_bar/settings.svg",
            AppIcon::FileTree => "icons/status_bar/file_tree.svg",
            AppIcon::GitBranchAlt => "icons/status_bar/git_branch_alt.svg",
            AppIcon::ListTree => "icons/status_bar/list_tree.svg",
            AppIcon::Search => "icons/status_bar/search.svg",
            AppIcon::BoltOutlined => "icons/status_bar/bolt_outlined.svg",
            AppIcon::Check => "icons/status_bar/check.svg",
            AppIcon::Terminal => "icons/status_bar/terminal.svg",
            AppIcon::Debug => "icons/status_bar/debug.svg",
            AppIcon::Notification => "icons/status_bar/notification.svg",
            AppIcon::Close => "icons/tab/close.svg",
            AppIcon::FileTreeFolderOpen => "icons/file_tree/folder_open.svg",
            AppIcon::FileTreeFolder => "icons/file_tree/folder.svg",
            AppIcon::FileTreeFile => "icons/file_tree/file.svg",
            AppIcon::ChevronUp => "icons/chevron/chevron_up.svg",
            AppIcon::ChevronDown => "icons/chevron/chevron_down.svg",
            AppIcon::FindReplaceCaseSensitive => "icons/find_replace/case_sensitive.svg",
            AppIcon::FindReplaceWholeWord => "icons/find_replace/whole_word.svg",
            AppIcon::FindReplaceRegex => "icons/find_replace/regex.svg",
            AppIcon::FindReplaceNext => "icons/find_replace/replace_next.svg",
            AppIcon::FindReplaceAll => "icons/find_replace/replace_all.svg",
        }
    }
}
