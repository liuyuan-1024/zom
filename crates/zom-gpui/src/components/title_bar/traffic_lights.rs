//! macOS 原生红绿灯按钮组的布局模型。
//! 这里不负责绘制按钮，只负责统一描述其位置、占位和顶栏避让规则。

use gpui::{Pixels, Point, point, px};

use crate::theme::size::{self, SPACE_1, SPACE_2, SPACE_3};

/// 计算 macOS 红绿灯按钮的摆放位置。
pub(crate) fn position() -> Point<Pixels> {
    let layout = layout();

    // 根据设计系统推导垂直居中的 Y 坐标：
    let top_padding = SPACE_1;
    // GPUI 中 text_xs 默认行高约为 16.0
    let text_line_height = 16.0;
    // 胶囊自然撑开的高度：文字行高 + 上下 Padding (SPACE_1 * 2)
    let content_height = text_line_height + SPACE_1 * 2.0;

    // 在推导出的内容区内垂直居中
    let y_offset = top_padding + (content_height - layout.button_size) / 2.0;

    point(px(layout.leading_inset), px(y_offset))
}

/// 计算标题栏左侧正文需要避开红绿灯的水平缩进。
pub(super) fn title_bar_leading_inset() -> f32 {
    layout().slot_width()
}

/// 返回红绿灯整体布局规格。
fn layout() -> TrafficLightLayout {
    TrafficLightLayout {
        leading_inset: SPACE_2,
        button_size: size::ICON_MD,
        button_gap: SPACE_1,
        trailing_gap: SPACE_3,
    }
}

/// 表达 macOS 红绿灯按钮组整体布局的值对象。
struct TrafficLightLayout {
    leading_inset: f32,
    button_size: f32,
    button_gap: f32,
    trailing_gap: f32,
}

impl TrafficLightLayout {
    fn group_width(&self) -> f32 {
        self.button_size * 3.0 + self.button_gap * 2.0
    }

    fn slot_width(&self) -> f32 {
        self.leading_inset + self.group_width() + self.trailing_gap
    }
}
