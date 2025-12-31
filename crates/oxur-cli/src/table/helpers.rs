//! Helper functions for advanced table styling
//!
//! This module provides utilities for applying cell-specific colors while
//! preserving the theme's background colors. Use these when you need more
//! control than the simple `OxurTable::new(data).render()` pattern.
//!
//! # Example: Cell-Specific Coloring
//!
//! ```no_run
//! use oxur_cli::table::{helpers, TableStyleConfig, Builder, TabledColor, Tabled};
//!
//! #[derive(Tabled)]
//! struct Row {
//!     id: String,
//!     name: String,
//!     status: String,
//! }
//!
//! // Build table manually
//! let mut builder = Builder::default();
//! builder.push_record(["ID", "Name", "Status"]);
//! builder.push_record(["001", "Alice", "Active"]);
//! builder.push_record(["002", "Bob", "Inactive"]);
//!
//! let mut table = builder.build();
//!
//! // Apply theme
//! let theme = TableStyleConfig::default();
//! theme.apply_to_table::<Row>(&mut table);
//!
//! // Get row background colors from theme
//! let row_bg_colors = helpers::parse_row_bg_colors(&theme);
//!
//! // Apply cell-specific colors to status column
//! let fg_color = TabledColor::FG_GREEN;
//! let bg_color = helpers::get_data_row_bg_color(0, &row_bg_colors);
//! helpers::apply_cell_color(&mut table, 2, 2, fg_color, bg_color);
//! ```

use tabled::settings::{object::Cell, Color as TabledColor};
use tabled::Table;

use super::config::{parse_bg_color, TableStyleConfig};

/// Parse row background colors from theme configuration.
///
/// Returns a vector of background colors for alternating rows as defined
/// in the theme's `rows.colors` configuration.
///
/// # Example
///
/// ```no_run
/// use oxur_cli::table::{helpers, TableStyleConfig};
///
/// let theme = TableStyleConfig::default();
/// let row_bg_colors = helpers::parse_row_bg_colors(&theme);
/// assert_eq!(row_bg_colors.len(), 2); // Default theme has 2 alternating colors
/// ```
pub fn parse_row_bg_colors(theme: &TableStyleConfig) -> Vec<TabledColor> {
    theme.rows.colors.iter().map(|rc| parse_bg_color(&rc.bg)).collect()
}

/// Get the background color for a data row using alternating colors.
///
/// # Arguments
///
/// * `data_row_index` - The 0-indexed position within the data section (not the absolute table row)
/// * `row_bg_colors` - Vector of background colors (typically from `parse_row_bg_colors()`)
///
/// # Returns
///
/// The background color for this row, selected using modulo for alternating colors.
///
/// # Example
///
/// ```no_run
/// use oxur_cli::table::{helpers, TableStyleConfig};
///
/// let theme = TableStyleConfig::default();
/// let row_bg_colors = helpers::parse_row_bg_colors(&theme);
///
/// // First data row gets first color
/// let bg0 = helpers::get_data_row_bg_color(0, &row_bg_colors);
///
/// // Second data row gets second color
/// let bg1 = helpers::get_data_row_bg_color(1, &row_bg_colors);
///
/// // Third data row wraps back to first color
/// let bg2 = helpers::get_data_row_bg_color(2, &row_bg_colors);
/// ```
pub fn get_data_row_bg_color(data_row_index: usize, row_bg_colors: &[TabledColor]) -> TabledColor {
    let color_idx = data_row_index % row_bg_colors.len();
    row_bg_colors[color_idx].clone()
}

/// Apply a foreground color to a specific cell while preserving its background.
///
/// This is the core pattern for cell-specific coloring. The `fg_color | bg_color`
/// combination ensures the background from the theme is preserved while applying
/// a custom foreground color.
///
/// # Arguments
///
/// * `table` - The table to modify
/// * `row_idx` - Absolute row index in the table (0 = title if enabled, 1 = header, 2+ = data)
/// * `col_idx` - Column index (0-based)
/// * `fg_color` - Foreground color to apply
/// * `bg_color` - Background color to preserve (typically from `get_data_row_bg_color()`)
///
/// # Example
///
/// ```no_run
/// use oxur_cli::table::{helpers, TableStyleConfig, Builder, TabledColor, Tabled};
///
/// #[derive(Tabled)]
/// struct Row { col: String }
///
/// let mut builder = Builder::default();
/// builder.push_record(["Title"]);
/// builder.push_record(["Header"]);
/// builder.push_record(["Data"]);
///
/// let mut table = builder.build();
///
/// let theme = TableStyleConfig::default();
/// theme.apply_to_table::<Row>(&mut table);
///
/// let row_bg_colors = helpers::parse_row_bg_colors(&theme);
/// let bg = helpers::get_data_row_bg_color(0, &row_bg_colors);
///
/// // Apply green foreground to data cell (row 2, col 0)
/// helpers::apply_cell_color(&mut table, 2, 0, TabledColor::FG_GREEN, bg);
/// ```
pub fn apply_cell_color(
    table: &mut Table,
    row_idx: usize,
    col_idx: usize,
    fg_color: TabledColor,
    bg_color: TabledColor,
) {
    let combined = fg_color | bg_color;
    table.modify(Cell::new(row_idx, col_idx), combined);
}

/// Map a state string to a foreground color.
///
/// This is a domain-specific helper for design document states. You can use this
/// as an example and create your own mapping functions for your domain.
///
/// # Supported States
///
/// * `"draft"` → Yellow
/// * `"under review"` / `"under-review"` → Cyan
/// * `"revised"` → Blue
/// * `"accepted"` → Green
/// * `"active"` → Bright Green
/// * `"final"` → Green
/// * `"deferred"` → Magenta
/// * `"rejected"` → Red
/// * `"withdrawn"` → Red
/// * `"superseded"` → Red
///
/// # Returns
///
/// `Some(color)` if the state is recognized, `None` otherwise.
///
/// # Example
///
/// ```no_run
/// use oxur_cli::table::helpers;
///
/// let color = helpers::state_to_fg_color("active");
/// assert!(color.is_some());
///
/// let unknown = helpers::state_to_fg_color("invalid");
/// assert!(unknown.is_none());
/// ```
pub fn state_to_fg_color(state: &str) -> Option<TabledColor> {
    match state.to_lowercase().as_str() {
        "draft" => Some(TabledColor::FG_YELLOW),
        "under review" | "under-review" => Some(TabledColor::FG_CYAN),
        "revised" => Some(TabledColor::FG_BLUE),
        "accepted" => Some(TabledColor::FG_GREEN),
        "active" => Some(TabledColor::FG_BRIGHT_GREEN),
        "final" => Some(TabledColor::FG_GREEN),
        "deferred" => Some(TabledColor::FG_MAGENTA),
        "rejected" => Some(TabledColor::FG_RED),
        "withdrawn" => Some(TabledColor::FG_RED),
        "superseded" => Some(TabledColor::FG_RED),
        _ => None,
    }
}

/// Map a deleted boolean to a foreground color.
///
/// This is a domain-specific helper for showing deletion status.
///
/// # Returns
///
/// * `true` (deleted) → Red
/// * `false` (not deleted) → Green
///
/// # Example
///
/// ```no_run
/// use oxur_cli::table::helpers;
/// use tabled::settings::Color as TabledColor;
///
/// let deleted = helpers::deleted_to_fg_color(true);
/// assert_eq!(deleted, TabledColor::FG_RED);
///
/// let not_deleted = helpers::deleted_to_fg_color(false);
/// assert_eq!(not_deleted, TabledColor::FG_GREEN);
/// ```
pub fn deleted_to_fg_color(deleted: bool) -> TabledColor {
    if deleted {
        TabledColor::FG_RED
    } else {
        TabledColor::FG_GREEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_row_bg_colors() {
        let theme = TableStyleConfig::default();
        let colors = parse_row_bg_colors(&theme);

        // Default Oxur theme has 2 alternating row colors
        assert_eq!(colors.len(), 2);
    }

    #[test]
    fn test_get_data_row_bg_color_alternating() {
        let theme = TableStyleConfig::default();
        let colors = parse_row_bg_colors(&theme);

        // Test alternating pattern
        let bg0 = get_data_row_bg_color(0, &colors);
        let bg1 = get_data_row_bg_color(1, &colors);
        let bg2 = get_data_row_bg_color(2, &colors);
        let bg3 = get_data_row_bg_color(3, &colors);

        // Should alternate (comparing via Debug since Color doesn't implement Eq)
        assert_eq!(format!("{:?}", bg0), format!("{:?}", bg2));
        assert_eq!(format!("{:?}", bg1), format!("{:?}", bg3));
    }

    #[test]
    fn test_state_to_fg_color_recognized() {
        assert!(state_to_fg_color("active").is_some());
        assert!(state_to_fg_color("Active").is_some()); // Case insensitive
        assert!(state_to_fg_color("ACTIVE").is_some());
        assert!(state_to_fg_color("draft").is_some());
        assert!(state_to_fg_color("under review").is_some());
        assert!(state_to_fg_color("under-review").is_some());
        assert!(state_to_fg_color("accepted").is_some());
        assert!(state_to_fg_color("final").is_some());
        assert!(state_to_fg_color("rejected").is_some());
    }

    #[test]
    fn test_state_to_fg_color_unrecognized() {
        assert!(state_to_fg_color("invalid").is_none());
        assert!(state_to_fg_color("").is_none());
        assert!(state_to_fg_color("unknown-state").is_none());
    }

    #[test]
    fn test_deleted_to_fg_color() {
        let deleted = deleted_to_fg_color(true);
        let not_deleted = deleted_to_fg_color(false);

        assert_eq!(deleted, TabledColor::FG_RED);
        assert_eq!(not_deleted, TabledColor::FG_GREEN);
    }

    #[test]
    fn test_apply_cell_color_does_not_panic() {
        use tabled::builder::Builder;
        use tabled::Tabled;

        #[derive(Tabled)]
        struct TestRow {
            col: String,
        }

        let mut builder = Builder::default();
        builder.push_record(["Title"]);
        builder.push_record(["Header"]);
        builder.push_record(["Data"]);

        let mut table = builder.build();

        let theme = TableStyleConfig::default();
        theme.apply_to_table::<TestRow>(&mut table);

        let row_bg_colors = parse_row_bg_colors(&theme);
        let bg = get_data_row_bg_color(0, &row_bg_colors);

        // Should not panic
        apply_cell_color(&mut table, 2, 0, TabledColor::FG_GREEN, bg);
    }
}
