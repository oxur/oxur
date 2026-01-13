/// A span representing a range in source code
///
/// Unlike `SourcePos` which represents a point with a length, `Span`
/// explicitly represents a range with start and end positions. This
/// is particularly useful for multi-line constructs like functions,
/// blocks, and expressions that span multiple lines.
///
/// # Examples
///
/// ```
/// use oxur_smap::Span;
///
/// // Single-line span
/// let span = Span::new(
///     "test.oxur".to_string(),
///     1, 5,   // start: line 1, column 5
///     1, 15,  // end: line 1, column 15
/// );
/// assert_eq!(span.num_lines(), 1);
/// assert_eq!(span.length_on_start_line(), 10);
///
/// // Multi-line span
/// let span = Span::new(
///     "test.oxur".to_string(),
///     1, 5,   // start: line 1, column 5
///     3, 10,  // end: line 3, column 10
/// );
/// assert_eq!(span.num_lines(), 3);
/// assert!(span.is_multi_line());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Source file path (or `<repl>` for REPL input)
    pub file: String,

    /// Start line number (1-indexed)
    pub start_line: u32,

    /// Start column number (1-indexed)
    pub start_column: u32,

    /// End line number (1-indexed)
    pub end_line: u32,

    /// End column number (1-indexed, exclusive)
    ///
    /// Note: This is the column *after* the last character in the span.
    /// For example, if the span covers "hello", end_column points to
    /// the position after 'o'.
    pub end_column: u32,
}

impl Span {
    /// Create a new span
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - `start_line` or `start_column` is 0 (must be 1-indexed)
    /// - `end_line` or `end_column` is 0 (must be 1-indexed)
    /// - `end_line` < `start_line` (invalid range)
    /// - `end_line` == `start_line` and `end_column` <= `start_column` (invalid range)
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span = Span::new("file.oxur".to_string(), 1, 5, 1, 15);
    /// assert_eq!(span.start_line, 1);
    /// assert_eq!(span.end_column, 15);
    /// ```
    pub fn new(
        file: String,
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
    ) -> Self {
        assert!(start_line > 0, "Line numbers are 1-indexed");
        assert!(start_column > 0, "Column numbers are 1-indexed");
        assert!(end_line > 0, "Line numbers are 1-indexed");
        assert!(end_column > 0, "Column numbers are 1-indexed");
        assert!(end_line >= start_line, "End line must be >= start line");
        if end_line == start_line {
            assert!(end_column > start_column, "End column must be > start column on same line");
        }

        Self { file, start_line, start_column, end_line, end_column }
    }

    /// Create a span for REPL input
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span = Span::repl(1, 1, 1, 10);
    /// assert_eq!(span.file, "<repl>");
    /// ```
    pub fn repl(start_line: u32, start_column: u32, end_line: u32, end_column: u32) -> Self {
        Self::new("<repl>".to_string(), start_line, start_column, end_line, end_column)
    }

    /// Check if this span is on a single line
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let single = Span::new("file.oxur".to_string(), 1, 5, 1, 15);
    /// assert!(single.is_single_line());
    ///
    /// let multi = Span::new("file.oxur".to_string(), 1, 5, 3, 10);
    /// assert!(!multi.is_single_line());
    /// ```
    pub fn is_single_line(&self) -> bool {
        self.start_line == self.end_line
    }

    /// Check if this span spans multiple lines
    pub fn is_multi_line(&self) -> bool {
        !self.is_single_line()
    }

    /// Get the number of lines this span covers
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span = Span::new("file.oxur".to_string(), 1, 5, 3, 10);
    /// assert_eq!(span.num_lines(), 3);
    /// ```
    pub fn num_lines(&self) -> u32 {
        self.end_line - self.start_line + 1
    }

    /// Get the length on the start line (for single-line spans)
    ///
    /// For multi-line spans, this returns the length from start_column
    /// to the end of the start line (which is undefined without source text).
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span = Span::new("file.oxur".to_string(), 1, 5, 1, 15);
    /// assert_eq!(span.length_on_start_line(), 10);
    /// ```
    pub fn length_on_start_line(&self) -> u32 {
        if self.is_single_line() {
            self.end_column - self.start_column
        } else {
            // For multi-line, we don't know the line length without source text
            // This is a limitation - caller should check is_single_line() first
            0
        }
    }

    /// Check if this span contains another span
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let outer = Span::new("file.oxur".to_string(), 1, 5, 3, 20);
    /// let inner = Span::new("file.oxur".to_string(), 2, 1, 2, 10);
    ///
    /// assert!(outer.contains(&inner));
    /// assert!(!inner.contains(&outer));
    /// ```
    pub fn contains(&self, other: &Span) -> bool {
        if self.file != other.file {
            return false;
        }

        // Check start is before or equal
        let start_ok = other.start_line > self.start_line
            || (other.start_line == self.start_line && other.start_column >= self.start_column);

        // Check end is after or equal
        let end_ok = other.end_line < self.end_line
            || (other.end_line == self.end_line && other.end_column <= self.end_column);

        start_ok && end_ok
    }

    /// Merge two spans into a single span covering both
    ///
    /// # Panics
    ///
    /// Panics if the spans are from different files.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxur_smap::Span;
    ///
    /// let span1 = Span::new("file.oxur".to_string(), 1, 5, 1, 10);
    /// let span2 = Span::new("file.oxur".to_string(), 1, 15, 2, 5);
    /// let merged = span1.merge(&span2);
    ///
    /// assert_eq!(merged.start_line, 1);
    /// assert_eq!(merged.start_column, 5);
    /// assert_eq!(merged.end_line, 2);
    /// assert_eq!(merged.end_column, 5);
    /// ```
    pub fn merge(&self, other: &Span) -> Span {
        assert_eq!(self.file, other.file, "Cannot merge spans from different files");

        let (start_line, start_column) = if self.start_line < other.start_line
            || (self.start_line == other.start_line && self.start_column < other.start_column)
        {
            (self.start_line, self.start_column)
        } else {
            (other.start_line, other.start_column)
        };

        let (end_line, end_column) = if self.end_line > other.end_line
            || (self.end_line == other.end_line && self.end_column > other.end_column)
        {
            (self.end_line, self.end_column)
        } else {
            (other.end_line, other.end_column)
        };

        Span::new(self.file.clone(), start_line, start_column, end_line, end_column)
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_single_line() {
            write!(f, "{}:{}:{}-{}", self.file, self.start_line, self.start_column, self.end_column)
        } else {
            write!(
                f,
                "{}:{}:{}-{}:{}",
                self.file, self.start_line, self.start_column, self.end_line, self.end_column
            )
        }
    }
}

/// Convert a single-line Span to SourcePos
///
/// # Panics
///
/// Panics if the span is multi-line.
impl From<&Span> for crate::SourcePos {
    fn from(span: &Span) -> Self {
        assert!(span.is_single_line(), "Can only convert single-line Span to SourcePos");
        crate::SourcePos::new(
            span.file.clone(),
            span.start_line,
            span.start_column,
            span.length_on_start_line(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_basic() {
        let span = Span::new("test.oxur".to_string(), 1, 5, 1, 15);
        assert_eq!(span.file, "test.oxur");
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_column, 5);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_column, 15);
    }

    #[test]
    fn test_span_display_single_line() {
        let span = Span::new("test.oxur".to_string(), 10, 5, 10, 15);
        assert_eq!(format!("{}", span), "test.oxur:10:5-15");
    }

    #[test]
    fn test_span_display_multi_line() {
        let span = Span::new("test.oxur".to_string(), 1, 5, 3, 10);
        assert_eq!(format!("{}", span), "test.oxur:1:5-3:10");
    }

    #[test]
    fn test_span_repl() {
        let span = Span::repl(1, 1, 1, 20);
        assert_eq!(span.file, "<repl>");
        assert_eq!(span.start_line, 1);
    }

    #[test]
    fn test_span_is_single_line() {
        let single = Span::new("test.oxur".to_string(), 1, 5, 1, 15);
        assert!(single.is_single_line());
        assert!(!single.is_multi_line());

        let multi = Span::new("test.oxur".to_string(), 1, 5, 3, 10);
        assert!(!multi.is_single_line());
        assert!(multi.is_multi_line());
    }

    #[test]
    fn test_span_num_lines() {
        let span1 = Span::new("test.oxur".to_string(), 1, 5, 1, 15);
        assert_eq!(span1.num_lines(), 1);

        let span2 = Span::new("test.oxur".to_string(), 1, 5, 3, 10);
        assert_eq!(span2.num_lines(), 3);

        let span3 = Span::new("test.oxur".to_string(), 5, 1, 10, 1);
        assert_eq!(span3.num_lines(), 6);
    }

    #[test]
    fn test_span_length_on_start_line() {
        let single = Span::new("test.oxur".to_string(), 1, 5, 1, 15);
        assert_eq!(single.length_on_start_line(), 10);

        let multi = Span::new("test.oxur".to_string(), 1, 5, 3, 10);
        // For multi-line, returns 0 (undefined without source text)
        assert_eq!(multi.length_on_start_line(), 0);
    }

    #[test]
    fn test_span_contains_same_line() {
        let outer = Span::new("test.oxur".to_string(), 1, 5, 1, 20);
        let inner = Span::new("test.oxur".to_string(), 1, 10, 1, 15);
        let before = Span::new("test.oxur".to_string(), 1, 1, 1, 4);
        let after = Span::new("test.oxur".to_string(), 1, 21, 1, 25);

        assert!(outer.contains(&inner));
        assert!(!outer.contains(&before));
        assert!(!outer.contains(&after));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_span_contains_multi_line() {
        let outer = Span::new("test.oxur".to_string(), 1, 5, 5, 20);
        let inner = Span::new("test.oxur".to_string(), 2, 1, 3, 10);
        let overlapping = Span::new("test.oxur".to_string(), 1, 1, 2, 10);

        assert!(outer.contains(&inner));
        assert!(!outer.contains(&overlapping)); // Starts before outer
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_span_contains_different_files() {
        let span1 = Span::new("file1.oxur".to_string(), 1, 5, 1, 20);
        let span2 = Span::new("file2.oxur".to_string(), 1, 10, 1, 15);

        assert!(!span1.contains(&span2));
    }

    #[test]
    fn test_span_merge_same_line() {
        let span1 = Span::new("test.oxur".to_string(), 1, 5, 1, 10);
        let span2 = Span::new("test.oxur".to_string(), 1, 15, 1, 20);
        let merged = span1.merge(&span2);

        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.start_column, 5);
        assert_eq!(merged.end_line, 1);
        assert_eq!(merged.end_column, 20);
    }

    #[test]
    fn test_span_merge_multi_line() {
        let span1 = Span::new("test.oxur".to_string(), 1, 5, 2, 10);
        let span2 = Span::new("test.oxur".to_string(), 3, 1, 4, 5);
        let merged = span1.merge(&span2);

        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.start_column, 5);
        assert_eq!(merged.end_line, 4);
        assert_eq!(merged.end_column, 5);
    }

    #[test]
    fn test_span_merge_overlapping() {
        let span1 = Span::new("test.oxur".to_string(), 1, 5, 2, 10);
        let span2 = Span::new("test.oxur".to_string(), 2, 1, 3, 5);
        let merged = span1.merge(&span2);

        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.start_column, 5);
        assert_eq!(merged.end_line, 3);
        assert_eq!(merged.end_column, 5);
    }

    #[test]
    fn test_span_merge_reversed() {
        let span1 = Span::new("test.oxur".to_string(), 3, 1, 4, 5);
        let span2 = Span::new("test.oxur".to_string(), 1, 5, 2, 10);
        let merged = span1.merge(&span2);

        // Should produce same result regardless of order
        assert_eq!(merged.start_line, 1);
        assert_eq!(merged.start_column, 5);
        assert_eq!(merged.end_line, 4);
        assert_eq!(merged.end_column, 5);
    }

    #[test]
    #[should_panic(expected = "Cannot merge spans from different files")]
    fn test_span_merge_different_files() {
        let span1 = Span::new("file1.oxur".to_string(), 1, 5, 1, 10);
        let span2 = Span::new("file2.oxur".to_string(), 1, 15, 1, 20);
        span1.merge(&span2);
    }

    #[test]
    #[should_panic(expected = "Line numbers are 1-indexed")]
    fn test_span_zero_start_line() {
        Span::new("test.oxur".to_string(), 0, 1, 1, 10);
    }

    #[test]
    #[should_panic(expected = "Column numbers are 1-indexed")]
    fn test_span_zero_start_column() {
        Span::new("test.oxur".to_string(), 1, 0, 1, 10);
    }

    #[test]
    #[should_panic(expected = "Line numbers are 1-indexed")]
    fn test_span_zero_end_line() {
        Span::new("test.oxur".to_string(), 1, 1, 0, 10);
    }

    #[test]
    #[should_panic(expected = "Column numbers are 1-indexed")]
    fn test_span_zero_end_column() {
        Span::new("test.oxur".to_string(), 1, 1, 1, 0);
    }

    #[test]
    #[should_panic(expected = "End line must be >= start line")]
    fn test_span_end_before_start_line() {
        Span::new("test.oxur".to_string(), 5, 1, 3, 10);
    }

    #[test]
    #[should_panic(expected = "End column must be > start column on same line")]
    fn test_span_end_before_start_column() {
        Span::new("test.oxur".to_string(), 1, 10, 1, 5);
    }

    #[test]
    #[should_panic(expected = "End column must be > start column on same line")]
    fn test_span_equal_positions() {
        Span::new("test.oxur".to_string(), 1, 10, 1, 10);
    }

    #[test]
    fn test_span_to_source_pos() {
        let span = Span::new("test.oxur".to_string(), 1, 5, 1, 15);
        let pos: crate::SourcePos = (&span).into();

        assert_eq!(pos.file, "test.oxur");
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.length, 10);
    }

    #[test]
    #[should_panic(expected = "Can only convert single-line Span to SourcePos")]
    fn test_span_to_source_pos_multi_line() {
        let span = Span::new("test.oxur".to_string(), 1, 5, 3, 10);
        let _pos: crate::SourcePos = (&span).into();
    }
}
