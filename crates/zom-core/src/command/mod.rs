//! 命令协议层。
//! 这里表达的是“用户想做什么（Kind）”与“本次执行什么（Invocation）”。

mod invocation;
mod kind;

pub use invocation::{
    CommandInvocation, EditorAction, EditorInvocation, FileTreeAction, TabAction, WorkspaceAction,
};
pub use kind::{
    Buildability, CommandKind, CommandKindId, CommandKindSpec, CommandMeta, CommandShortcut,
    ShortcutScope, command_kind, command_kind_id, command_kind_spec, command_kind_spec_by_id,
    command_kind_spec_by_kind, command_kind_specs, command_meta, default_shortcut_bindings,
    default_shortcuts, invocation_from_kind,
};
