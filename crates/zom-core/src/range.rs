use crate::Position;

/// 文本模型中的逻辑范围，采用半开区间语义 `[start, end)`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    /// 范围起点。
    start: Position,
    /// 范围终点，不包含在范围内。
    end: Position,
}

impl Range {
    /// 用起点和终点构造一个范围，并保证结果是规范化的。
    pub fn new(start: Position, end: Position) -> Self {
        if start <= end {
            Self { start, end }
        } else {
            Self {
                start: end,
                end: start,
            }
        }
    }

    /// 返回规范化后的起点。
    pub fn start(self) -> Position {
        self.start
    }

    /// 返回规范化后的终点。
    pub fn end(self) -> Position {
        self.end
    }

    /// 返回一个保证 `start <= end` 的范围。
    pub fn normalized(self) -> Self {
        self
    }

    /// 判断该范围是否为空范围。
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// 判断一个位置是否落在当前范围内。
    pub fn contains(self, position: Position) -> bool {
        self.start <= position && position < self.end
    }

    /// 判断当前范围是否完整包含另一个范围。
    pub fn contains_range(self, other: Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// 判断两个范围是否存在交集。
    pub fn intersects(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::Range;
    use crate::Position;

    #[test]
    fn normalized_orders_reversed_ranges() {
        let range = Range::new(Position::new(3, 5), Position::new(1, 2));

        assert_eq!(
            range.normalized(),
            Range::new(Position::new(1, 2), Position::new(3, 5))
        );
        assert_eq!(range.start(), Position::new(1, 2));
        assert_eq!(range.end(), Position::new(3, 5));
    }

    #[test]
    fn empty_range_is_detected() {
        let point = Position::new(1, 1);
        assert!(Range::new(point, point).is_empty());
    }

    #[test]
    fn contains_uses_half_open_semantics() {
        let range = Range::new(Position::new(0, 2), Position::new(0, 5));

        assert!(range.contains(Position::new(0, 2)));
        assert!(range.contains(Position::new(0, 4)));
        assert!(!range.contains(Position::new(0, 5)));
    }

    #[test]
    fn contains_range_requires_full_coverage() {
        let outer = Range::new(Position::new(1, 0), Position::new(3, 0));
        let inner = Range::new(Position::new(1, 2), Position::new(2, 4));
        let overlapping = Range::new(Position::new(2, 8), Position::new(4, 1));

        assert!(outer.contains_range(inner));
        assert!(!outer.contains_range(overlapping));
    }

    #[test]
    fn intersects_detects_overlap_but_respects_half_open_end() {
        let left = Range::new(Position::new(0, 0), Position::new(0, 3));
        let right = Range::new(Position::new(0, 2), Position::new(0, 5));
        let touching = Range::new(Position::new(0, 3), Position::new(0, 7));

        assert!(left.intersects(right));
        assert!(!left.intersects(touching));
    }
}
