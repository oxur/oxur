//! Configuration for S-expression formatting.

use crate::error::{FormatterError, Result};

/// Configuration for formatting S-expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct FormatConfig {
    /// Maximum line width before breaking to multiple lines (default: 100, matches rustfmt)
    pub max_width: usize,

    /// Number of spaces per indentation level (default: 2)
    pub tab_spaces: usize,

    /// Whether to align keywords vertically (default: true)
    pub align_keywords: bool,

    /// Whether to keep simple values on one line (default: true)
    pub compact_simple_values: bool,

    /// Maximum number of items to keep on one line (default: 3)
    pub max_inline_items: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            max_width: 100,              // Match rustfmt default
            tab_spaces: 2,               // Common Lisp convention
            align_keywords: true,        // Readable keyword-value pairs
            compact_simple_values: true, // (Span :lo 0 :hi 0) on one line
            max_inline_items: 3,         // Keep small lists inline
        }
    }
}

impl FormatConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum line width.
    pub fn with_max_width(mut self, width: usize) -> Result<Self> {
        if width == 0 {
            return Err(FormatterError::invalid_config("max_width must be greater than 0"));
        }
        self.max_width = width;
        Ok(self)
    }

    /// Set the number of spaces per indentation level.
    pub fn with_tab_spaces(mut self, spaces: usize) -> Result<Self> {
        if spaces == 0 {
            return Err(FormatterError::invalid_config("tab_spaces must be greater than 0"));
        }
        self.tab_spaces = spaces;
        Ok(self)
    }

    /// Set whether to align keywords vertically.
    pub fn with_align_keywords(mut self, align: bool) -> Self {
        self.align_keywords = align;
        self
    }

    /// Set whether to compact simple values.
    pub fn with_compact_simple_values(mut self, compact: bool) -> Self {
        self.compact_simple_values = compact;
        self
    }

    /// Set the maximum number of inline items.
    pub fn with_max_inline_items(mut self, max: usize) -> Self {
        self.max_inline_items = max;
        self
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.max_width == 0 {
            return Err(FormatterError::invalid_config("max_width must be greater than 0"));
        }
        if self.tab_spaces == 0 {
            return Err(FormatterError::invalid_config("tab_spaces must be greater than 0"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FormatConfig::default();
        assert_eq!(config.max_width, 100);
        assert_eq!(config.tab_spaces, 2);
        assert!(config.align_keywords);
        assert!(config.compact_simple_values);
        assert_eq!(config.max_inline_items, 3);
    }

    #[test]
    fn test_new_config() {
        let config = FormatConfig::new();
        assert_eq!(config, FormatConfig::default());
    }

    #[test]
    fn test_with_max_width() {
        let config = FormatConfig::new().with_max_width(120).unwrap();
        assert_eq!(config.max_width, 120);
    }

    #[test]
    fn test_with_max_width_zero_error() {
        let result = FormatConfig::new().with_max_width(0);
        assert!(matches!(result, Err(FormatterError::InvalidConfig { .. })));
    }

    #[test]
    fn test_with_tab_spaces() {
        let config = FormatConfig::new().with_tab_spaces(4).unwrap();
        assert_eq!(config.tab_spaces, 4);
    }

    #[test]
    fn test_with_tab_spaces_zero_error() {
        let result = FormatConfig::new().with_tab_spaces(0);
        assert!(matches!(result, Err(FormatterError::InvalidConfig { .. })));
    }

    #[test]
    fn test_with_align_keywords() {
        let config = FormatConfig::new().with_align_keywords(false);
        assert!(!config.align_keywords);
    }

    #[test]
    fn test_with_compact_simple_values() {
        let config = FormatConfig::new().with_compact_simple_values(false);
        assert!(!config.compact_simple_values);
    }

    #[test]
    fn test_with_max_inline_items() {
        let config = FormatConfig::new().with_max_inline_items(5);
        assert_eq!(config.max_inline_items, 5);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = FormatConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_max_width() {
        let mut config = FormatConfig::default();
        config.max_width = 0;
        assert!(matches!(config.validate(), Err(FormatterError::InvalidConfig { .. })));
    }

    #[test]
    fn test_validate_zero_tab_spaces() {
        let mut config = FormatConfig::default();
        config.tab_spaces = 0;
        assert!(matches!(config.validate(), Err(FormatterError::InvalidConfig { .. })));
    }

    #[test]
    fn test_builder_chain() {
        let config = FormatConfig::new()
            .with_max_width(80)
            .unwrap()
            .with_tab_spaces(4)
            .unwrap()
            .with_align_keywords(false)
            .with_compact_simple_values(false)
            .with_max_inline_items(5);

        assert_eq!(config.max_width, 80);
        assert_eq!(config.tab_spaces, 4);
        assert!(!config.align_keywords);
        assert!(!config.compact_simple_values);
        assert_eq!(config.max_inline_items, 5);
    }
}
