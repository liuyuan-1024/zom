//! window chrome 模块入口。
//! 这里负责汇总 bar 布局与原生红绿灯布局能力，并对外暴露稳定入口。

mod bar;
mod traffic_lights;

pub(crate) use bar::{bar, chrome_padding, group, titlebar_icon_size, tool_icon_size};

use gpui::{Pixels, Point};

/// 计算 macOS 红绿灯按钮的摆放位置。
pub(crate) fn traffic_light_position() -> Point<Pixels> {
    traffic_lights::traffic_light_position()
}

/// 计算标题栏左侧正文需要避开红绿灯的水平缩进。
pub(crate) fn title_bar_leading_inset() -> f32 {
    traffic_lights::title_bar_leading_inset()
}
