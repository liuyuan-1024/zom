//! 全局图标资产 token 与路径映射。

/// 应用内允许使用的图标语义集合。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppIcon {
    Keyboard,
    TitleSettings,
    ToolFileTree,
    ToolGitBranchAlt,
    ToolListTree,
    ToolSearch,
    ToolBoltOutlined,
    ToolTerminal,
    ToolDebug,
    ToolNotification,
    TabClose,
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
            AppIcon::TitleSettings => "icons/title_bar/title_settings.svg",
            AppIcon::ToolFileTree => "icons/tool_bar/tool_file_tree.svg",
            AppIcon::ToolGitBranchAlt => "icons/tool_bar/tool_git_branch_alt.svg",
            AppIcon::ToolListTree => "icons/tool_bar/tool_list_tree.svg",
            AppIcon::ToolSearch => "icons/tool_bar/tool_search.svg",
            AppIcon::ToolBoltOutlined => "icons/tool_bar/tool_bolt_outlined.svg",
            AppIcon::ToolTerminal => "icons/tool_bar/tool_terminal.svg",
            AppIcon::ToolDebug => "icons/tool_bar/tool_debug.svg",
            AppIcon::ToolNotification => "icons/tool_bar/tool_notification.svg",
            AppIcon::TabClose => "icons/tab/close.svg",
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
