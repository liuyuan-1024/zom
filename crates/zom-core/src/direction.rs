//! 通用方向类型。
//! 这些抽象会在移动、导航、布局等场景中复用。

/// 表示一维上的前进或后退方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// 向前。
    Forward,
    /// 向后。
    Backward,
}

/// 表示二维空间中的坐标轴。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    /// 水平方向。
    Horizontal,
    /// 垂直方向。
    Vertical,
}
