//! 文件树组件模块入口。

use crate::theme::size::SPACE_2;

/// 文件树层级缩进的步长。
pub(super) const FILE_TREE_INDENT_STEP: f32 = SPACE_2;

mod row;
mod view;

pub(crate) use view::{FileTreeNodeClicked, FileTreePanel};
