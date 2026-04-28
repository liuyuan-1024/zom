//! `zom-editor` 的公共入口。
//! 负责承载编辑领域行为与文本视图语义。

mod features;
mod viewer_layout;

pub use features::editing::{
    invocation::{InvocationResult, apply_editor_invocation},
    state::{DocVersion, EditorState, Offset},
    transaction::{
        ApplyError, TextChange, TransactionMeta, TransactionResult, TransactionSource,
        TransactionSpec, apply_transaction,
    },
};
pub use features::runtime_bridge::{
    EditorToRuntimeEvent, RuntimeErrorCode, RuntimeRequestId, RuntimeResponse,
    RuntimeToEditorRequest, dispatch_runtime_request,
};
pub use viewer_layout::wrap_visual_line;
pub use zom_protocol::Selection;
