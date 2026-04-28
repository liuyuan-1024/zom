//! 主题颜色 token 定义。

// --- 容器与表面 (Surfaces & Backgrounds) ---
// 逻辑：越往上的层级（浮层 > 面板 > 底色），颜色越亮。
/// 基础底色 (用于编辑器核心区、工具栏背景)。
pub(crate) const COLOR_BG_APP: u32 = 0x0D1117;
/// 面板色 (用于文件树、顶栏 TitleBar、浮层 Tooltip)。
pub(crate) const COLOR_BG_PANEL: u32 = 0x161B22;

// --- 交互元素与状态 (Elements & States) ---
// 逻辑：统一所有组件的常态、悬停、激活表现。
/// 元素底色 (用于胶囊常态、未激活的 Tab)。
pub(crate) const COLOR_BG_ELEMENT: u32 = 0x21262D;
/// 统一悬停态 (Hover)。
pub(crate) const COLOR_BG_HOVER: u32 = 0x30363D;
/// 激活/选中态 (用于当前 Tab、文件树选中行)。
pub(crate) const COLOR_BG_ACTIVE: u32 = 0x2A405A;

// --- 边框 (Borders) ---
// 逻辑：全系统只需一种通用分割线颜色即可，靠环境对比度自然显现。
/// 统一边框和分割线色。
pub(crate) const COLOR_BORDER: u32 = 0x30363D;

// --- 前景色 (Foregrounds: Text & Icons) ---
// 逻辑：不再区分文字和图标，它们都是用来传递信息的“墨水”。
/// 主内容 (正文内容、激活的 Tab 标题、高亮图标)。
pub(crate) const COLOR_FG_PRIMARY: u32 = 0xE6EDF3;
/// 次要内容 (辅助文本、快捷键、常规图标、行号)。
pub(crate) const COLOR_FG_MUTED: u32 = 0x8D96A0;
