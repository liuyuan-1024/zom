//! 编辑器域组件聚合入口。

mod caret;
pub(crate) mod find_replace_bar;
mod layout_cache;
mod selection_paint;
mod view;
mod virtual_window;

pub(crate) use view::EditorView;
