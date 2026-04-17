use std::path::Path;

use zom_core::{BufferId, PaneId};

use crate::{
    state::{
        DesktopAppState, FileTreeNode, FileTreeNodeKind, FileTreeState, PaneState, TabState,
        TitleBarIcon, TitleBarState, ToolBarEntry, ToolBarIcon, ToolBarState,
    },
    utils,
};

impl DesktopAppState {
    /// 构造一个用于界面预览的示例状态。
    pub fn sample() -> Self {
        let active_tab_relative_path = "crates/zom-core/src/lib.rs";
        let active_tab_absolute_path =
            utils::workspace_file_absolute_path(active_tab_relative_path);
        let (active_buffer_lines, line_ending, cursor) =
            utils::load_buffer_preview(&active_tab_absolute_path);
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
                cursor,
                language: "Rust".into(),
                line_ending,
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
                                                    true,
                                                    true,
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
                tabs: vec![
                    TabState {
                        buffer_id: BufferId::new(1),
                        title: "lib.rs".into(),
                        relative_path: active_tab_relative_path.into(),
                        buffer_lines: active_buffer_lines,
                    },
                    tab_from_file(BufferId::new(2), "crates/zom-core/src/selection.rs"),
                    tab_from_file(BufferId::new(3), "crates/zom-core/src/input.rs"),
                ],
                active_tab_index: Some(0),
            },
        }
    }
}

/// 从真实文件创建用于 Pane 的标签页状态。
fn tab_from_file(buffer_id: BufferId, relative_path: &str) -> TabState {
    let absolute_path = utils::workspace_file_absolute_path(relative_path);
    let (buffer_lines, _, _) = utils::load_buffer_preview(&absolute_path);

    TabState {
        buffer_id,
        title: file_name(relative_path),
        relative_path: relative_path.into(),
        buffer_lines,
    }
}

/// 从相对路径提取标签标题。
fn file_name(relative_path: &str) -> String {
    Path::new(relative_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| relative_path.to_string())
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
        assert!(!state.pane.tabs.is_empty());
    }

    #[test]
    fn sample_state_active_tab_has_loaded_file_content() {
        let state = DesktopAppState::sample();
        let active_tab = state.pane.active_tab().expect("active tab should exist");

        assert!(!active_tab.buffer_lines.is_empty());
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
