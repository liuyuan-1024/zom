use crate::state::{FileTreeState, PaneState, TitleBarState, ToolBarState};

/// 桌面端根界面使用的应用状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopAppState {
    /// 顶部标题栏状态。
    pub title_bar: TitleBarState,
    /// 底部工具栏信息。
    pub tool_bar: ToolBarState,
    /// 左侧文件树内容。
    pub file_tree: FileTreeState,
    /// 窗格
    pub pane: PaneState,
    /// 主编辑区预览文本。（后续应下放到具体的 EditorView 状态中）
    pub editor_preview: Vec<String>,
    /// 当前打开项目的名称。
    pub project_name: String,
}
