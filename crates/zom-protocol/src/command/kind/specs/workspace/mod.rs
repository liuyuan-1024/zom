//! workspace 领域命令规范聚合模块。

// 顶层工作台动作（窗口、项目、保存、关闭）。
pub mod actions;
// 聚焦入口（编辑器/面板/浮层）。
pub mod pane;
pub mod panels;
pub mod overlays;
// 面板内域动作。
pub mod file_tree;
pub mod tab;
