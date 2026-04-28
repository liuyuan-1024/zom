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
    /// 混合换行风格。
    Mixed,
}

impl LineEnding {
    /// 返回换行符文本。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => LF,
            Self::Crlf => CRLF,
            Self::Cr => CR,
            Self::Mixed => LF,
        }
    }

    /// 返回换行风格标签。
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
    /// 使用指定数量的空格缩进。
    Spaces(u8),
}

impl IndentUnit {
    /// 返回缩进文本。
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
    fn indent_unit_builds_tab_and_spaces() {
        assert_eq!(IndentUnit::Tab.as_string(), "\t");
        assert_eq!(IndentUnit::Spaces(4).as_string(), "    ");
    }
}
