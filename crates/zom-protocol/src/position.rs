//! 二维文本坐标 Position 值对象定义。

/// 文本中的逻辑位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// 0 基行号；比较时优先按行排序。
    pub line: u32,
    /// 0 基列号；仅在行号相同的情况下参与排序。
    pub column: u32,
}

impl Position {
    /// 构造一个新的逻辑位置。
    ///
    /// 该类型只表达协议层坐标，不在这里做“是否越界”的文档约束校验。
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }

    /// 返回文档起点位置。
    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    /// 基于当前值创建一个仅修改行号的新位置。
    ///
    /// 常用于“保留列偏好”的垂直移动场景。
    pub fn with_row(self, line: u32) -> Self {
        Self { line, ..self }
    }

    /// 基于当前值创建一个仅修改列号的新位置。
    pub fn with_col(self, column: u32) -> Self {
        Self { column, ..self }
    }
}

impl Default for Position {
    /// 默认位置是文档起点。
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    /// 创建位置并完成初始化。
    fn default_position_is_zero() {
        assert_eq!(Position::default(), Position::new(0, 0));
    }

    #[test]
    fn with_helpers_preserve_the_other_axis() {
        let position = Position::new(2, 4);

        assert_eq!(position.with_row(7), Position::new(7, 4));
        assert_eq!(position.with_col(9), Position::new(2, 9));
    }
}
