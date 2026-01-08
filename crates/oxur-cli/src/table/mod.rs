//! Styled table rendering for Oxur tools
//!
//! Provides a flexible table builder with TOML-based theming for terminal output.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```no_run
//! use oxur_cli::table::{OxurTable, Tabled};
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
//!
//! ## With Title and Footer
//!
//! ```no_run
//! use oxur_cli::table::{OxurTable, Tabled};
//!
//! #[derive(Tabled)]
//! struct Row {
//!     #[tabled(rename = "ID")]
//!     id: u32,
//!     #[tabled(rename = "Name")]
//!     name: String,
//! }
//!
//! let data = vec![
//!     Row { id: 1, name: "Alice".into() },
//!     Row { id: 2, name: "Bob".into() },
//! ];
//!
//! let table = OxurTable::new(data)
//!     .with_title("USER LIST")
//!     .with_footer()
//!     .render();
//! println!("{}", table);
//! ```

use tabled::Table;

pub mod config;
pub mod helpers;
mod themes;

pub use config::TableStyleConfig;
pub use tabled::Tabled; // Re-export for convenience

// Re-exports for advanced usage (manual table building and cell coloring)
pub use tabled::builder::Builder;
pub use tabled::settings::object::Cell;
pub use tabled::settings::Color as TabledColor;

/// A themed table builder for terminal output
///
/// Creates tables with the default Oxur theme (warm orange sunset colors).
/// Supports any data type that implements `Tabled`.
///
/// The table can optionally include:
/// - A **title row** at the top (styled with the theme's title colors)
/// - A **footer row** at the bottom (styled with the theme's footer colors)
pub struct OxurTable<T: Tabled> {
    data: Vec<T>,
    theme: TableStyleConfig,
    title: Option<String>,
    has_footer: bool,
}

impl<T: Tabled> OxurTable<T> {
    /// Create a new table with data, using the default Oxur theme
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_cli::table::{OxurTable, Tabled};
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
        Self { data, theme: TableStyleConfig::default(), title: None, has_footer: false }
    }

    /// Add a title row at the top of the table
    ///
    /// The title will span all columns and be styled with the theme's title colors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_cli::table::{OxurTable, Tabled};
    ///
    /// #[derive(Tabled)]
    /// struct Row { name: String }
    ///
    /// let data = vec![Row { name: "Test".into() }];
    /// let table = OxurTable::new(data)
    ///     .with_title("MY TABLE")
    ///     .render();
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add a blank footer row at the bottom of the table
    ///
    /// The footer provides visual closure and is styled with the theme's footer colors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_cli::table::{OxurTable, Tabled};
    ///
    /// #[derive(Tabled)]
    /// struct Row { name: String }
    ///
    /// let data = vec![Row { name: "Test".into() }];
    /// let table = OxurTable::new(data)
    ///     .with_footer()
    ///     .render();
    /// ```
    pub fn with_footer(mut self) -> Self {
        self.has_footer = true;
        self
    }

    /// Render the table as a styled string for terminal output
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_cli::table::{OxurTable, Tabled};
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
        // If no title and no footer, use the simple path
        if self.title.is_none() && !self.has_footer {
            let mut table = Table::new(&self.data);
            self.theme.apply_to_table::<T>(&mut table);
            return table.to_string();
        }

        // Use Builder to add title and/or footer rows to the data
        let col_count = T::headers().len();
        let mut builder = Builder::default();

        // Add title row (first column has title, rest are empty)
        if let Some(ref title) = self.title {
            let mut title_row = vec![title.clone()];
            title_row.extend(std::iter::repeat_n(String::new(), col_count.saturating_sub(1)));
            builder.push_record(title_row);
        }

        // Add header row (column names from Tabled)
        let headers: Vec<String> = T::headers().iter().map(|h| h.to_string()).collect();
        builder.push_record(headers);

        // Add data rows
        for item in &self.data {
            let fields: Vec<String> = T::fields(item).iter().map(|f| f.to_string()).collect();
            builder.push_record(fields);
        }

        // Add footer row (all empty)
        if self.has_footer {
            let footer_row: Vec<String> = std::iter::repeat_n(String::new(), col_count).collect();
            builder.push_record(footer_row);
        }

        let mut table = builder.build();
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

    // ===== with_title tests =====

    #[test]
    fn test_with_title_adds_title_row() {
        let data = vec![TestRow { id: 1, name: "Alice".into(), status: "Active".into() }];

        let table = OxurTable::new(data).with_title("MY TABLE");
        let output = table.render();

        // Title should appear in the output
        assert!(output.contains("MY TABLE"));
        // Data should still be present
        assert!(output.contains("Alice"));
    }

    #[test]
    fn test_with_title_string_slice() {
        let data = vec![TestRow { id: 1, name: "Test".into(), status: "OK".into() }];

        let table = OxurTable::new(data).with_title("TITLE FROM &str");
        let output = table.render();

        assert!(output.contains("TITLE FROM &str"));
    }

    #[test]
    fn test_with_title_owned_string() {
        let data = vec![TestRow { id: 1, name: "Test".into(), status: "OK".into() }];
        let title = String::from("OWNED TITLE");

        let table = OxurTable::new(data).with_title(title);
        let output = table.render();

        assert!(output.contains("OWNED TITLE"));
    }

    // ===== with_footer tests =====

    #[test]
    fn test_with_footer_adds_footer_row() {
        let data = vec![TestRow { id: 1, name: "Alice".into(), status: "Active".into() }];

        let table = OxurTable::new(data).with_footer();
        let output = table.render();

        // Data should be present
        assert!(output.contains("Alice"));
        // Output should be longer than without footer (footer adds a row)
        let without_footer =
            OxurTable::new(vec![TestRow { id: 1, name: "Alice".into(), status: "Active".into() }])
                .render();
        assert!(output.len() > without_footer.len());
    }

    // ===== with_title and with_footer combined tests =====

    #[test]
    fn test_with_title_and_footer() {
        let data = vec![
            TestRow { id: 1, name: "Alice".into(), status: "Active".into() },
            TestRow { id: 2, name: "Bob".into(), status: "Inactive".into() },
        ];

        let table = OxurTable::new(data).with_title("USER LIST").with_footer();
        let output = table.render();

        // Title should appear
        assert!(output.contains("USER LIST"));
        // Data should be present
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        // Headers should be present
        assert!(output.contains("ID"));
        assert!(output.contains("Name"));
        assert!(output.contains("Status"));
    }

    #[test]
    fn test_chaining_order_does_not_matter() {
        let data1 = vec![TestRow { id: 1, name: "Test".into(), status: "OK".into() }];
        let data2 = vec![TestRow { id: 1, name: "Test".into(), status: "OK".into() }];

        let output1 = OxurTable::new(data1).with_title("TITLE").with_footer().render();
        let output2 = OxurTable::new(data2).with_footer().with_title("TITLE").render();

        // Both should produce the same result
        assert_eq!(output1, output2);
    }

    #[test]
    fn test_empty_data_with_title_and_footer() {
        let data: Vec<TestRow> = vec![];

        let table = OxurTable::new(data).with_title("EMPTY TABLE").with_footer();
        let output = table.render();

        // Title should appear
        assert!(output.contains("EMPTY TABLE"));
        // Headers should be present
        assert!(output.contains("ID"));
    }
}
