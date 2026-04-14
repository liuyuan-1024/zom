//! 全局 spacing token。
//! 所有常规的 gap、padding、margin 都应优先从这里取值，而不是直接写裸数字。

/// 整个界面的基础节奏单位。
pub(crate) const GAP: f32 = 6.0;
/// 1 倍基础节奏。
pub(crate) const SPACE_1: f32 = GAP;
/// 2 倍基础节奏。
pub(crate) const SPACE_2: f32 = GAP * 2.0;
/// 3 倍基础节奏。
pub(crate) const SPACE_3: f32 = GAP * 3.0;
/// 4 倍基础节奏。
pub(crate) const SPACE_4: f32 = GAP * 4.0;
/// 5 倍基础节奏。
pub(crate) const SPACE_5: f32 = GAP * 5.0;
