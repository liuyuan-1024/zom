//! macOS 原生红绿灯按钮组的布局模型。
//! 这里不负责绘制按钮，只负责统一描述其位置、占位和顶栏避让规则。

use gpui::{Pixels, Point, point, px};

use crate::spacing::{SPACE_1, SPACE_2, SPACE_3};

/// macOS 红绿灯按钮的视觉直径。
const TRAFFIC_LIGHT_SIZE: f32 = 12.0;
/// 红绿灯按钮组之间的固定间距。
const TRAFFIC_LIGHT_INTERNAL_GAP: f32 = SPACE_1;
/// 红绿灯的左侧偏移量。
const TRAFFIC_LIGHT_LEADING_INSET: f32 = SPACE_2;
/// 按钮组结束后到正文安全区域之间的固定留白。
const TRAFFIC_LIGHT_TRAILING_GAP: f32 = SPACE_3;

/// 计算 macOS 红绿灯按钮的摆放位置。
pub(super) fn traffic_light_position() -> Point<Pixels> {
    let layout = traffic_light_layout();

    // 根据设计系统推导垂直居中的 Y 坐标：
    // 1. 获取顶栏的顶部内边距
    let top_padding = SPACE_1;
    // 2. 核心内容区（Chip 胶囊）的标准高度
    let content_height = 24.0;
    // 3. 在内容区内垂直居中
    let y_offset = top_padding + (content_height - layout.button_size) / 2.0;

    point(px(layout.leading_inset), px(y_offset))
}

/// 计算标题栏左侧正文需要避开红绿灯的水平缩进。
pub(super) fn title_bar_leading_inset() -> f32 {
    traffic_light_layout().slot_width()
}

/// 返回红绿灯整体布局规格。
fn traffic_light_layout() -> TrafficLightLayout {
    TrafficLightLayout {
        leading_inset: TRAFFIC_LIGHT_LEADING_INSET,
        button_size: TRAFFIC_LIGHT_SIZE,
        button_gap: TRAFFIC_LIGHT_INTERNAL_GAP,
        trailing_gap: TRAFFIC_LIGHT_TRAILING_GAP,
    }
}

/// 表达 macOS 红绿灯按钮组整体布局的值对象。
/// 所有与顶栏左侧避让有关的计算，都应从这个对象推导，而不是分散拼装。
struct TrafficLightLayout {
    /// 按钮组距离窗口左边界的起始偏移。
    leading_inset: f32,
    /// 单个按钮的直径。
    button_size: f32,
    /// 相邻按钮之间的固定间距。
    button_gap: f32,
    /// 按钮组结束后到正文安全区域之间的额外留白。
    trailing_gap: f32,
}

impl TrafficLightLayout {
    /// 计算三颗红绿灯按钮自身占据的总宽度。
    fn group_width(&self) -> f32 {
        self.button_size * 3.0 + self.button_gap * 2.0
    }

    /// 计算顶栏正文应整体避让的总宽度。
    /// 这个值包含左侧起始偏移、按钮组宽度以及按钮组后的安全留白。
    fn slot_width(&self) -> f32 {
        self.leading_inset + self.group_width() + self.trailing_gap
    }
}
