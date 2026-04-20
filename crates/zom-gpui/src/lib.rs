//! `zom-gpui` 负责把应用状态渲染成桌面界面。
//! 当前阶段先提供一个最小可运行的编辑器壳子。

mod assets;
mod components;
mod input;
mod root_view;
mod theme;

pub use root_view::run;
