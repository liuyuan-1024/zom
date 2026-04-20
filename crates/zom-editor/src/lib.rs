//! `zom-editor` 的公共入口。
//! 负责承载编辑领域行为与文本视图语义。

mod buffer;
mod viewer_layout;

pub use buffer::EditorBuffer;
pub use viewer_layout::wrap_visual_line;
