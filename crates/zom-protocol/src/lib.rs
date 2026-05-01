//! `zom-protocol` 是整个工程共享的协议层。
//! 这里只放跨 crate 都成立的基础类型、命令语义和键盘协议。

/// 命令语义模型。
pub mod command;
/// 通用方向与坐标轴定义。
pub mod direction;
/// 编辑器事件与 runtime 交互契约。
pub mod editor_events;
/// 焦点目标定义。
pub mod focus;
/// 强类型 ID 定义。
pub mod ids;
/// 键盘协议模型（纯类型契约）。
pub mod keyboard;
/// 文本位置模型。
pub mod position;
/// 文本范围模型。
pub mod range;
/// 选区与多选区模型。
pub mod selection;

/// 统一导出命令协议。
pub use command::{
    CommandInvocation, CommandKind, CommandKindId, CommandKindSpec, CommandMeta, EditorAction,
    EditorInvocation, FileTreeAction, FindReplaceAction, FindReplaceRequest, TabAction,
    WorkspaceAction, command_kind, command_kind_id, command_kind_spec, command_kind_spec_by_id,
    command_kind_spec_by_kind, command_kind_specs, command_meta,
};
/// 统一导出方向类型。
pub use direction::{Axis, Direction};
/// 统一导出编辑器事件契约。
pub use editor_events::{
    DocumentVersion, EditorToRuntimeEvent, LineRange, RuntimeErrorCode, RuntimeRequestId,
    RuntimeResponse, RuntimeToEditorRequest, TextDelta, ViewportInvalidationReason, ViewportState,
};
/// 统一导出焦点、面板停靠位与悬浮层目标。
pub use focus::{FocusTarget, OverlayTarget, PanelDock, ToolBarSide, dock_targets, panel_dock};
/// 统一导出强类型 ID。
pub use ids::{BufferId, PaneId, WorkspaceId};
/// 统一导出键盘协议类型。
pub use keyboard::{
    EditorInputContext, InputContext, InputResolution, KeyCode, Keystroke, Modifiers,
};
/// 统一导出文本位置。
pub use position::Position;
/// 统一导出文本范围。
pub use range::Range;
/// 统一导出选区类型。
pub use selection::{Selection, SelectionSet};
