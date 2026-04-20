//! 应用层投影视图：把统一命令语义投影成 UI 可消费的信息。

mod shortcuts;
mod text;

pub use shortcuts::shortcut_hint;
pub use text::wrap_visual_line;
