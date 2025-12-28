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

use tabled::{Table, builder::Builder};

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
        Self {
            data,
            theme: TableStyleConfig::default(),
        }
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

    /// Render using manual builder (no auto-generated header)
    ///
    /// This builds the table manually using Builder::default(), which gives full control
    /// over row structure. Use with structs that have empty #[tabled(rename = "")] attributes.
    pub fn render_without_header(self) -> String {
        // Build table manually using Builder, just like the sample code
        let mut builder = Builder::default();

        // Convert each Tabled struct into its string representation and push as record
        for item in &self.data {
            let temp_table = Table::new(std::slice::from_ref(item));
            let table_str = temp_table.to_string();

            // Extract the data row (skip header line and separator lines)
            for line in table_str.lines().skip(1) {
                if line.contains('│') {
                    let cells: Vec<String> = line
                        .split('│')
                        .skip(1) // Skip leading empty cell from border
                        .take_while(|_| true)
                        .filter(|s| !s.trim().is_empty() || true)
                        .map(|s| s.trim().to_string())
                        .collect();

                    if !cells.is_empty() {
                        builder.push_record(cells);
                        break; // Only take the first data row
                    }
                }
            }
        }

        let mut table = builder.build();
        self.theme.apply_to_table::<T>(&mut table);
        table.to_string()
    }
}
