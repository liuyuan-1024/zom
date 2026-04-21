//! runtime 与编辑器核心之间的最小桥接协议。

use zom_protocol::Selection;

use crate::features::editing::{
    state::{DocVersion, EditorState},
    transaction::{
        ApplyError, TextChange, TransactionMeta, TransactionResult, TransactionSource,
        TransactionSpec, apply_transaction,
    },
};

/// runtime 请求的关联 ID。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeRequestId(String);

impl RuntimeRequestId {
    /// 创建请求 ID。
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<&str> for RuntimeRequestId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// editor 发送给 runtime 的事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorToRuntimeEvent {
    Snapshot {
        state: EditorState,
    },
    Delta {
        version: DocVersion,
        changes: Vec<TextChange>,
        selection: Selection,
    },
    SelectionChanged {
        version: DocVersion,
        selection: Selection,
    },
}

/// runtime 发给 editor 的请求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeToEditorRequest {
    RequestSnapshot,
    ApplyEdits {
        request_id: RuntimeRequestId,
        expected_version: DocVersion,
        changes: Vec<TextChange>,
        selection: Option<Selection>,
    },
    SetSelection {
        request_id: RuntimeRequestId,
        expected_version: Option<DocVersion>,
        selection: Selection,
    },
}

/// editor 对 runtime 请求的响应。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeResponse {
    Snapshot(EditorToRuntimeEvent),
    Ack {
        request_id: RuntimeRequestId,
        version: DocVersion,
        event: Option<EditorToRuntimeEvent>,
    },
    Error {
        request_id: RuntimeRequestId,
        code: RuntimeErrorCode,
        current_version: DocVersion,
    },
}

/// runtime 协议错误码。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeErrorCode {
    VersionMismatch,
    InvalidRequest,
}

/// 处理 runtime 请求并返回响应。
pub fn handle_runtime_request(
    state: &mut EditorState,
    request: RuntimeToEditorRequest,
) -> RuntimeResponse {
    match request {
        RuntimeToEditorRequest::RequestSnapshot => {
            RuntimeResponse::Snapshot(EditorToRuntimeEvent::Snapshot {
                state: state.clone(),
            })
        }
        RuntimeToEditorRequest::ApplyEdits {
            request_id,
            expected_version,
            changes,
            selection,
        } => apply_runtime_transaction(
            state,
            request_id,
            TransactionSpec {
                changes,
                selection,
                expected_version: Some(expected_version),
                meta: TransactionMeta::from_source(TransactionSource::Runtime),
            },
        ),
        RuntimeToEditorRequest::SetSelection {
            request_id,
            expected_version,
            selection,
        } => apply_runtime_transaction(
            state,
            request_id,
            TransactionSpec {
                changes: Vec::new(),
                selection: Some(selection),
                expected_version,
                meta: TransactionMeta::from_source(TransactionSource::Runtime),
            },
        ),
    }
}

fn apply_runtime_transaction(
    state: &mut EditorState,
    request_id: RuntimeRequestId,
    spec: TransactionSpec,
) -> RuntimeResponse {
    match apply_transaction(state, spec) {
        Ok(result) => {
            let event = event_from_transaction(&result);
            *state = result.state;
            RuntimeResponse::Ack {
                request_id,
                version: state.version(),
                event,
            }
        }
        Err(ApplyError::VersionMismatch { current_version }) => RuntimeResponse::Error {
            request_id,
            code: RuntimeErrorCode::VersionMismatch,
            current_version,
        },
        Err(ApplyError::OverlappingChanges { .. } | ApplyError::InvalidChangeRange { .. }) => {
            RuntimeResponse::Error {
                request_id,
                code: RuntimeErrorCode::InvalidRequest,
                current_version: state.version(),
            }
        }
    }
}

fn event_from_transaction(result: &TransactionResult) -> Option<EditorToRuntimeEvent> {
    if result.is_document_changed {
        return Some(EditorToRuntimeEvent::Delta {
            version: result.state.version(),
            changes: result.applied_changes.clone(),
            selection: result.state.selection(),
        });
    }

    if result.is_selection_changed {
        return Some(EditorToRuntimeEvent::SelectionChanged {
            version: result.state.version(),
            selection: result.state.selection(),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use zom_protocol::{Position, Selection};

    use crate::features::editing::{
        state::{DocVersion, EditorState},
        transaction::TextChange,
    };

    use super::{
        EditorToRuntimeEvent, RuntimeErrorCode, RuntimeRequestId, RuntimeResponse,
        RuntimeToEditorRequest, handle_runtime_request,
    };

    #[test]
    fn request_snapshot_returns_full_state() {
        let mut state = EditorState::from_text("abc");
        let response = handle_runtime_request(&mut state, RuntimeToEditorRequest::RequestSnapshot);

        assert_eq!(
            response,
            RuntimeResponse::Snapshot(EditorToRuntimeEvent::Snapshot { state })
        );
    }

    #[test]
    fn apply_edits_acks_and_emits_delta() {
        let mut state = EditorState::from_text("ab");
        let response = handle_runtime_request(
            &mut state,
            RuntimeToEditorRequest::ApplyEdits {
                request_id: RuntimeRequestId::new("req-1"),
                expected_version: DocVersion::zero(),
                changes: vec![TextChange::new(1, 1, "X")],
                selection: Some(Selection::caret(Position::new(0, 2))),
            },
        );

        assert_eq!(state.text(), "aXb");
        assert_eq!(state.version(), DocVersion::from(1));
        assert_eq!(
            response,
            RuntimeResponse::Ack {
                request_id: RuntimeRequestId::new("req-1"),
                version: DocVersion::from(1),
                event: Some(EditorToRuntimeEvent::Delta {
                    version: DocVersion::from(1),
                    changes: vec![TextChange::new(1, 1, "X")],
                    selection: Selection::caret(Position::new(0, 2)),
                }),
            }
        );
    }

    #[test]
    fn version_mismatch_returns_error() {
        let mut state = EditorState::from_text("ab");
        let response = handle_runtime_request(
            &mut state,
            RuntimeToEditorRequest::ApplyEdits {
                request_id: RuntimeRequestId::new("req-2"),
                expected_version: DocVersion::from(9),
                changes: vec![TextChange::new(1, 1, "X")],
                selection: None,
            },
        );

        assert_eq!(
            response,
            RuntimeResponse::Error {
                request_id: RuntimeRequestId::new("req-2"),
                code: RuntimeErrorCode::VersionMismatch,
                current_version: DocVersion::zero(),
            }
        );
    }
}
