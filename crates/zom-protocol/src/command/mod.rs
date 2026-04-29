//! 命令协议公共导出入口。

//! 命令协议层。
//! 这里表达的是“用户想做什么（Kind）”与“本次执行什么（Invocation）”。

mod invocation;
mod kind;

pub use invocation::{
    CommandInvocation, EditorAction, EditorInvocation, FileTreeAction, FindReplaceAction,
    FindReplaceRequest, TabAction, WorkspaceAction,
};
pub use kind::{
    CommandKind, CommandKindId, CommandKindSpec, CommandMeta, command_kind, command_kind_id,
    command_kind_spec, command_kind_spec_by_id, command_kind_spec_by_kind, command_kind_specs,
    command_meta,
};
