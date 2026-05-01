//! 基于 Rope 的行索引与位置映射能力。

use ropey::Rope;
use zom_protocol::Position;
use zom_text_tokens::{CR_CHAR, LF_CHAR};

/// 行索引视图：封装 offset <-> (line, column) 映射与位置夹紧语义。
pub(crate) struct LineIndex<'a> {
    rope: &'a Rope,
}

impl<'a> LineIndex<'a> {
    /// 基于给定 rope 构建只读行索引视图。
    pub(crate) fn new(rope: &'a Rope) -> Self {
        Self { rope }
    }

    /// 将逻辑位置映射到字节偏移（越界时夹紧到文档边界）。
    ///
    /// 列语义按“可视列”计算：`\r` 不计列宽，`\n` 终止当前行。
    pub(crate) fn position_to_offset(&self, position: Position) -> usize {
        let target = self.clamp_position(position);
        let line_index = target.line as usize;
        let line_start_char = self.rope.line_to_char(line_index);
        let line = self.rope.line(line_index);

        let mut visual_column = 0u32;
        let mut relative_char_index = 0usize;
        for ch in line.chars() {
            if ch == LF_CHAR || visual_column == target.column {
                break;
            }
            if ch != CR_CHAR {
                visual_column = visual_column.saturating_add(1);
            }
            relative_char_index += 1;
        }

        self.rope
            .char_to_byte(line_start_char + relative_char_index)
    }

    /// 将字节偏移映射到行列坐标，越界时返回 `None`。
    ///
    /// 仅当偏移不超过文本末尾才可映射；列统计同样忽略 `\r`。
    pub(crate) fn offset_to_position(&self, offset: usize) -> Option<Position> {
        if offset > self.rope.len_bytes() {
            return None;
        }
        let char_index = self.rope.byte_to_char(offset);
        let line_index = self.rope.char_to_line(char_index);
        let line_start_char = self.rope.line_to_char(line_index);
        let mut column = 0u32;
        for ch in self.rope.slice(line_start_char..char_index).chars() {
            if ch != CR_CHAR {
                column = column.saturating_add(1);
            }
        }
        Some(Position::new(
            u32::try_from(line_index).unwrap_or(u32::MAX),
            column,
        ))
    }

    /// 文档总行数（最少为 1）。
    ///
    /// 与大多数编辑器一致，空文本视为单行空行。
    pub(crate) fn line_count(&self) -> u32 {
        u32::try_from(self.rope.len_lines()).unwrap_or(u32::MAX)
    }

    /// 指定行的可视列宽（忽略 `\r`，不计入换行符）。
    pub(crate) fn line_len(&self, line: u32) -> u32 {
        let line_index = line as usize;
        if line_index >= self.rope.len_lines() {
            return 0;
        }
        let mut column = 0u32;
        for ch in self.rope.line(line_index).chars() {
            match ch {
                LF_CHAR => break,
                CR_CHAR => {}
                _ => column = column.saturating_add(1),
            }
        }
        column
    }

    /// 将位置夹紧到当前文档范围。
    ///
    /// 先夹行再夹列，避免把列夹到错误行宽。
    pub(crate) fn clamp_position(&self, position: Position) -> Position {
        let line = position.line.min(self.line_count().saturating_sub(1));
        let column = position.column.min(self.line_len(line));
        Position::new(line, column)
    }
}
