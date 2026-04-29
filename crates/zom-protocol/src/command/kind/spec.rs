//! 命令语义目录条目定义。

use crate::command::kind::{CommandKind, CommandKindId, CommandMeta};

/// 命令语义族统一声明结构。
#[derive(Debug, Clone, Copy)]
pub struct CommandKindSpec {
    /// 稳定语义族。
    pub kind: CommandKind,
    /// 只读元信息（UI 文案 / 文档引用）。
    pub meta: CommandMeta,
}

impl CommandKindSpec {
    /// 创建一条命令语义族声明。
    ///
    /// 该结构是命令目录的单一声明单元：`kind` 负责稳定分类，
    /// `meta` 负责对外展示与文档索引。
    pub const fn new(
        kind: CommandKind,
        id: &'static str,
        title: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            kind,
            meta: CommandMeta {
                id: CommandKindId(id),
                title,
                description,
            },
        }
    }
}
