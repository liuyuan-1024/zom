//! 极简且语义化的尺寸主题 Tokens

// --- 间距与网格 (Spacing & Grid) ---
// 逻辑：以 6px 为基础网格，提供线性递增的间距单位，用于 padding, margin, gap
pub(crate) const GAP_0_5: f32 = GAP_1 * 0.5; // 3.0px (一般是绝对定位保持居中的计算需要)
pub(crate) const GAP_1: f32 = 6.0; // 6.0px (极小间距/微调)
pub(crate) const GAP_1_5: f32 = GAP_1 * 1.5; // 9.0px (紧凑图标簇间距)
pub(crate) const GAP_2: f32 = GAP_1 * 2.0; // 12.0px (组件内容间距)
pub(crate) const GAP_3: f32 = GAP_1 * 3.0; // 18.0px (段落/区块间距)

pub(crate) const PADDING_SM: f32 = 8.0; // 小尺寸内边距 (如只读查看区内边距)

// --- 字体尺寸 (Font Sizes) ---
// 逻辑：统一不同场景的字体物理大小，不再根据“它是哪个组件”来命名
pub(crate) const FONT_MD: f32 = 16.0; // 核心字体 (如正文、标题、工具栏文本)

// --- 图标尺寸 (Icon Sizes) ---
// 逻辑：统一不同场景的图标物理大小，不再根据“它是哪个组件”来命名
pub(crate) const ICON_SM: f32 = 12.0; // 次要图标 (如红绿灯、文件树辅助图标、Tab 的关闭叉号)
pub(crate) const ICON_MD: f32 = 16.0; // 核心图标 (如顶栏、工具栏、文件树主体图标)

// --- 交互控件热区 (Control Areas) ---
// 逻辑：图标可能很小，但为了符合 Fitts's Law (方便点击)，悬浮和点击的热区需要标准化
pub(crate) const CONTROL_XS: f32 = 16.0; // 微型按钮热区 (如 Tab 关闭按钮的容器)
pub(crate) const GUTTER_MD: f32 = 40.0; // 中尺寸行号/序号栏宽度

// --- 宏观布局与面板 (Macro Layout) ---
// 逻辑：定义应用级的骨架尺寸界限
pub(crate) const WINDOW_WIDTH: f32 = 850.0;
pub(crate) const WINDOW_HEIGHT: f32 = 950.0;
pub(crate) const PANEL_WIDTH: f32 = 250.0;

// 拖拽遮罩，TODO 后续我想优化，不再用这种暴力遮罩的方法
pub(crate) const DRAG_CAPTURE_OFFSET: f32 = 2000.0;
pub(crate) const DRAG_CAPTURE_SPAN: f32 = DRAG_CAPTURE_OFFSET * 5.0;
