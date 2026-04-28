//! `zom-runtime` 负责应用层编排。
//! 当前已承接桌面状态机、命令分发与 UI 副作用编排。

mod buffer_preview;
mod draft_store;
mod initial_state;
pub mod projection;
pub mod state;
mod workspace_paths;
