//! Styled table rendering for Oxur tools
//!
//! Provides a flexible table builder with TOML-based theming for terminal output.
//!
//! # Examples
//!
//! ```no_run
//! use oxur_table::{OxurTable, Tabled};
//!
//! #[derive(Tabled)]
//! struct Employee {
//!     #[tabled(rename = "Name")]
//!     name: String,
//!     #[tabled(rename = "Age")]
//!     age: u32,
//!     #[tabled(rename = "Role")]
//!     role: String,
//! }
//!
//! let employees = vec![
//!     Employee { name: "Alice".into(), age: 30, role: "Engineer".into() },
//!     Employee { name: "Bob".into(), age: 25, role: "Designer".into() },
//! ];
//!
//! let table = OxurTable::new(employees).render();
//! println!("{}", table);
//! ```

use tabled::Table;

mod config;
mod themes;

pub use config::TableStyleConfig;
pub use tabled::Tabled; // Re-export for convenience

/// A themed table builder for terminal output
///
/// Creates tables with the default Oxur theme (warm orange sunset colors).
/// Supports any data type that implements `Tabled`.
pub struct OxurTable<T: Tabled> {
    data: Vec<T>,
    theme: TableStyleConfig,
}

impl<T: Tabled> OxurTable<T> {
    /// Create a new table with data, using the default Oxur theme
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_table::{OxurTable, Tabled};
    ///
    /// #[derive(Tabled)]
    /// struct Row {
    ///     #[tabled(rename = "ID")]
    ///     id: u32,
    ///     #[tabled(rename = "Name")]
    ///     name: String,
    /// }
    ///
    /// let data = vec![
    ///     Row { id: 1, name: "Alice".into() },
    ///     Row { id: 2, name: "Bob".into() },
    /// ];
    ///
    /// let table = OxurTable::new(data);
    /// ```
    pub fn new(data: Vec<T>) -> Self {
        Self { data, theme: TableStyleConfig::default() }
    }

    /// Render the table as a styled string for terminal output
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_table::{OxurTable, Tabled};
    ///
    /// #[derive(Tabled)]
    /// struct Row {
    ///     name: String,
    /// }
    ///
    /// let data = vec![Row { name: "Test".into() }];
    /// let output = OxurTable::new(data).render();
    /// println!("{}", output);
    /// ```
    pub fn render(self) -> String {
        let mut table = Table::new(&self.data);
        self.theme.apply_to_table::<T>(&mut table);
        table.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Tabled)]
    struct TestRow {
        #[tabled(rename = "ID")]
        id: u32,
        #[tabled(rename = "Name")]
        name: String,
        #[tabled(rename = "Status")]
        status: String,
    }

    // ===== OxurTable::new tests =====

    #[test]
    fn test_new_creates_table_with_default_theme() {
        let data = vec![TestRow { id: 1, name: "Alice".into(), status: "Active".into() }];

        let table = OxurTable::new(data);

        // Verify the table was created (we can't easily inspect internals)
        // Just ensure it doesn't panic
        assert_eq!(table.data.len(), 1);
    }

    #[test]
    fn test_new_with_empty_data() {
        let data: Vec<TestRow> = vec![];
        let table = OxurTable::new(data);
        assert_eq!(table.data.len(), 0);
    }

    #[test]
    fn test_new_with_multiple_rows() {
        let data = vec![
            TestRow { id: 1, name: "Alice".into(), status: "Active".into() },
            TestRow { id: 2, name: "Bob".into(), status: "Inactive".into() },
            TestRow { id: 3, name: "Charlie".into(), status: "Active".into() },
        ];

        let table = OxurTable::new(data);
        assert_eq!(table.data.len(), 3);
    }

    // ===== OxurTable::render tests =====

    #[test]
    fn test_render_produces_output() {
        let data = vec![TestRow { id: 1, name: "Alice".into(), status: "Active".into() }];

        let table = OxurTable::new(data);
        let output = table.render();

        // Verify output is not empty and contains data
        assert!(!output.is_empty());
        assert!(output.contains("Alice"));
        assert!(output.contains("Active"));
    }

    #[test]
    fn test_render_empty_data() {
        let data: Vec<TestRow> = vec![];
        let table = OxurTable::new(data);
        let output = table.render();

        // Should still produce some output (at least headers)
        assert!(!output.is_empty());
    }

    #[test]
    fn test_render_multiple_rows() {
        let data = vec![
            TestRow { id: 1, name: "Alice".into(), status: "Active".into() },
            TestRow { id: 2, name: "Bob".into(), status: "Inactive".into() },
        ];

        let table = OxurTable::new(data);
        let output = table.render();

        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("Active"));
        assert!(output.contains("Inactive"));
    }

    #[test]
    fn test_render_includes_headers() {
        let data = vec![TestRow { id: 1, name: "Test".into(), status: "OK".into() }];

        let table = OxurTable::new(data);
        let output = table.render();

        // Headers from #[tabled(rename = "...")] should be present
        assert!(output.contains("ID"));
        assert!(output.contains("Name"));
        assert!(output.contains("Status"));
    }

    #[test]
    fn test_render_contains_ansi_codes() {
        let data = vec![TestRow { id: 1, name: "Test".into(), status: "OK".into() }];

        let table = OxurTable::new(data);
        let output = table.render();

        // Should contain ANSI escape codes for colors
        // (The theme applies colors, so there should be escape sequences)
        assert!(output.contains("\x1b[") || output.contains("\u{001b}["));
    }

    // ===== Integration tests =====

    #[test]
    fn test_table_with_special_characters() {
        let data = vec![TestRow { id: 1, name: "Test & \"Special\"".into(), status: "OK".into() }];

        let table = OxurTable::new(data);
        let output = table.render();

        assert!(output.contains("Test & \"Special\""));
    }

    #[test]
    fn test_table_with_unicode() {
        let data = vec![TestRow { id: 1, name: "Ñoño 日本語".into(), status: "✓".into() }];

        let table = OxurTable::new(data);
        let output = table.render();

        assert!(output.contains("Ñoño"));
        assert!(output.contains("日本語"));
        assert!(output.contains("✓"));
    }

    #[test]
    fn test_table_with_long_text() {
        let long_name = "A".repeat(100);
        let data = vec![TestRow { id: 1, name: long_name.clone(), status: "OK".into() }];

        let table = OxurTable::new(data);
        let output = table.render();

        // Long text should be in the output (might be wrapped or truncated by tabled)
        assert!(output.contains(&long_name[..50])); // At least first 50 chars
    }

    #[test]
    fn test_table_with_empty_strings() {
        let data = vec![TestRow { id: 1, name: "".into(), status: "".into() }];

        let table = OxurTable::new(data);
        let output = table.render();

        // Should handle empty strings gracefully
        assert!(!output.is_empty());
        assert!(output.contains("ID")); // Headers should still be present
    }

    #[test]
    fn test_different_struct_type() {
        #[derive(Tabled)]
        struct DifferentRow {
            #[tabled(rename = "Col1")]
            col1: String,
            #[tabled(rename = "Col2")]
            col2: i32,
        }

        let data = vec![DifferentRow { col1: "Test".into(), col2: 42 }];

        let table = OxurTable::new(data);
        let output = table.render();

        assert!(output.contains("Test"));
        assert!(output.contains("42"));
        assert!(output.contains("Col1"));
        assert!(output.contains("Col2"));
    }
}
