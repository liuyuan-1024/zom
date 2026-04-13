use crate::Position;

/// 文本模型中的逻辑范围，采用半开区间语义 `[start, end)`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    /// 范围起点。
    pub start: Position,
    /// 范围终点，不包含在范围内。
    pub end: Position,
}

impl Range {
    /// 用起点和终点构造一个范围。
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// 返回一个保证 `start <= end` 的范围。
    pub fn normalized(self) -> Self {
        if self.start <= self.end {
            self
        } else {
            Self::new(self.end, self.start)
        }
    }

    /// 判断该范围是否为空范围。
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// 判断一个位置是否落在当前范围内。
    pub fn contains(self, position: Position) -> bool {
        let normalized = self.normalized();
        normalized.start <= position && position < normalized.end
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
}
