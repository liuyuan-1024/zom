//! Chip 组件聚合入口。
//! 这里统一承载文本胶囊、图标胶囊、状态胶囊以及胶囊级 tooltip 能力。

mod tooltip;
mod view;

pub(crate) use view::{Chip, ChipStyle};
