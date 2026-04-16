//! window chrome 模块入口。
//! 这里负责汇总 bar 布局与原生红绿灯布局能力，并对外暴露稳定入口。

mod bar;

pub(crate) use bar::{bar, group};
