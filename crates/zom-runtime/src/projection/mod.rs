//! 应用层投影视图：把统一命令语义投影成 UI 可消费的信息。

mod command;
mod shortcuts;
mod status;
mod text;

pub use command::{command_dock, command_is_active, panel_target_for_command};
pub use shortcuts::shortcut_hint;
pub use status::cursor_text;
pub use text::wrap_visual_line;
