#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub row: u32,
    pub col: u32,
}

impl Position {
    pub fn new(row: u32, col: u32) -> Self {
        Self { row, col }
    }
}
