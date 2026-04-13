use crate::{Position, Range};

/// 编辑器中的单个光标或单个选区。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selection {
    /// 固定端。
    pub anchor: Position,
    /// 活动端，也就是当前光标端。
    pub active: Position,
}

impl Selection {
    /// 用锚点和活动点构造一个选区。
    pub fn new(anchor: Position, active: Position) -> Self {
        Self { anchor, active }
    }

    /// 创建一个没有范围长度的光标选区。
    pub fn caret(position: Position) -> Self {
        Self::new(position, position)
    }

    /// 判断当前选区是否只是一个光标点。
    pub fn is_caret(self) -> bool {
        self.anchor == self.active
    }

    /// 判断活动端是否位于锚点之前。
    pub fn is_reversed(self) -> bool {
        self.active < self.anchor
    }

    /// 取规范化后的起点。
    pub fn start(self) -> Position {
        self.anchor.min(self.active)
    }

    /// 取规范化后的终点。
    pub fn end(self) -> Position {
        self.anchor.max(self.active)
    }

    /// 将选区转换为一个规范化范围。
    pub fn range(self) -> Range {
        Range::new(self.start(), self.end())
    }
}

/// 多光标编辑时的一组稳定选区。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    /// 当前所有选区，默认约定第一个为主选区。
    pub selections: Vec<Selection>,
}

impl SelectionSet {
    /// 用一组选区构造选区集合。
    pub fn new(selections: Vec<Selection>) -> Self {
        Self { selections }
    }

    /// 用单个选区构造集合。
    pub fn single(selection: Selection) -> Self {
        Self::new(vec![selection])
    }

    /// 判断集合是否为空。
    pub fn is_empty(&self) -> bool {
        self.selections.is_empty()
    }

    /// 返回选区数量。
    pub fn len(&self) -> usize {
        self.selections.len()
    }

    /// 返回主选区。
    pub fn primary(&self) -> Option<&Selection> {
        self.selections.first()
    }
}

#[cfg(test)]
mod tests {
    use super::{Selection, SelectionSet};
    use crate::{Position, Range};

    #[test]
    fn caret_selection_has_no_extent() {
        let caret = Selection::caret(Position::new(2, 3));

        assert!(caret.is_caret());
        assert_eq!(
            caret.range(),
            Range::new(Position::new(2, 3), Position::new(2, 3))
        );
    }

    #[test]
    fn reversed_selection_normalizes_to_range() {
        let selection = Selection::new(Position::new(4, 8), Position::new(1, 2));

        assert!(selection.is_reversed());
        assert_eq!(selection.start(), Position::new(1, 2));
        assert_eq!(selection.end(), Position::new(4, 8));
    }

    #[test]
    fn primary_selection_comes_from_first_entry() {
        let first = Selection::caret(Position::new(0, 0));
        let second = Selection::caret(Position::new(1, 1));
        let set = SelectionSet::new(vec![first, second]);

        assert_eq!(set.len(), 2);
        assert_eq!(set.primary(), Some(&first));
    }
}
