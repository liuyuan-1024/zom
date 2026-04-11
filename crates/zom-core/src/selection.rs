use crate::Position;

/// 单光标选区
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selection {
    pub anchor: Position,
    pub active: Position,
}

/// 多光标选区
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    pub selections: Vec<Selection>,
}
