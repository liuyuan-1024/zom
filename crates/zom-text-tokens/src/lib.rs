//! 文本控制符与缩进/换行语义的统一定义。

/// 换行符（LF）。
pub const LF: &str = "\n";
/// 回车符（CR）。
pub const CR: &str = "\r";
/// 制表符（Tab）。
pub const TAB: &str = "\t";
/// 回车换行符（CRLF）。
pub const CRLF: &str = "\r\n";

/// LF 字符。
pub const LF_CHAR: char = '\n';
/// CR 字符。
pub const CR_CHAR: char = '\r';
/// Tab 字符。
pub const TAB_CHAR: char = '\t';

/// LF 字节。
pub const LF_BYTE: u8 = b'\n';
/// CR 字节。
pub const CR_BYTE: u8 = b'\r';
/// Tab 字节。
pub const TAB_BYTE: u8 = b'\t';

/// 文本换行风格。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineEnding {
    /// `\n`
    Lf,
    /// `\r\n`
    Crlf,
    /// `\r`
    Cr,
    /// 混合换行风格（同一文本内出现多种换行符）。
    Mixed,
}

impl LineEnding {
    /// 返回对应换行文本。
    ///
    /// `Mixed` 不存在单一可逆表示，这里约定回退 `LF` 作为写回默认值。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => LF,
            Self::Crlf => CRLF,
            Self::Cr => CR,
            Self::Mixed => LF,
        }
    }

    /// 返回稳定标签（用于状态栏/诊断展示）。
    pub const fn label(self) -> &'static str {
        match self {
            Self::Lf => "LF",
            Self::Crlf => "CRLF",
            Self::Cr => "CR",
            Self::Mixed => "Mixed",
        }
    }
}

/// 缩进单元定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IndentUnit {
    /// 使用制表符缩进。
    #[default]
    Tab,
    /// 使用指定数量的空格缩进（宽度由调用方策略决定）。
    Spaces(u8),
}

impl IndentUnit {
    /// 返回缩进文本。
    ///
    /// 该函数不做“宽度是否合理”校验，例如 `Spaces(0)` 会返回空串。
    pub fn as_string(self) -> String {
        match self {
            Self::Tab => TAB.to_string(),
            Self::Spaces(width) => " ".repeat(usize::from(width)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IndentUnit, LineEnding};

    #[test]
    fn line_ending_exposes_stable_label_and_text() {
        assert_eq!(LineEnding::Lf.label(), "LF");
        assert_eq!(LineEnding::Lf.as_str(), "\n");
        assert_eq!(LineEnding::Crlf.label(), "CRLF");
        assert_eq!(LineEnding::Crlf.as_str(), "\r\n");
        assert_eq!(LineEnding::Mixed.label(), "Mixed");
        assert_eq!(LineEnding::Mixed.as_str(), "\n");
    }

    #[test]
    /// 计算标签页结果。
    fn indent_unit_builds_tab_and_spaces() {
        assert_eq!(IndentUnit::Tab.as_string(), "\t");
        assert_eq!(IndentUnit::Spaces(4).as_string(), "    ");
    }
}
