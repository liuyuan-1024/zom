//! 文件树面板模块聚合入口。

//! 文件树组件模块入口。

use crate::theme::size::GAP_2;

/// 文件树层级缩进的步长。
pub(super) const FILE_TREE_INDENT_STEP: f32 = GAP_2;

mod row;
mod view;

pub(crate) use view::FileTreePanel;
