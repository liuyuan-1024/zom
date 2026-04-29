//! 全局图标资产 token 与路径映射。

/// 应用内允许使用的图标语义集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppIcon {
    // 标题栏
    Settings,

    // 状态栏
    FileTree,
    GitBranchAlt,
    ListTree,
    Search,
    BoltOutlined,
    Check,
    DiagnosticsWarn,
    Terminal,
    Debug,
    Notification,
    Keyboard,

    // 关闭
    Close,

    // 文件树
    FileTreeFolderOpen,
    FileTreeFolder,
    FileTreeFile,

    // 尖括号
    ChevronUp,
    ChevronDown,

    // 查找替换
    FindReplaceCaseSensitive,
    FindReplaceWholeWord,
    FindReplaceRegex,
    FindReplaceNext,
    FindReplaceAll,
}

impl AppIcon {
    /// 返回图标资源相对路径。
    ///
    /// 所有 UI 组件应只通过该枚举取图标，避免散落硬编码路径。
    pub(crate) const fn asset_path(self) -> &'static str {
        match self {
            // 标题栏
            AppIcon::Settings => "icons/title_bar/settings.svg",

            // 状态栏
            AppIcon::FileTree => "icons/status_bar/file_tree.svg",
            AppIcon::GitBranchAlt => "icons/status_bar/git_branch_alt.svg",
            AppIcon::ListTree => "icons/status_bar/list_tree.svg",
            AppIcon::Search => "icons/status_bar/search.svg",
            AppIcon::BoltOutlined => "icons/status_bar/bolt_outlined.svg",
            AppIcon::Check => "icons/status_bar/check.svg",
            AppIcon::DiagnosticsWarn => "icons/status_bar/diagnostics_warn.svg",
            AppIcon::Terminal => "icons/status_bar/terminal.svg",
            AppIcon::Debug => "icons/status_bar/debug.svg",
            AppIcon::Notification => "icons/status_bar/notification.svg",
            AppIcon::Keyboard => "icons/status_bar/keyboard.svg",

            // 关闭
            AppIcon::Close => "icons/tab/close.svg",

            // 文件树
            AppIcon::FileTreeFolderOpen => "icons/file_tree/folder_open.svg",
            AppIcon::FileTreeFolder => "icons/file_tree/folder.svg",
            AppIcon::FileTreeFile => "icons/file_tree/file.svg",

            // 尖括号
            AppIcon::ChevronUp => "icons/chevron/chevron_up.svg",
            AppIcon::ChevronDown => "icons/chevron/chevron_down.svg",

            // 查找替换
            AppIcon::FindReplaceCaseSensitive => "icons/find_replace/case_sensitive.svg",
            AppIcon::FindReplaceWholeWord => "icons/find_replace/whole_word.svg",
            AppIcon::FindReplaceRegex => "icons/find_replace/regex.svg",
            AppIcon::FindReplaceNext => "icons/find_replace/replace_next.svg",
            AppIcon::FindReplaceAll => "icons/find_replace/replace_all.svg",
        }
    }
}
