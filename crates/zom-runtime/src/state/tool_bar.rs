//! 工具栏状态与图标语义定义。

use zom_protocol::CommandInvocation;
use zom_protocol::Position;

/// 工具栏展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolBarEntry {
    /// 该入口对应的命令语义。
    pub command: CommandInvocation,
}

/// 工具栏展示信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolBarState {
    /// 左侧工具入口。
    pub left_tools: Vec<ToolBarEntry>,
    /// 光标逻辑位置（零基行列）。
    pub cursor: Position,
    /// 当前文本语言类型。
    pub language: String,
    /// 当前文件换行符格式。
    pub line_ending: String,
    /// 当前文件编码。
    pub encoding: String,
    /// 右侧工具入口。
    pub right_tools: Vec<ToolBarEntry>,
}
