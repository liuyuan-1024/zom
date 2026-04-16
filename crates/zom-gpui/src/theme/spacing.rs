//! 全局 spacing token。
//! 所有常规的 gap、padding、margin 都应优先从这里取值，而不是直接写裸数字。

/// 整个界面的基础节奏单位。
pub(crate) const GAP: f32 = 6.0;

// 基础倍率
pub(crate) const SPACE_1: f32 = GAP;
pub(crate) const SPACE_2: f32 = GAP * 2.0;
pub(crate) const SPACE_3: f32 = GAP * 3.0;
pub(crate) const SPACE_4: f32 = GAP * 4.0;
pub(crate) const SPACE_5: f32 = GAP * 5.0;

// --- 窗口与布局基础 (Layout & Windows) ---
pub(crate) const WINDOW_DEFAULT_WIDTH: f32 = 900.0;
pub(crate) const WINDOW_DEFAULT_HEIGHT: f32 = 900.0;
pub(crate) const PANEL_DEFAULT_WIDTH: f32 = 260.0;
pub(crate) const PANEL_MIN_WIDTH: f32 = 160.0;
pub(crate) const PANEL_MAX_WIDTH: f32 = 600.0;

// --- 组件统一尺寸 (Components) ---
pub(crate) const TRAFFIC_LIGHT_SIZE: f32 = 12.0; // macOS 红绿灯尺寸
pub(crate) const LINE_NUMBER_WIDTH: f32 = 40.0; // 编辑器行号列宽度

// --- 图标与控件尺寸 (Icons & Controls) ---
pub(crate) const ICON_SIZE_SM: f32 = 12.0;
pub(crate) const ICON_SIZE_MD: f32 = 15.0; // 标题栏/工具栏通用图标
pub(crate) const BTN_CLOSE_SIZE: f32 = 16.0; // Tab 关闭按钮悬浮区尺寸
pub(crate) const BTN_CLOSE_ICON_SIZE: f32 = 10.0; // Tab 关闭按钮实际图标尺寸
