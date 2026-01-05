/// Source position in original Oxur code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePos {
    /// Source file path (or "\<repl\>" for REPL input)
    pub file: String,

    /// 1-indexed line number
    pub line: u32,

    /// 1-indexed column number
    pub column: u32,

    /// Length of the span (for error highlighting)
    pub length: u32,
}

impl SourcePos {
    /// Create a new source position
    pub fn new(file: String, line: u32, column: u32, length: u32) -> Self {
        assert!(line > 0, "Line numbers are 1-indexed");
        assert!(column > 0, "Column numbers are 1-indexed");
        Self { file, line, column, length }
    }

    /// Create a position for REPL input
    pub fn repl(line: u32, column: u32, length: u32) -> Self {
        Self::new("<repl>".to_string(), line, column, length)
    }

    /// Get end column (column + length)
    pub fn end_column(&self) -> u32 {
        self.column + self.length
    }

    /// Check if this position contains another position
    pub fn contains(&self, other: &SourcePos) -> bool {
        self.file == other.file
            && self.line == other.line
            && other.column >= self.column
            && other.column < self.end_column()
    }
}

impl std::fmt::Display for SourcePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_pos_basic() {
        let pos = SourcePos::new("test.oxur".to_string(), 1, 5, 10);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.length, 10);
        assert_eq!(pos.end_column(), 15);
    }

    #[test]
    fn test_source_pos_display() {
        let pos = SourcePos::new("test.oxur".to_string(), 10, 20, 5);
        assert_eq!(format!("{}", pos), "test.oxur:10:20");
    }

    #[test]
    fn test_source_pos_repl() {
        let pos = SourcePos::repl(1, 1, 20);
        assert_eq!(pos.file, "<repl>");
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.length, 20);
    }

    #[test]
    fn test_source_pos_contains() {
        let span = SourcePos::new("test.oxur".to_string(), 1, 5, 10);
        let inner = SourcePos::new("test.oxur".to_string(), 1, 7, 3);
        let outer = SourcePos::new("test.oxur".to_string(), 1, 3, 2);

        assert!(span.contains(&inner));
        assert!(!span.contains(&outer));
    }

    #[test]
    fn test_source_pos_contains_different_files() {
        let span1 = SourcePos::new("file1.oxur".to_string(), 1, 5, 10);
        let span2 = SourcePos::new("file2.oxur".to_string(), 1, 7, 3);

        assert!(!span1.contains(&span2));
    }

    #[test]
    fn test_source_pos_contains_different_lines() {
        let span1 = SourcePos::new("test.oxur".to_string(), 1, 5, 10);
        let span2 = SourcePos::new("test.oxur".to_string(), 2, 7, 3);

        assert!(!span1.contains(&span2));
    }

    #[test]
    #[should_panic(expected = "Line numbers are 1-indexed")]
    fn test_source_pos_zero_line() {
        SourcePos::new("test.oxur".to_string(), 0, 1, 1);
    }

    #[test]
    #[should_panic(expected = "Column numbers are 1-indexed")]
    fn test_source_pos_zero_column() {
        SourcePos::new("test.oxur".to_string(), 1, 0, 1);
    }
}
