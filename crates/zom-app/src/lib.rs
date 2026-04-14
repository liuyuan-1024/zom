//! `zom-app` 负责应用层编排。
//! 当前阶段先提供桌面界面所需的静态应用状态，后续再接命令分发和服务注入。

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
pub struct StatusBarState {
    /// 当前编辑模式。
    pub mode: String,
    /// 行尾格式。
    pub line_ending: String,
    /// 文本编码。
    pub encoding: String,
    /// 光标位置文本。
    pub cursor: String,
}

/// 桌面端根界面使用的应用状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAppState {
    /// 产品名。
    pub product_name: String,
    /// 当前工作区名称。
    pub workspace_name: String,
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
        Self {
            product_name: "zom".into(),
            workspace_name: "zom".into(),
            active_buffer: "crates/zom-core/src/lib.rs".into(),
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
            editor_preview: vec![
                "//! `zom-core` 是整个工程共享的协议层。".into(),
                "//! 这里只放跨 crate 都成立的基础类型、命令语义和输入协议。".into(),
                "".into(),
                "pub mod command;".into(),
                "pub mod direction;".into(),
                "pub mod ids;".into(),
                "pub mod input;".into(),
                "pub mod position;".into(),
                "pub mod range;".into(),
                "pub mod selection;".into(),
            ],
            status_bar: StatusBarState {
                mode: "NORMAL".into(),
                line_ending: "LF".into(),
                encoding: "UTF-8".into(),
                cursor: "Ln 10, Col 1".into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopAppState;

    #[test]
    fn sample_state_has_buffers_and_sidebar_content() {
        let state = DesktopAppState::sample();

        assert!(!state.buffers.is_empty());
        assert!(!state.sidebar_sections.is_empty());
        assert_eq!(state.product_name, "zom");
    }
}
