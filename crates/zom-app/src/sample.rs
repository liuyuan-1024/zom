use crate::{
    state::{
        BufferSummary, DesktopAppState, SidebarSection, TitleBarIcon, TitleBarState, ToolBarIcon,
        ToolBarItem, ToolBarState,
    },
    utils,
};

impl DesktopAppState {
    /// 构造一个用于界面预览的示例状态。
    pub fn sample() -> Self {
        let active_buffer = "crates/zom-core/src/lib.rs".to_string();
        let active_buffer_path = utils::workspace_file(&active_buffer);
        let (editor_preview, line_ending, cursor) = utils::load_buffer_preview(&active_buffer_path);
        let workspace_name = utils::detect_workspace_name();

        Self {
            title_bar: TitleBarState {
                right_items: vec![TitleBarIcon::Settings],
            },
            tool_bar: ToolBarState {
                left_items: vec![
                    ToolBarItem {
                        icon: ToolBarIcon::Files,
                    },
                    ToolBarItem {
                        icon: ToolBarIcon::GitBranch,
                    },
                    ToolBarItem {
                        icon: ToolBarIcon::Outline,
                    },
                    ToolBarItem {
                        icon: ToolBarIcon::Search,
                    },
                    ToolBarItem {
                        icon: ToolBarIcon::LanguageServer,
                    },
                ],
                cursor,
                language: "Rust".into(),
                line_ending,
                encoding: "UTF-8".into(),
                right_items: vec![
                    ToolBarItem {
                        icon: ToolBarIcon::Terminal,
                    },
                    ToolBarItem {
                        icon: ToolBarIcon::Debug,
                    },
                    ToolBarItem {
                        icon: ToolBarIcon::Notifications,
                    },
                ],
            },
            workspace_name,
            active_buffer,
            buffers: vec![
                BufferSummary {
                    title: "lib.rs".into(),
                    is_active: true,
                },
                BufferSummary {
                    title: "selection.rs".into(),
                    is_active: false,
                },
                BufferSummary {
                    title: "input.rs".into(),
                    is_active: false,
                },
            ],
            sidebar_sections: vec![
                SidebarSection {
                    title: "EXPLORER".into(),
                    items: vec![
                        "crates".into(),
                        "apps".into(),
                        "docs".into(),
                        "Cargo.toml".into(),
                    ],
                },
                SidebarSection {
                    title: "OPEN EDITORS".into(),
                    items: vec!["lib.rs".into(), "selection.rs".into(), "input.rs".into()],
                },
            ],
            editor_preview,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{state::DesktopAppState, utils};

    #[test]
    fn sample_state_has_buffers_and_sidebar_content() {
        let state = DesktopAppState::sample();

        assert!(!state.buffers.is_empty());
        assert!(!state.sidebar_sections.is_empty());
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
