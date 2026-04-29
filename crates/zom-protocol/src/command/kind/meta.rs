//! 命令语义元信息定义。

use std::fmt;

/// 命令语义族的稳定字符串 ID，供跨层引用与文档检索。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandKindId(pub &'static str);

impl fmt::Display for CommandKindId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// 命令元信息（纯描述，不含行为）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandMeta {
    /// 稳定 ID，供跨层引用、埋点归因和文档检索。
    pub id: CommandKindId,
    /// 简短标题（适合按钮、菜单项等短文本场景）。
    pub title: &'static str,
    /// 语义说明（面向帮助文档或命令面板详情）。
    pub description: &'static str,
}
