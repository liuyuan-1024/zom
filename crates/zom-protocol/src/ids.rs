//! 跨模块共享的强类型 ID 定义。

use std::fmt;

/// 定义共享 ID 新类型，避免业务层到处传裸 `u64`。
macro_rules! define_id {
    ($name:ident, $doc:literal) => {
        /// 强类型 ID 包装，避免业务层直接传递裸 `u64`。
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub u64);

        impl $name {
            /// 用底层整数值构造一个强类型 ID。
            ///
            /// 不在协议层做“是否已注册/是否存在”校验，生命周期约束由上层负责。
            pub fn new(value: u64) -> Self {
                Self(value)
            }

            /// 取出底层整数值（用于序列化、日志和跨边界传输）。
            pub fn value(self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $name {
            fn from(value: u64) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(BufferId, "文本缓冲区的标识符。");
define_id!(PaneId, "面板的标识符。");
define_id!(WorkspaceId, "工作区的标识符。");

#[cfg(test)]
mod tests {
    use super::{BufferId, PaneId, WorkspaceId};

    #[test]
    fn ids_expose_their_underlying_value() {
        assert_eq!(BufferId::new(7).value(), 7);
        assert_eq!(PaneId::from(9).to_string(), "9");
        assert_eq!(WorkspaceId::new(11).value(), 11);
    }
}
