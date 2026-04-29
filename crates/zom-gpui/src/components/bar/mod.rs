//! window chrome 的 bar 布局原语。
//! 集中维护顶栏、底栏共享的节奏、尺寸和容器样式。
//! 负责汇总 bar 布局与原生红绿灯布局能力，并对外暴露稳定入口。

mod bar_shell;
pub(crate) mod status_bar;
pub(crate) mod title_bar;
pub(crate) mod traffic_lights;
