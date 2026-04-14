//! 顶栏与底栏共享的视觉节奏定义。
//! 这里集中维护公共间距、胶囊控件和 macOS 红绿灯位置计算。

use gpui::{Div, Pixels, Point, div, point, prelude::*, px, rgb};

/// macOS 红绿灯按钮的视觉间距基准。
/// 顶栏、底栏以及标题避让都以这个值作为统一的节奏单位。
const MAC_SPACE: f32 = 6.0;
/// 顶栏和底栏共用的内边距，统一控制四周留白。
const CHROME_PADDING: f32 = MAC_SPACE;
/// 顶栏和底栏内部元素共用的间距。
const CHROME_GAP: f32 = MAC_SPACE;
/// 顶栏和底栏内部胶囊按钮的统一高度。
const CHROME_ITEM_HEIGHT: f32 = 24.0;
/// 顶栏和底栏的总高度，由统一的内边距和内容高度推导得出。
const CHROME_BAR_HEIGHT: f32 = CHROME_ITEM_HEIGHT + CHROME_PADDING * 2.0;
/// macOS 红绿灯按钮的视觉直径。
const TRAFFIC_LIGHT_SIZE: f32 = 12.0;
/// 红绿灯按钮组之间的固定间距。
const TRAFFIC_LIGHT_INTERNAL_GAP: f32 = MAC_SPACE;
/// 顶栏左侧为红绿灯预留的固定前导区域。
/// 这比单纯按按钮几何宽度估算更接近 Zed 的标题栏处理方式：
/// 把红绿灯当成一个完整的 leading slot，而不是让标题内容贴着按钮组起算。
const TRAFFIC_LIGHT_LEADING_SLOT_UNITS: f32 = 4.0;
/// 顶栏图标使用的尺寸。
const TITLEBAR_ICON_SIZE: f32 = 14.0;
/// 状态栏图标使用的尺寸。
const STATUS_ICON_SIZE: f32 = 13.0;

/// 返回顶栏和底栏通用的容器样式。
pub(crate) fn bar() -> Div {
    div()
        .w_full()
        .h(px(CHROME_BAR_HEIGHT))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .p(px(CHROME_PADDING))
}

/// 返回顶栏和底栏通用的胶囊控件样式。
pub(crate) fn chip() -> Div {
    div()
        .h(px(CHROME_ITEM_HEIGHT))
        .px(px(CHROME_PADDING))
        .flex()
        .flex_row()
        .items_center()
        .bg(rgb(0x0f1319))
        .border_1()
        .border_color(rgb(0x2b3444))
        .rounded_sm()
}

/// 返回顶栏和底栏通用的纯图标按钮样式。
pub(crate) fn icon_chip() -> Div {
    div()
        .w(px(CHROME_ITEM_HEIGHT))
        .h(px(CHROME_ITEM_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .rounded_sm()
}

/// 返回顶栏和底栏通用的水平分组样式。
pub(crate) fn group() -> Div {
    div().flex().flex_row().items_center().gap(px(CHROME_GAP))
}

/// 返回顶栏图标的统一尺寸。
pub(crate) fn titlebar_icon_size() -> f32 {
    TITLEBAR_ICON_SIZE
}

/// 返回状态栏图标的统一尺寸。
pub(crate) fn status_icon_size() -> f32 {
    STATUS_ICON_SIZE
}

/// 计算 macOS 红绿灯按钮的摆放位置。
pub(crate) fn traffic_light_position() -> Point<Pixels> {
    point(
        px(CHROME_PADDING),
        px((CHROME_BAR_HEIGHT - TRAFFIC_LIGHT_SIZE) / 2.0),
    )
}

/// 计算标题栏左侧正文需要避开红绿灯的水平缩进。
pub(crate) fn title_bar_leading_inset() -> f32 {
    traffic_light_group_width() + MAC_SPACE * TRAFFIC_LIGHT_LEADING_SLOT_UNITS
}

/// 估算红绿灯三按钮占据的总宽度。
fn traffic_light_group_width() -> f32 {
    TRAFFIC_LIGHT_SIZE * 3.0 + TRAFFIC_LIGHT_INTERNAL_GAP * 2.0
}
