//! 文件树组件模块入口。

use crate::spacing::SPACE_2;

/// 文件树层级缩进的步长。
/// 这里采用 2 倍基础节奏，避免层级展开过快导致横向空间被过早吃掉。
pub(super) const FILE_TREE_INDENT_STEP: f32 = SPACE_2;
/// 文件树层级导线的颜色。
pub(super) const FILE_TREE_GUIDE_COLOR: u32 = 0x2a3242;

mod row;
mod view;

pub(crate) use view::FileTreePanel;
