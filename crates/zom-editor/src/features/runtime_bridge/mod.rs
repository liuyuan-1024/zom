use zom_protocol::{
    DocumentVersion, EditorToRuntimeEvent, LineRange, RuntimeErrorCode, RuntimeRequestId,
    RuntimeResponse, RuntimeToEditorRequest, Selection, TextDelta,
};

use crate::features::editing::{
    state::{DocVersion, EditorState},
    transaction::{
        ApplyError, TextChange, TransactionMeta, TransactionResult, TransactionSource,
        TransactionSpec, apply_transaction,
    },
};
use crate::features::viewport::{ViewportModel, ViewportUpdate};

pub fn dispatch_runtime_request(
    state: &mut EditorState,
    request: RuntimeToEditorRequest,
) -> RuntimeResponse {
    match request {
        RuntimeToEditorRequest::RequestSnapshot => {
            RuntimeResponse::Snapshot(EditorToRuntimeEvent::Snapshot {
                version: protocol_version(state.version()),
                text: state.text(),
                selection: state.selection(),
            })
        }
        RuntimeToEditorRequest::ApplyEdits {
            request_id,
            expected_version,
            changes,
            selection,
        } => {
            let Some(changes) = protocol_changes_to_editor(changes) else {
                return RuntimeResponse::Error {
                    request_id,
                    code: RuntimeErrorCode::InvalidRequest,
                    current_version: protocol_version(state.version()),
                };
            };
            apply_runtime_transaction(
                state,
                request_id,
                TransactionSpec {
                    changes,
                    selection,
                    expected_version: Some(editor_version(expected_version)),
                    meta: TransactionMeta::from_source(TransactionSource::Runtime),
                },
            )
        }
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
                expected_version: expected_version.map(editor_version),
                meta: TransactionMeta::from_source(TransactionSource::Runtime),
            },
        ),
    }
}

pub fn dispatch_viewport_update(
    state: &EditorState,
    model: &mut ViewportModel,
    update: ViewportUpdate,
) -> Option<EditorToRuntimeEvent> {
    model.apply(
        protocol_version(state.version()),
        state.buffer().line_count(),
        update,
    )
}

fn apply_runtime_transaction(
    state: &mut EditorState,
    request_id: RuntimeRequestId,
    spec: TransactionSpec,
) -> RuntimeResponse {
    // 编辑域错误在桥接层被压缩为协议错误码，避免向 runtime 泄露内部细节。
    match apply_transaction(state, spec) {
        Ok(result) => {
            let event = event_from_transaction(state, &result);
            *state = result.state;
            RuntimeResponse::Ack {
                request_id,
                version: protocol_version(state.version()),
                event,
            }
        }
        Err(ApplyError::VersionMismatch { current_version }) => RuntimeResponse::Error {
            request_id,
            code: RuntimeErrorCode::VersionMismatch,
            current_version: protocol_version(current_version),
        },
        Err(ApplyError::OverlappingChanges { .. } | ApplyError::InvalidChangeRange { .. }) => {
            RuntimeResponse::Error {
                request_id,
                code: RuntimeErrorCode::InvalidRequest,
                current_version: protocol_version(state.version()),
            }
        }
    }
}

fn event_from_transaction(
    previous_state: &EditorState,
    result: &TransactionResult,
) -> Option<EditorToRuntimeEvent> {
    // 同时发生文本与选区变更时优先发 Delta，避免重复事件。
    if result.is_document_changed {
        let dirty_lines =
            collect_dirty_lines_for_changes(previous_state, &result.state, &result.applied_changes);
        return Some(EditorToRuntimeEvent::Delta {
            version: protocol_version(result.state.version()),
            changes: editor_changes_to_protocol(&result.applied_changes),
            selection: result.state.selection(),
            dirty_lines,
        });
    }

    if result.is_selection_changed {
        let dirty_lines =
            collect_dirty_lines_for_selection(previous_state.selection(), result.state.selection());
        return Some(EditorToRuntimeEvent::SelectionChanged {
            version: protocol_version(result.state.version()),
            selection: result.state.selection(),
            dirty_lines,
        });
    }

    None
}

fn protocol_changes_to_editor(changes: Vec<TextDelta>) -> Option<Vec<TextChange>> {
    changes
        .into_iter()
        .map(|change| {
            let from = usize::try_from(change.from).ok()?;
            let to = usize::try_from(change.to).ok()?;
            Some(TextChange::new(from, to, change.insert))
        })
        .collect()
}

fn editor_changes_to_protocol(changes: &[TextChange]) -> Vec<TextDelta> {
    changes
        .iter()
        .map(|change| {
            TextDelta::new(
                u64::try_from(change.from).expect("usize should fit into u64"),
                u64::try_from(change.to).expect("usize should fit into u64"),
                change.insert.clone(),
            )
        })
        .collect()
}

fn protocol_version(version: DocVersion) -> DocumentVersion {
    DocumentVersion::from(version.get())
}

fn editor_version(version: DocumentVersion) -> DocVersion {
    DocVersion::from(version.get())
}

fn collect_dirty_lines_for_changes(
    previous_state: &EditorState,
    next_state: &EditorState,
    changes: &[TextChange],
) -> Vec<LineRange> {
    let ranges = changes
        .iter()
        .map(|change| {
            let start_line = previous_state.offset_to_position(change.from).line;
            let end_line_before = previous_state.offset_to_position(change.to).line;
            let mapped_end = map_offset(change.to, changes).min(next_state.len());
            let end_line_after = next_state.offset_to_position(mapped_end).line;
            let end_line_exclusive = end_line_before.max(end_line_after).saturating_add(1);
            LineRange::new(start_line, end_line_exclusive)
        })
        .collect();
    merge_line_ranges(ranges)
}

fn collect_dirty_lines_for_selection(previous: Selection, next: Selection) -> Vec<LineRange> {
    let ranges = vec![
        LineRange::new(
            previous.anchor().line,
            previous.anchor().line.saturating_add(1),
        ),
        LineRange::new(
            previous.active().line,
            previous.active().line.saturating_add(1),
        ),
        LineRange::new(next.anchor().line, next.anchor().line.saturating_add(1)),
        LineRange::new(next.active().line, next.active().line.saturating_add(1)),
    ];
    merge_line_ranges(ranges)
}

fn merge_line_ranges(mut ranges: Vec<LineRange>) -> Vec<LineRange> {
    if ranges.is_empty() {
        return ranges;
    }
    ranges.sort_by_key(|range| (range.start_line, range.end_line_exclusive));
    let mut merged: Vec<LineRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && range.start_line <= last.end_line_exclusive
        {
            last.end_line_exclusive = last.end_line_exclusive.max(range.end_line_exclusive);
            continue;
        }
        merged.push(range);
    }
    merged
}

fn map_offset(offset: usize, changes: &[TextChange]) -> usize {
    let mut mapped = offset;
    for change in changes {
        let removed = change.to - change.from;
        let added = change.insert.len();
        if mapped < change.from {
            continue;
        }
        if mapped >= change.to {
            let delta = added as isize - removed as isize;
            mapped = shift_offset(mapped, delta);
            continue;
        }
        mapped = change.from + added;
    }
    mapped
}

fn shift_offset(offset: usize, delta: isize) -> usize {
    if delta >= 0 {
        offset + delta as usize
    } else {
        offset.saturating_sub(delta.unsigned_abs())
    }
}

#[cfg(test)]
mod tests {
    use zom_protocol::{Position, Selection, ViewportState};

    use crate::features::editing::state::{DocVersion, EditorState};

    use super::{
        EditorToRuntimeEvent, RuntimeErrorCode, RuntimeRequestId, RuntimeResponse,
        RuntimeToEditorRequest, dispatch_runtime_request, dispatch_viewport_update,
    };
    use crate::features::viewport::{ViewportModel, ViewportMutation, ViewportUpdate};
    use zom_protocol::{DocumentVersion, LineRange, TextDelta, ViewportInvalidationReason};

    #[test]
    fn request_snapshot_returns_full_state() {
        let mut state = EditorState::from_text("abc");
        let response =
            dispatch_runtime_request(&mut state, RuntimeToEditorRequest::RequestSnapshot);

        assert_eq!(
            response,
            RuntimeResponse::Snapshot(EditorToRuntimeEvent::Snapshot {
                version: DocumentVersion::zero(),
                text: "abc".to_string(),
                selection: Selection::caret(Position::zero()),
            })
        );
    }

    #[test]
    fn apply_edits_acks_and_emits_delta() {
        let mut state = EditorState::from_text("ab");
        let response = dispatch_runtime_request(
            &mut state,
            RuntimeToEditorRequest::ApplyEdits {
                request_id: RuntimeRequestId::new("req-1"),
                expected_version: DocumentVersion::zero(),
                changes: vec![TextDelta::new(1, 1, "X")],
                selection: Some(Selection::caret(Position::new(0, 2))),
            },
        );

        assert_eq!(state.text(), "aXb");
        assert_eq!(state.version(), DocVersion::from(1));
        assert_eq!(
            response,
            RuntimeResponse::Ack {
                request_id: RuntimeRequestId::new("req-1"),
                version: DocumentVersion::from(1),
                event: Some(EditorToRuntimeEvent::Delta {
                    version: DocumentVersion::from(1),
                    changes: vec![TextDelta::new(1, 1, "X")],
                    selection: Selection::caret(Position::new(0, 2)),
                    dirty_lines: vec![LineRange::new(0, 1)],
                }),
            }
        );
    }

    #[test]
    fn version_mismatch_returns_error() {
        let mut state = EditorState::from_text("ab");
        let response = dispatch_runtime_request(
            &mut state,
            RuntimeToEditorRequest::ApplyEdits {
                request_id: RuntimeRequestId::new("req-2"),
                expected_version: DocumentVersion::from(9),
                changes: vec![TextDelta::new(1, 1, "X")],
                selection: None,
            },
        );

        assert_eq!(
            response,
            RuntimeResponse::Error {
                request_id: RuntimeRequestId::new("req-2"),
                code: RuntimeErrorCode::VersionMismatch,
                current_version: DocumentVersion::zero(),
            }
        );
    }

    #[test]
    fn set_selection_emits_dirty_lines_for_old_and_new_caret_rows() {
        let mut state = EditorState::from_text("a\nb\nc");
        let response = dispatch_runtime_request(
            &mut state,
            RuntimeToEditorRequest::SetSelection {
                request_id: RuntimeRequestId::new("req-3"),
                expected_version: Some(DocumentVersion::zero()),
                selection: Selection::caret(Position::new(2, 1)),
            },
        );

        assert_eq!(
            response,
            RuntimeResponse::Ack {
                request_id: RuntimeRequestId::new("req-3"),
                version: DocumentVersion::from(1),
                event: Some(EditorToRuntimeEvent::SelectionChanged {
                    version: DocumentVersion::from(1),
                    selection: Selection::caret(Position::new(2, 1)),
                    dirty_lines: vec![LineRange::new(0, 1), LineRange::new(2, 3)],
                }),
            }
        );
    }

    #[test]
    fn invalid_u64_offsets_are_rejected_as_invalid_request() {
        let mut state = EditorState::from_text("ab");
        let response = dispatch_runtime_request(
            &mut state,
            RuntimeToEditorRequest::ApplyEdits {
                request_id: RuntimeRequestId::new("req-4"),
                expected_version: DocumentVersion::zero(),
                changes: vec![TextDelta::new(u64::MAX, u64::MAX, "X")],
                selection: None,
            },
        );

        assert_eq!(
            response,
            RuntimeResponse::Error {
                request_id: RuntimeRequestId::new("req-4"),
                code: RuntimeErrorCode::InvalidRequest,
                current_version: DocumentVersion::zero(),
            }
        );
    }

    #[test]
    fn viewport_invalidated_contract_variant_is_constructible() {
        let event = EditorToRuntimeEvent::ViewportInvalidated {
            version: DocumentVersion::from(7),
            dirty_lines: vec![LineRange::new(10, 20)],
            viewport: None,
            reason: ViewportInvalidationReason::LayoutChanged,
        };

        assert_eq!(
            event,
            EditorToRuntimeEvent::ViewportInvalidated {
                version: DocumentVersion::from(7),
                dirty_lines: vec![LineRange::new(10, 20)],
                viewport: None,
                reason: ViewportInvalidationReason::LayoutChanged,
            }
        );
    }

    #[test]
    fn dispatch_viewport_update_emits_event_for_scroll_resize_and_wrap() {
        let state = EditorState::from_text("a\nb\nc\nd\ne");
        let mut model = ViewportModel::new();

        let first = dispatch_viewport_update(
            &state,
            &mut model,
            ViewportUpdate::new(ViewportState::new(0, 2), 120, ViewportMutation::Scroll),
        )
        .expect("first viewport update should emit event");
        assert_eq!(
            first,
            EditorToRuntimeEvent::ViewportInvalidated {
                version: DocumentVersion::zero(),
                dirty_lines: vec![LineRange::new(0, 2)],
                viewport: Some(ViewportState::new(0, 2)),
                reason: ViewportInvalidationReason::ViewportScrolled,
            }
        );

        let resized = dispatch_viewport_update(
            &state,
            &mut model,
            ViewportUpdate::new(ViewportState::new(0, 3), 120, ViewportMutation::Resize),
        )
        .expect("resize should emit event");
        assert_eq!(
            resized,
            EditorToRuntimeEvent::ViewportInvalidated {
                version: DocumentVersion::zero(),
                dirty_lines: vec![LineRange::new(0, 3)],
                viewport: Some(ViewportState::new(0, 3)),
                reason: ViewportInvalidationReason::ViewportResized,
            }
        );

        let wrap_changed = dispatch_viewport_update(
            &state,
            &mut model,
            ViewportUpdate::new(
                ViewportState::new(0, 3),
                90,
                ViewportMutation::WrapWidthChanged,
            ),
        )
        .expect("wrap width change should emit event");
        assert_eq!(
            wrap_changed,
            EditorToRuntimeEvent::ViewportInvalidated {
                version: DocumentVersion::zero(),
                dirty_lines: vec![LineRange::new(0, 3)],
                viewport: Some(ViewportState::new(0, 3)),
                reason: ViewportInvalidationReason::WrapWidthChanged,
            }
        );
    }
}
