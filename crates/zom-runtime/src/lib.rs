//! `zom-runtime` 负责应用层编排。
//! 当前阶段先提供桌面界面所需的静态应用状态，后续再接命令分发和服务注入。

mod buffer_preview;
mod initial_state;
pub mod projection;
pub mod state;
mod workspace_paths;
