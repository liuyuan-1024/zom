//! `zom-app` 负责应用层编排。
//! 当前阶段先提供桌面界面所需的静态应用状态，后续再接命令分发和服务注入。

use std::{env, fs, path::PathBuf};

use zom_text::TextBuffer;

/// 编辑器标签页的摘要信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferSummary {
    /// 标签页标题。
    pub title: String,
    /// 该标签页是否为当前激活项。
    pub is_active: bool,
}

/// 侧边栏分组信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarSection {
    /// 分组名称。
    pub title: String,
    /// 分组下的条目。
    pub items: Vec<String>,
}

/// 状态栏展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarItem {
    /// 图标文本。
    pub icon: String,
    /// 项目标签。
    pub label: String,
}

/// 当前用户摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSummary {
    /// 用户显示名称。
    pub name: String,
    /// 头像缩写。
    pub initials: String,
}

/// 状态栏展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarState {
    /// 左侧工具入口。
    pub left_items: Vec<StatusBarItem>,
    /// 光标位置文本。
    pub cursor: String,
    /// 当前文本语言类型。
    pub language: String,
    /// 右侧工具入口。
    pub right_items: Vec<StatusBarItem>,
}

/// 桌面端根界面使用的应用状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAppState {
    /// 产品名。
    pub product_name: String,
    /// 当前打开目录名称。
    pub workspace_name: String,
    /// 当前用户信息。
    pub user: UserSummary,
    /// 当前激活文件。
    pub active_buffer: String,
    /// 打开的标签页。
    pub buffers: Vec<BufferSummary>,
    /// 左侧侧边栏内容。
    pub sidebar_sections: Vec<SidebarSection>,
    /// 主编辑区预览文本。
    pub editor_preview: Vec<String>,
    /// 状态栏信息。
    pub status_bar: StatusBarState,
}

impl DesktopAppState {
    /// 构造一个用于界面预览的示例状态。
    pub fn sample() -> Self {
        let active_buffer = "crates/zom-core/src/lib.rs".to_string();
        let active_buffer_path = workspace_file(&active_buffer);
        let (editor_preview, line_ending, cursor) = load_buffer_preview(&active_buffer_path);
        let workspace_name = detect_workspace_name();
        let user = detect_user_summary();

        Self {
            product_name: "zom".into(),
            workspace_name,
            user,
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
            status_bar: StatusBarState {
                left_items: vec![
                    StatusBarItem {
                        icon: "▦".into(),
                        label: "Files".into(),
                    },
                    StatusBarItem {
                        icon: "⑂".into(),
                        label: "Git".into(),
                    },
                    StatusBarItem {
                        icon: "≣".into(),
                        label: "Outline".into(),
                    },
                    StatusBarItem {
                        icon: "⌕".into(),
                        label: "Search".into(),
                    },
                    StatusBarItem {
                        icon: "λ".into(),
                        label: "LSP".into(),
                    },
                ],
                cursor,
                language: "Rust".into(),
                right_items: vec![
                    StatusBarItem {
                        icon: line_ending,
                        label: "Line Endings".into(),
                    },
                    StatusBarItem {
                        icon: "UTF-8".into(),
                        label: "Encoding".into(),
                    },
                    StatusBarItem {
                        icon: "▹".into(),
                        label: "Terminal".into(),
                    },
                    StatusBarItem {
                        icon: "▷".into(),
                        label: "Debug".into(),
                    },
                    StatusBarItem {
                        icon: "●".into(),
                        label: "Notifications".into(),
                    },
                ],
            },
        }
    }
}

/// 生成工作区文件的绝对路径。
fn workspace_file(relative_path: &str) -> PathBuf {
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(relative_path)
}

/// 推断当前工作区目录名称。
fn detect_workspace_name() -> String {
    env::current_dir()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "workspace".into())
}

/// 推断当前用户名称和头像缩写。
fn detect_user_summary() -> UserSummary {
    let name = env::var("USER")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "user".into());

    let initials = name
        .split(['.', '_', '-'])
        .filter(|part| !part.is_empty())
        .take(2)
        .filter_map(|part| part.chars().next())
        .flat_map(|ch| ch.to_uppercase())
        .collect::<String>();

    UserSummary {
        name,
        initials: if initials.is_empty() {
            "U".into()
        } else {
            initials
        },
    }
}

/// 读取真实文件内容，并转换成界面需要的预览数据。
fn load_buffer_preview(path: &PathBuf) -> (Vec<String>, String, String) {
    let Ok(text) = fs::read_to_string(path) else {
        return (
            vec![format!("// failed to read {}", path.display())],
            "LF".into(),
            "1:1".into(),
        );
    };

    let buffer = TextBuffer::from_text(text.clone());
    let lines = split_lines(buffer.as_str());
    let line_ending = detect_line_ending(&text);
    let cursor = format!("{}:{}", lines.len().max(1), 1);

    (lines, line_ending, cursor)
}

/// 按编辑器视角拆分文本行，并保留空行。
fn split_lines(text: &str) -> Vec<String> {
    let mut lines = text
        .split('\n')
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect::<Vec<_>>();

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// 识别文本的换行风格。
fn detect_line_ending(text: &str) -> String {
    if text.contains("\r\n") {
        "CRLF".into()
    } else {
        "LF".into()
    }
}

#[cfg(test)]
mod tests {
    use super::{DesktopAppState, detect_line_ending, detect_user_summary, split_lines};

    #[test]
    fn sample_state_has_buffers_and_sidebar_content() {
        let state = DesktopAppState::sample();

        assert!(!state.buffers.is_empty());
        assert!(!state.sidebar_sections.is_empty());
        assert_eq!(state.product_name, "zom");
    }

    #[test]
    fn split_lines_preserves_blank_lines() {
        let lines = split_lines("a\n\nb\n");

        assert_eq!(lines, vec!["a", "", "b", ""]);
    }

    #[test]
    fn detect_line_ending_distinguishes_crlf_and_lf() {
        assert_eq!(detect_line_ending("a\r\nb\r\n"), "CRLF");
        assert_eq!(detect_line_ending("a\nb\n"), "LF");
    }

    #[test]
    fn user_summary_has_name_and_initials() {
        let user = detect_user_summary();

        assert!(!user.name.is_empty());
        assert!(!user.initials.is_empty());
    }
}
