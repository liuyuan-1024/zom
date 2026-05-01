use zom_protocol::{
    DocumentVersion, EditorToRuntimeEvent, LineRange, ViewportInvalidationReason, ViewportState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportMutation {
    Scroll,
    Resize,
    WrapWidthChanged,
}

impl ViewportMutation {
    fn reason(self) -> ViewportInvalidationReason {
        match self {
            Self::Scroll => ViewportInvalidationReason::ViewportScrolled,
            Self::Resize => ViewportInvalidationReason::ViewportResized,
            Self::WrapWidthChanged => ViewportInvalidationReason::WrapWidthChanged,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportUpdate {
    pub viewport: ViewportState,
    pub wrap_column: u32,
    pub mutation: ViewportMutation,
}

impl ViewportUpdate {
    pub fn new(viewport: ViewportState, wrap_column: u32, mutation: ViewportMutation) -> Self {
        Self {
            viewport,
            wrap_column: wrap_column.max(1),
            mutation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ViewportSnapshot {
    viewport: ViewportState,
    wrap_column: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewportModel {
    current: Option<ViewportSnapshot>,
}

impl ViewportModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(
        &mut self,
        version: DocumentVersion,
        document_line_count: u32,
        update: ViewportUpdate,
    ) -> Option<EditorToRuntimeEvent> {
        let normalized = normalize_viewport(update.viewport, document_line_count);
        let next = ViewportSnapshot {
            viewport: normalized,
            wrap_column: update.wrap_column.max(1),
        };

        if self.current == Some(next) {
            return None;
        }

        let dirty_lines = match self.current {
            Some(previous) => merge_line_ranges(vec![
                viewport_to_line_range(previous.viewport),
                viewport_to_line_range(next.viewport),
            ]),
            None => vec![viewport_to_line_range(next.viewport)],
        };

        self.current = Some(next);
        Some(EditorToRuntimeEvent::ViewportInvalidated {
            version,
            dirty_lines,
            viewport: Some(next.viewport),
            reason: update.mutation.reason(),
        })
    }
}

fn normalize_viewport(viewport: ViewportState, document_line_count: u32) -> ViewportState {
    let total_lines = document_line_count.max(1);
    let first_visible = viewport
        .first_visible_line
        .min(total_lines.saturating_sub(1));
    let max_count = total_lines.saturating_sub(first_visible).max(1);
    let visible_count = viewport.visible_line_count.max(1).min(max_count);
    ViewportState::new(first_visible, visible_count)
}

fn viewport_to_line_range(viewport: ViewportState) -> LineRange {
    LineRange::new(
        viewport.first_visible_line,
        viewport
            .first_visible_line
            .saturating_add(viewport.visible_line_count),
    )
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

#[cfg(test)]
mod tests {
    use zom_protocol::{DocumentVersion, EditorToRuntimeEvent, LineRange, ViewportState};

    use super::{ViewportModel, ViewportMutation, ViewportUpdate};

    #[test]
    fn emits_invalidation_when_scrolled() {
        let mut model = ViewportModel::new();
        let _ = model.apply(
            DocumentVersion::from(1),
            200,
            ViewportUpdate::new(ViewportState::new(0, 20), 120, ViewportMutation::Scroll),
        );

        let event = model
            .apply(
                DocumentVersion::from(2),
                200,
                ViewportUpdate::new(ViewportState::new(10, 20), 120, ViewportMutation::Scroll),
            )
            .expect("scroll should invalidate viewport");

        assert_eq!(
            event,
            EditorToRuntimeEvent::ViewportInvalidated {
                version: DocumentVersion::from(2),
                dirty_lines: vec![LineRange::new(0, 30)],
                viewport: Some(ViewportState::new(10, 20)),
                reason: zom_protocol::ViewportInvalidationReason::ViewportScrolled,
            }
        );
    }

    #[test]
    fn emits_invalidation_when_resized() {
        let mut model = ViewportModel::new();
        let _ = model.apply(
            DocumentVersion::from(1),
            200,
            ViewportUpdate::new(ViewportState::new(10, 20), 100, ViewportMutation::Scroll),
        );

        let event = model
            .apply(
                DocumentVersion::from(2),
                200,
                ViewportUpdate::new(ViewportState::new(10, 30), 100, ViewportMutation::Resize),
            )
            .expect("resize should invalidate viewport");

        assert_eq!(
            event,
            EditorToRuntimeEvent::ViewportInvalidated {
                version: DocumentVersion::from(2),
                dirty_lines: vec![LineRange::new(10, 40)],
                viewport: Some(ViewportState::new(10, 30)),
                reason: zom_protocol::ViewportInvalidationReason::ViewportResized,
            }
        );
    }

    #[test]
    fn emits_invalidation_when_wrap_width_changes() {
        let mut model = ViewportModel::new();
        let _ = model.apply(
            DocumentVersion::from(1),
            200,
            ViewportUpdate::new(ViewportState::new(10, 20), 100, ViewportMutation::Scroll),
        );

        let event = model
            .apply(
                DocumentVersion::from(2),
                200,
                ViewportUpdate::new(
                    ViewportState::new(10, 20),
                    80,
                    ViewportMutation::WrapWidthChanged,
                ),
            )
            .expect("wrap width change should invalidate viewport");

        assert_eq!(
            event,
            EditorToRuntimeEvent::ViewportInvalidated {
                version: DocumentVersion::from(2),
                dirty_lines: vec![LineRange::new(10, 30)],
                viewport: Some(ViewportState::new(10, 20)),
                reason: zom_protocol::ViewportInvalidationReason::WrapWidthChanged,
            }
        );
    }

    #[test]
    fn unchanged_viewport_produces_no_event() {
        let mut model = ViewportModel::new();
        let _ = model.apply(
            DocumentVersion::from(1),
            200,
            ViewportUpdate::new(ViewportState::new(10, 20), 100, ViewportMutation::Scroll),
        );

        let event = model.apply(
            DocumentVersion::from(2),
            200,
            ViewportUpdate::new(ViewportState::new(10, 20), 100, ViewportMutation::Scroll),
        );
        assert_eq!(event, None);
    }

    #[test]
    fn viewport_is_clamped_to_document_line_count() {
        let mut model = ViewportModel::new();
        let event = model
            .apply(
                DocumentVersion::from(1),
                3,
                ViewportUpdate::new(ViewportState::new(20, 50), 100, ViewportMutation::Scroll),
            )
            .expect("first update should emit event");

        assert_eq!(
            event,
            EditorToRuntimeEvent::ViewportInvalidated {
                version: DocumentVersion::from(1),
                dirty_lines: vec![LineRange::new(2, 3)],
                viewport: Some(ViewportState::new(2, 1)),
                reason: zom_protocol::ViewportInvalidationReason::ViewportScrolled,
            }
        );
    }
}
