use zom_core::PaneId;

use crate::{
    state::{
        DesktopAppState, FileTreeNode, FileTreeNodeKind, FileTreeState, PaneState, TitleBarIcon,
        TitleBarState, ToolBarEntry, ToolBarIcon, ToolBarState,
    },
    utils,
};

impl DesktopAppState {
    /// 构造一个用于界面预览的示例状态。
    pub fn sample() -> Self {
        let workspace_name = utils::detect_workspace_project_name();

        Self {
            title_bar: TitleBarState {
                right_icons: vec![TitleBarIcon::Settings],
            },
            tool_bar: ToolBarState {
                left_tools: vec![
                    ToolBarEntry {
                        icon: ToolBarIcon::Files,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::GitBranch,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::Outline,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::Search,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::LanguageServer,
                    },
                ],
                cursor: "1:1".into(),
                language: "Rust".into(),
                line_ending: "LF".into(),
                encoding: "UTF-8".into(),
                right_tools: vec![
                    ToolBarEntry {
                        icon: ToolBarIcon::Terminal,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::Debug,
                    },
                    ToolBarEntry {
                        icon: ToolBarIcon::Notifications,
                    },
                ],
            },
            file_tree: FileTreeState {
                title: "EXPLORER".into(),
                roots: vec![directory(
                    &workspace_name,
                    "",
                    true,
                    vec![
                        directory(
                            ".github / workflows",
                            ".github/workflows",
                            false,
                            Vec::new(),
                        ),
                        directory("apps / zom-desktop", "apps/zom-desktop", false, Vec::new()),
                        directory(
                            "crates",
                            "crates",
                            true,
                            vec![
                                directory(
                                    "zom-app",
                                    "crates/zom-app",
                                    true,
                                    vec![
                                        directory(
                                            "src",
                                            "crates/zom-app/src",
                                            true,
                                            vec![
                                                file(
                                                    "lib.rs",
                                                    "crates/zom-app/src/lib.rs",
                                                    false,
                                                    false,
                                                ),
                                                file(
                                                    "sample.rs",
                                                    "crates/zom-app/src/sample.rs",
                                                    false,
                                                    false,
                                                ),
                                                file(
                                                    "state.rs",
                                                    "crates/zom-app/src/state.rs",
                                                    false,
                                                    false,
                                                ),
                                                file(
                                                    "utils.rs",
                                                    "crates/zom-app/src/utils.rs",
                                                    false,
                                                    false,
                                                ),
                                            ],
                                        ),
                                        file(
                                            "Cargo.toml",
                                            "crates/zom-app/Cargo.toml",
                                            false,
                                            false,
                                        ),
                                    ],
                                ),
                                directory("zom-core", "crates/zom-core", false, Vec::new()),
                                directory("zom-gpui", "crates/zom-gpui", false, Vec::new()),
                            ],
                        ),
                    ],
                )],
            },
            project_name: workspace_name.clone(),
            pane: PaneState {
                id: PaneId::new(1),
                tabs: Vec::new(),
                active_tab_index: None,
            },
        }
    }
}

/// 构造目录节点，简化示例文件树的声明。
fn directory(
    name: &str,
    path: &str,
    is_expanded: bool,
    children: Vec<FileTreeNode>,
) -> FileTreeNode {
    FileTreeNode {
        name: name.into(),
        path: path.into(),
        kind: FileTreeNodeKind::Directory,
        is_expanded,
        is_selected: false,
        is_active: false,
        children,
    }
}

/// 构造文件节点，简化示例文件树的声明。
fn file(name: &str, path: &str, is_selected: bool, is_active: bool) -> FileTreeNode {
    FileTreeNode {
        name: name.into(),
        path: path.into(),
        kind: FileTreeNodeKind::File,
        is_expanded: false,
        is_selected,
        is_active,
        children: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{state::DesktopAppState, utils};

    #[test]
    fn sample_state_has_buffers_and_file_tree_content() {
        let state = DesktopAppState::sample();

        assert!(!state.file_tree.roots.is_empty());
        assert!(state.pane.tabs.is_empty());
    }

    #[test]
    fn sample_state_starts_without_active_tab() {
        let state = DesktopAppState::sample();
        assert!(state.pane.active_tab().is_none());
    }

    #[test]
    fn split_lines_preserves_blank_lines() {
        let lines = utils::split_lines("a\n\nb\n");

        assert_eq!(lines, vec!["a", "", "b", ""]);
    }

    #[test]
    fn detect_line_ending_distinguishes_crlf_and_lf() {
        assert_eq!(utils::detect_line_ending("a\r\nb\r\n"), "CRLF");
        assert_eq!(utils::detect_line_ending("a\nb\n"), "LF");
    }
}
