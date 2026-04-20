//! 主题透明度 token 定义。
//!
//! 设计原则：
//! 1. 透明度属于「状态语义」，不归到 size（尺寸）中。
//! 2. 组件不直接写 0.0 / 1.0，统一走语义 token。

/// 完全不可见（常用于 hover 前隐藏辅助控件）。
pub(crate) const OPACITY_HIDDEN: f32 = 0.0;
/// 完全可见（常用于 hover 后显示控件）。
pub(crate) const OPACITY_VISIBLE: f32 = 1.0;
