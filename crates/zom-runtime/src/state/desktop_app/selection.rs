//! 编辑选区语义（SelectAll）状态承接。

use zom_editor::{
    EditorState, TransactionMeta, TransactionSource, TransactionSpec, apply_transaction,
};
use zom_protocol::{Position, Selection};

use super::DesktopAppState;

impl DesktopAppState {
    pub(super) fn select_all_in_editor(&self, current: &EditorState) -> EditorState {
        let full_selection =
            Selection::new(Position::zero(), current.offset_to_position(current.len()));
        if current.selection() == full_selection {
            return current.clone();
        }

        let spec = TransactionSpec {
            changes: Vec::new(),
            selection: Some(full_selection),
            meta: TransactionMeta::from_source(TransactionSource::Keyboard),
            expected_version: Some(current.version()),
        };
        apply_transaction(current, spec)
            .map(|result| result.state)
            .unwrap_or_else(|_| current.clone())
    }
}
