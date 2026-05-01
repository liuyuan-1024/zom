//! 编辑器与 runtime 之间的稳定事件契约。

use crate::Selection;

/// 编辑器文档版本号（协议层语义）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentVersion(u64);

impl DocumentVersion {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for DocumentVersion {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// runtime 发起请求时的去重/关联 ID。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeRequestId(String);

impl RuntimeRequestId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RuntimeRequestId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// 按 UTF-8 字节偏移描述的文本变更。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDelta {
    pub from: u64,
    pub to: u64,
    pub insert: String,
}

impl TextDelta {
    pub fn new(from: u64, to: u64, insert: impl Into<String>) -> Self {
        Self {
            from,
            to,
            insert: insert.into(),
        }
    }
}

/// 逻辑行脏区，采用半开区间 `[start_line, end_line_exclusive)`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineRange {
    pub start_line: u32,
    pub end_line_exclusive: u32,
}

impl LineRange {
    pub fn new(start_line: u32, end_line_exclusive: u32) -> Self {
        if start_line <= end_line_exclusive {
            Self {
                start_line,
                end_line_exclusive,
            }
        } else {
            Self {
                start_line: end_line_exclusive,
                end_line_exclusive: start_line,
            }
        }
    }
}

/// 逻辑视口状态（按行）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewportState {
    pub first_visible_line: u32,
    pub visible_line_count: u32,
}

impl ViewportState {
    pub fn new(first_visible_line: u32, visible_line_count: u32) -> Self {
        Self {
            first_visible_line,
            visible_line_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewportInvalidationReason {
    DocumentChanged,
    SelectionChanged,
    LayoutChanged,
    ViewportScrolled,
    ViewportResized,
    WrapWidthChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorToRuntimeEvent {
    Snapshot {
        version: DocumentVersion,
        text: String,
        selection: Selection,
    },
    Delta {
        version: DocumentVersion,
        changes: Vec<TextDelta>,
        selection: Selection,
        dirty_lines: Vec<LineRange>,
    },
    SelectionChanged {
        version: DocumentVersion,
        selection: Selection,
        dirty_lines: Vec<LineRange>,
    },
    ViewportInvalidated {
        version: DocumentVersion,
        dirty_lines: Vec<LineRange>,
        viewport: Option<ViewportState>,
        reason: ViewportInvalidationReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeToEditorRequest {
    RequestSnapshot,
    ApplyEdits {
        request_id: RuntimeRequestId,
        expected_version: DocumentVersion,
        changes: Vec<TextDelta>,
        selection: Option<Selection>,
    },
    SetSelection {
        request_id: RuntimeRequestId,
        expected_version: Option<DocumentVersion>,
        selection: Selection,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeResponse {
    Snapshot(EditorToRuntimeEvent),
    Ack {
        request_id: RuntimeRequestId,
        version: DocumentVersion,
        event: Option<EditorToRuntimeEvent>,
    },
    Error {
        request_id: RuntimeRequestId,
        code: RuntimeErrorCode,
        current_version: DocumentVersion,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeErrorCode {
    VersionMismatch,
    InvalidRequest,
}
