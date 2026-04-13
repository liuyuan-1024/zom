use crate::{Position, Range};

/// 编辑器中的单个光标或单个选区。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selection {
    /// 固定端。
    anchor: Position,
    /// 活动端，也就是当前光标端。
    active: Position,
}

impl Selection {
    /// 用锚点和活动点构造一个选区。
    pub fn new(anchor: Position, active: Position) -> Self {
        Self { anchor, active }
    }

    /// 返回锚点位置。
    pub fn anchor(self) -> Position {
        self.anchor
    }

    /// 返回活动点位置。
    pub fn active(self) -> Position {
        self.active
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

    /// 返回选区的逻辑排序键。
    pub fn sort_key(self) -> (Position, Position, Position, Position) {
        (self.start(), self.end(), self.anchor, self.active)
    }
}

/// 多光标编辑时的一组稳定选区。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    /// 当前所有选区，始终按逻辑位置排序。
    selections: Vec<Selection>,
    /// 主选区在有序数组中的索引。
    primary_index: Option<usize>,
}

impl SelectionSet {
    /// 用一组选区构造选区集合，移除重复项并按逻辑位置排序。
    pub fn new(selections: Vec<Selection>) -> Self {
        let primary = selections.first().copied();
        let selections = Self::normalize_selections(selections);
        let primary_index = primary
            .and_then(|selection| selections.iter().position(|current| *current == selection));

        Self {
            selections,
            primary_index,
        }
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

    /// 以切片形式返回已排序的全部选区。
    pub fn as_slice(&self) -> &[Selection] {
        &self.selections
    }

    /// 返回所有选区的只读迭代器。
    pub fn iter(&self) -> impl Iterator<Item = &Selection> {
        self.selections.iter()
    }

    /// 返回主选区。
    pub fn primary(&self) -> Option<&Selection> {
        self.primary_index
            .and_then(|index| self.selections.get(index))
    }

    /// 追加一个选区；如果已存在则保持集合不变。
    pub fn push(&mut self, selection: Selection) {
        if self.selections.contains(&selection) {
            return;
        }

        self.selections.push(selection);
        self.reindex_primary_after_normalize();
    }

    /// 将某个已存在或新加入的选区标记为主选区。
    pub fn set_primary(&mut self, selection: Selection) {
        if !self.selections.contains(&selection) {
            self.selections.push(selection);
        }

        self.reindex_primary_after_normalize_with(selection);
    }

    /// 重新规范化当前集合，保持主选区语义不变。
    pub fn normalize(&mut self) {
        self.reindex_primary_after_normalize();
    }

    fn normalize_selections(mut selections: Vec<Selection>) -> Vec<Selection> {
        selections.sort_by_key(|selection| selection.sort_key());
        selections.dedup();
        selections
    }

    fn reindex_primary_after_normalize(&mut self) {
        let primary = self.primary().copied();
        self.reindex_primary_after_normalize_with_optional(primary);
    }

    fn reindex_primary_after_normalize_with(&mut self, primary: Selection) {
        self.reindex_primary_after_normalize_with_optional(Some(primary));
    }

    fn reindex_primary_after_normalize_with_optional(&mut self, primary: Option<Selection>) {
        self.selections = Self::normalize_selections(std::mem::take(&mut self.selections));
        self.primary_index = primary.and_then(|selection| {
            self.selections
                .iter()
                .position(|current| *current == selection)
        });
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
        let first = Selection::caret(Position::new(1, 1));
        let second = Selection::caret(Position::new(0, 0));
        let set = SelectionSet::new(vec![first, second]);

        assert_eq!(set.len(), 2);
        assert_eq!(set.primary(), Some(&first));
        assert_eq!(set.as_slice(), &[second, first]);
    }

    #[test]
    fn selection_exposes_anchor_and_active_points() {
        let selection = Selection::new(Position::new(2, 1), Position::new(4, 3));

        assert_eq!(selection.anchor(), Position::new(2, 1));
        assert_eq!(selection.active(), Position::new(4, 3));
    }

    #[test]
    fn selection_set_deduplicates_and_sorts_by_logical_position() {
        let first = Selection::caret(Position::new(2, 2));
        let second = Selection::caret(Position::new(0, 0));
        let third = Selection::caret(Position::new(1, 1));
        let set = SelectionSet::new(vec![first, second, first, third]);

        assert_eq!(set.len(), 3);
        assert_eq!(set.as_slice(), &[second, third, first]);
    }

    #[test]
    fn push_keeps_selection_set_unique() {
        let first = Selection::caret(Position::new(0, 0));
        let mut set = SelectionSet::single(first);

        set.push(first);
        set.push(Selection::caret(Position::new(1, 2)));

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn set_primary_preserves_primary_semantics_after_sorting() {
        let first = Selection::caret(Position::new(2, 2));
        let second = Selection::caret(Position::new(0, 0));
        let mut set = SelectionSet::new(vec![first, second]);

        set.set_primary(second);

        assert_eq!(set.primary(), Some(&second));
        assert_eq!(set.as_slice(), &[second, first]);
    }

    #[test]
    fn normalize_rebuilds_sorted_unique_state() {
        let first = Selection::caret(Position::new(1, 1));
        let second = Selection::caret(Position::new(0, 0));
        let mut set = SelectionSet {
            selections: vec![first, second, first],
            primary_index: Some(0),
        };

        set.normalize();

        assert_eq!(set.as_slice(), &[second, first]);
        assert_eq!(set.primary(), Some(&first));
    }
}
