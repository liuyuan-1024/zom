mod features;
mod viewer_layout;

pub use features::editing::{
    history::{EditorHistory, should_record_history},
    invocation::{InvocationResult, apply_editor_invocation},
    state::{DocVersion, EditorState, Offset},
    transaction::{
        ApplyError, TextChange, TransactionMeta, TransactionResult, TransactionSource,
        TransactionSpec, apply_transaction,
    },
};
pub use features::runtime_bridge::dispatch_runtime_request;
pub use viewer_layout::wrap_visual_line;
pub use zom_protocol::{
    DocumentVersion, EditorToRuntimeEvent, LineRange, RuntimeErrorCode, RuntimeRequestId,
    RuntimeResponse, RuntimeToEditorRequest, Selection, TextDelta, ViewportInvalidationReason,
    ViewportState,
};
