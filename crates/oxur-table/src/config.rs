use serde::Deserialize;
use tabled::{
    settings::{
        formatting::Justification,
        object::{Object, Rows, Segment},
        style::BorderColor,
        style::Style,
        themes::Colorization,
        Color, Padding, Width,
    },
    Table,
};

#[derive(Debug, Deserialize)]
pub struct TableStyleConfig {
    pub table: TableConfig,
    #[serde(default)]
    pub title: Option<TitleConfig>,
    pub header: HeaderConfig,
    pub rows: RowsConfig,
    pub style: StyleConfig,
    #[serde(default)]
    pub footer: Option<FooterConfig>,
}

/// Title configuration - styles a title row ABOVE the header
///
/// To create a title row, add it as the FIRST data row in your data structure.
/// Then set `enabled = true` in the TOML config.
#[derive(Debug, Deserialize, Clone)]
pub struct TitleConfig {
    /// Enable title styling on the first data row
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bg_color: Option<String>,
    #[serde(default)]
    pub fg_color: Option<String>,
    #[serde(default)]
    pub justification_char: Option<String>,
    #[serde(default)]
    pub vertical_char: Option<String>,
    #[serde(default)]
    pub vertical_fg_color: Option<String>,
    #[serde(default)]
    pub vertical_bg_color: Option<String>,
}

/// Footer configuration - styles the LAST ROW of your data
///
/// To create a visual footer bar, add a footer row as the last element in your data.
/// Then set `enabled = true` in the TOML config.
#[derive(Debug, Deserialize, Clone)]
pub struct FooterConfig {
    /// Enable footer styling on the last data row
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bg_color: Option<String>,
    #[serde(default)]
    pub fg_color: Option<String>,
    #[serde(default)]
    pub justification_char: Option<String>,
    #[serde(default)]
    pub vertical_char: Option<String>,
    #[serde(default)]
    pub vertical_fg_color: Option<String>,
    #[serde(default)]
    pub vertical_bg_color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TableConfig {
    pub padding_left: usize,
    pub padding_right: usize,
    pub padding_top: usize,
    pub padding_bottom: usize,
}

#[derive(Debug, Deserialize)]
pub struct HeaderConfig {
    pub bg_color: String,
    pub fg_color: String,
    pub justification_char: String,
    #[serde(default)]
    pub vertical_char: Option<String>,
    #[serde(default)]
    pub vertical_fg_color: Option<String>,
    #[serde(default)]
    pub vertical_bg_color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RowsConfig {
    pub colors: Vec<RowColor>,
    #[serde(default)]
    pub justification_char: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RowColor {
    pub bg: String,
    pub fg: String,
}

#[derive(Debug, Deserialize)]
pub struct StyleConfig {
    #[serde(default)]
    pub vertical_char: Option<String>,
    #[serde(default)]
    pub vertical_fg_color: Option<String>,
    #[serde(default)]
    pub vertical_bg_color: Option<String>,
}

impl TableStyleConfig {
    /// Apply the configuration to a tabled Table
    pub fn apply_to_table<T>(&self, table: &mut Table)
    where
        T: tabled::Tabled,
    {
        // Apply base style (empty)
        table.with(Style::empty());

        // Determine which vertical separator character to use globally
        // The limitation: tabled only allows ONE vertical char for the whole table
        // Priority: use header.vertical_char if specified, else style.vertical_char
        let global_vert_char = self
            .header
            .vertical_char
            .as_deref()
            .or(self.style.vertical_char.as_deref())
            .unwrap_or("");

        // Apply global vertical separator if specified
        if !global_vert_char.is_empty() {
            let vert_char = global_vert_char.chars().next().unwrap_or(' ');
            table.with(Style::empty().vertical(vert_char));
        }

        // Apply padding
        table.with(Padding::new(
            self.table.padding_left,
            self.table.padding_right,
            self.table.padding_top,
            self.table.padding_bottom,
        ));

        // Ensure consistent row widths by setting column widths
        // This works correctly when table is built with plain text first
        table.with(Width::list([10, 75, 15]));

        // Apply alternating row colors
        let row_colors: Vec<Color> =
            self.rows.colors.iter().map(|rc| parse_color(&rc.bg, &rc.fg)).collect();
        table.with(Colorization::rows(row_colors));

        // Determine if title is enabled (we need this for data row styling)
        let title_enabled = self.title.as_ref().map(|t| t.enabled).unwrap_or(false);

        // Apply title styling if enabled (title is the first row = row index 0)
        if let Some(title) = &self.title {
            if title.enabled {
                // Determine title colors (default to header colors for consistency)
                let title_bg_str = title.bg_color.as_deref().unwrap_or(&self.header.bg_color);
                let title_fg_str = title.fg_color.as_deref().unwrap_or(&self.header.fg_color);
                let title_color = parse_color(title_bg_str, title_fg_str);
                let title_bg = parse_bg_color(title_bg_str);

                // Apply title row colors (first row = row index 0)
                table.modify(Rows::first(), title_color);

                // Apply title justification
                let title_just_char = title
                    .justification_char
                    .as_deref()
                    .or(Some(&self.header.justification_char))
                    .and_then(|s| s.chars().next())
                    .unwrap_or(' ');
                table.modify(
                    Rows::first(),
                    Justification::new(title_just_char).color(title_bg.clone()),
                );
            }
        }

        // Apply header styling (header is the second row = row index 1)
        let header_color = parse_color(&self.header.bg_color, &self.header.fg_color);
        table.modify(Rows::new(1..2), header_color);

        // Apply header justification
        let just_char = self.header.justification_char.chars().next().unwrap_or(' ');
        let header_bg = parse_bg_color(&self.header.bg_color);
        table.modify(Rows::new(1..2), Justification::new(just_char).color(header_bg.clone()));

        // Determine if footer is enabled (we need this for data row styling)
        let footer_enabled = self.footer.as_ref().map(|f| f.enabled).unwrap_or(false);

        // Apply data row justification if specified
        if let Some(data_just_char) =
            self.rows.justification_char.as_ref().and_then(|s| s.chars().next())
        {
            // Data rows start at row 2 (after title row 0 and header row 1)
            // Use the first data row background color for justification padding color
            let data_bg = parse_bg_color(&self.rows.colors[0].bg);

            // Apply justification to all rows except title, header, and footer
            match footer_enabled {
                true => {
                    // Exclude rows 0, 1, and last row
                    table.modify(
                        Segment::all().not(Rows::first()).not(Rows::new(1..2)).not(Rows::last()),
                        Justification::new(data_just_char).color(data_bg),
                    );
                }
                false => {
                    // Exclude rows 0 and 1 only
                    table.modify(
                        Segment::all().not(Rows::first()).not(Rows::new(1..2)),
                        Justification::new(data_just_char).color(data_bg),
                    );
                }
            }
        }

        // Color the vertical separators in different sections

        // 1. Color data row separators if colors specified
        if self.style.vertical_fg_color.is_some() || self.style.vertical_bg_color.is_some() {
            let fg = self.style.vertical_fg_color.as_deref().unwrap_or("white");
            let bg = self.style.vertical_bg_color.as_deref().unwrap_or("black");
            let vert_color = parse_color(bg, fg);

            // Color all vertical borders in data rows (excluding header, title if enabled, and footer if enabled)
            // We need to handle different combinations due to Rust's type system
            match (title_enabled, footer_enabled) {
                (true, true) => {
                    table.modify(
                        Segment::all().not(Rows::first()).not(Rows::new(1..2)).not(Rows::last()),
                        BorderColor::filled(vert_color),
                    );
                }
                (true, false) => {
                    table.modify(
                        Segment::all().not(Rows::first()).not(Rows::new(1..2)),
                        BorderColor::filled(vert_color),
                    );
                }
                (false, true) => {
                    table.modify(
                        Segment::all().not(Rows::first()).not(Rows::last()),
                        BorderColor::filled(vert_color),
                    );
                }
                (false, false) => {
                    table
                        .modify(Segment::all().not(Rows::first()), BorderColor::filled(vert_color));
                }
            }
        }

        // 2. Color title separators if enabled (title is now row 0)
        if let Some(title) = &self.title {
            if title.enabled {
                let title_bg_str = title.bg_color.as_deref().unwrap_or(&self.header.bg_color);

                if title.vertical_fg_color.is_some() || title.vertical_bg_color.is_some() {
                    let fg = title.vertical_fg_color.as_deref().unwrap_or("white");
                    let bg = title.vertical_bg_color.as_deref().unwrap_or(title_bg_str);
                    let title_vert_color = parse_color(bg, fg);

                    table.modify(
                        Segment::all().intersect(Rows::first()),
                        BorderColor::filled(title_vert_color),
                    );
                } else {
                    // Default: match title background
                    let title_bg = parse_bg_color(title_bg_str);
                    table.modify(
                        Segment::all().intersect(Rows::first()),
                        BorderColor::filled(title_bg),
                    );
                }
            }
        }

        // 3. Color header separators (header is now row 1)
        if self.header.vertical_fg_color.is_some() || self.header.vertical_bg_color.is_some() {
            let fg = self.header.vertical_fg_color.as_deref().unwrap_or("white");
            let bg =
                self.header.vertical_bg_color.as_deref().unwrap_or_else(|| &self.header.bg_color);
            let header_vert_color = parse_color(bg, fg);

            table.modify(
                Segment::all().intersect(Rows::new(1..2)),
                BorderColor::filled(header_vert_color),
            );
        } else {
            // Default: match header background
            table.modify(Segment::all().intersect(Rows::new(1..2)), BorderColor::filled(header_bg));
        }

        // 4. Apply footer styling if enabled
        if let Some(footer) = &self.footer {
            if footer.enabled {
                // Determine footer colors (default to header colors)
                let footer_bg_str = footer.bg_color.as_deref().unwrap_or(&self.header.bg_color);
                let footer_fg_str = footer.fg_color.as_deref().unwrap_or(&self.header.fg_color);
                let footer_color = parse_color(footer_bg_str, footer_fg_str);
                let footer_bg = parse_bg_color(footer_bg_str);

                // Apply footer row colors
                table.modify(Rows::last(), footer_color);

                // Apply footer justification
                let footer_just_char = footer
                    .justification_char
                    .as_deref()
                    .or(Some(&self.header.justification_char))
                    .and_then(|s| s.chars().next())
                    .unwrap_or(' ');
                table.modify(
                    Rows::last(),
                    Justification::new(footer_just_char).color(footer_bg.clone()),
                );

                // Color footer vertical separators
                if footer.vertical_fg_color.is_some() || footer.vertical_bg_color.is_some() {
                    let fg = footer.vertical_fg_color.as_deref().unwrap_or("white");
                    let bg = footer.vertical_bg_color.as_deref().unwrap_or(footer_bg_str);
                    let footer_vert_color = parse_color(bg, fg);

                    table.modify(
                        Segment::all().intersect(Rows::last()),
                        BorderColor::filled(footer_vert_color),
                    );
                } else {
                    // Default: match footer background
                    table.modify(
                        Segment::all().intersect(Rows::last()),
                        BorderColor::filled(footer_bg),
                    );
                }
            }
        }
    }
}

/// Parse a color name string into a tabled Color (foreground)
fn parse_single_color(color: &str) -> Color {
    // Check if it's a hex color
    if let Some(rgb) = parse_hex_color(color) {
        return Color::rgb_fg(rgb.0, rgb.1, rgb.2);
    }

    match color.to_lowercase().as_str() {
        // Foreground colors
        "black" => Color::FG_BLACK,
        "red" => Color::FG_RED,
        "green" => Color::FG_GREEN,
        "yellow" => Color::FG_YELLOW,
        "blue" => Color::FG_BLUE,
        "magenta" => Color::FG_MAGENTA,
        "cyan" => Color::FG_CYAN,
        "white" => Color::FG_WHITE,
        "bright_black" | "gray" | "grey" => Color::FG_BRIGHT_BLACK,
        "bright_red" => Color::FG_BRIGHT_RED,
        "bright_green" => Color::FG_BRIGHT_GREEN,
        "bright_yellow" => Color::FG_BRIGHT_YELLOW,
        "bright_blue" => Color::FG_BRIGHT_BLUE,
        "bright_magenta" => Color::FG_BRIGHT_MAGENTA,
        "bright_cyan" => Color::FG_BRIGHT_CYAN,
        "bright_white" => Color::FG_BRIGHT_WHITE,
        _ => Color::FG_WHITE, // default
    }
}

/// Parse background and foreground colors and combine them
fn parse_color(bg: &str, fg: &str) -> Color {
    // Check if bg is a hex color
    let bg_color = if let Some(rgb) = parse_hex_color(bg) {
        Color::rgb_bg(rgb.0, rgb.1, rgb.2)
    } else {
        match bg.to_lowercase().as_str() {
            "black" => Color::BG_BLACK,
            "red" => Color::BG_RED,
            "green" => Color::BG_GREEN,
            "yellow" => Color::BG_YELLOW,
            "blue" => Color::BG_BLUE,
            "magenta" => Color::BG_MAGENTA,
            "cyan" => Color::BG_CYAN,
            "white" => Color::BG_WHITE,
            "bright_black" | "gray" | "grey" => Color::BG_BRIGHT_BLACK,
            "bright_red" => Color::BG_BRIGHT_RED,
            "bright_green" => Color::BG_BRIGHT_GREEN,
            "bright_yellow" => Color::BG_BRIGHT_YELLOW,
            "bright_blue" => Color::BG_BRIGHT_BLUE,
            "bright_magenta" => Color::BG_BRIGHT_MAGENTA,
            "bright_cyan" => Color::BG_BRIGHT_CYAN,
            "bright_white" => Color::BG_BRIGHT_WHITE,
            _ => Color::BG_BLACK, // default
        }
    };

    let fg_color = parse_single_color(fg);

    bg_color | fg_color
}

/// Parse just a background color
pub fn parse_bg_color(bg: &str) -> Color {
    // Check if it's a hex color
    if let Some(rgb) = parse_hex_color(bg) {
        return Color::rgb_bg(rgb.0, rgb.1, rgb.2);
    }

    match bg.to_lowercase().as_str() {
        "black" => Color::BG_BLACK,
        "red" => Color::BG_RED,
        "green" => Color::BG_GREEN,
        "yellow" => Color::BG_YELLOW,
        "blue" => Color::BG_BLUE,
        "magenta" => Color::BG_MAGENTA,
        "cyan" => Color::BG_CYAN,
        "white" => Color::BG_WHITE,
        "bright_black" | "gray" | "grey" => Color::BG_BRIGHT_BLACK,
        "bright_red" => Color::BG_BRIGHT_RED,
        "bright_green" => Color::BG_BRIGHT_GREEN,
        "bright_yellow" => Color::BG_BRIGHT_YELLOW,
        "bright_blue" => Color::BG_BRIGHT_BLUE,
        "bright_magenta" => Color::BG_BRIGHT_MAGENTA,
        "bright_cyan" => Color::BG_BRIGHT_CYAN,
        "bright_white" => Color::BG_BRIGHT_WHITE,
        _ => Color::BG_BLACK, // default
    }
}

/// Parse a hex color string (with or without #) into RGB components
/// Returns None if the string is not a valid hex color
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');

    // Support both 3-digit (#RGB) and 6-digit (#RRGGBB) formats
    if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        Some((r, g, b))
    } else if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== parse_hex_color tests =====

    #[test]
    fn test_parse_hex_color_six_digit_with_hash() {
        assert_eq!(parse_hex_color("#FF0000"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("#00FF00"), Some((0, 255, 0)));
        assert_eq!(parse_hex_color("#0000FF"), Some((0, 0, 255)));
        assert_eq!(parse_hex_color("#FFFFFF"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#000000"), Some((0, 0, 0)));
    }

    #[test]
    fn test_parse_hex_color_six_digit_without_hash() {
        assert_eq!(parse_hex_color("FF0000"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("00FF00"), Some((0, 255, 0)));
        assert_eq!(parse_hex_color("0000FF"), Some((0, 0, 255)));
    }

    #[test]
    fn test_parse_hex_color_three_digit_with_hash() {
        assert_eq!(parse_hex_color("#F00"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("#0F0"), Some((0, 255, 0)));
        assert_eq!(parse_hex_color("#00F"), Some((0, 0, 255)));
        assert_eq!(parse_hex_color("#FFF"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#000"), Some((0, 0, 0)));
    }

    #[test]
    fn test_parse_hex_color_three_digit_without_hash() {
        assert_eq!(parse_hex_color("F00"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("0F0"), Some((0, 255, 0)));
        assert_eq!(parse_hex_color("00F"), Some((0, 0, 255)));
    }

    #[test]
    fn test_parse_hex_color_lowercase() {
        assert_eq!(parse_hex_color("#ff0000"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("#f00"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("abcdef"), Some((171, 205, 239)));
    }

    #[test]
    fn test_parse_hex_color_mixed_case() {
        assert_eq!(parse_hex_color("#Ff0000"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("#AbCdEf"), Some((171, 205, 239)));
    }

    #[test]
    fn test_parse_hex_color_invalid_length() {
        assert_eq!(parse_hex_color("#FF"), None);
        assert_eq!(parse_hex_color("#FFFF"), None);
        assert_eq!(parse_hex_color("#FFFFFFF"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn test_parse_hex_color_invalid_characters() {
        assert_eq!(parse_hex_color("#GGGGGG"), None);
        assert_eq!(parse_hex_color("#ZZZZZZ"), None);
        assert_eq!(parse_hex_color("#12345G"), None);
        assert_eq!(parse_hex_color("notahex"), None);
    }

    #[test]
    fn test_parse_hex_color_real_world_colors() {
        // Oxur theme colors
        assert_eq!(parse_hex_color("#F97316"), Some((249, 115, 22)));
        assert_eq!(parse_hex_color("#D45500"), Some((212, 85, 0)));
        assert_eq!(parse_hex_color("#451A03"), Some((69, 26, 3)));
        assert_eq!(parse_hex_color("#FED7AA"), Some((254, 215, 170)));
        assert_eq!(parse_hex_color("#FDBA74"), Some((253, 186, 116)));
        assert_eq!(parse_hex_color("#803300"), Some((128, 51, 0)));
    }

    // ===== parse_single_color tests =====

    #[test]
    fn test_parse_single_color_basic_colors() {
        // Just verify these don't panic - we can't easily test Color equality
        parse_single_color("black");
        parse_single_color("red");
        parse_single_color("green");
        parse_single_color("yellow");
        parse_single_color("blue");
        parse_single_color("magenta");
        parse_single_color("cyan");
        parse_single_color("white");
    }

    #[test]
    fn test_parse_single_color_bright_colors() {
        parse_single_color("bright_black");
        parse_single_color("bright_red");
        parse_single_color("bright_green");
        parse_single_color("bright_yellow");
        parse_single_color("bright_blue");
        parse_single_color("bright_magenta");
        parse_single_color("bright_cyan");
        parse_single_color("bright_white");
    }

    #[test]
    fn test_parse_single_color_aliases() {
        parse_single_color("gray");
        parse_single_color("grey");
    }

    #[test]
    fn test_parse_single_color_case_insensitive() {
        parse_single_color("RED");
        parse_single_color("Green");
        parse_single_color("BLUE");
        parse_single_color("bright_RED");
    }

    #[test]
    fn test_parse_single_color_hex() {
        // Should handle hex colors
        parse_single_color("#FF0000");
        parse_single_color("#F00");
    }

    #[test]
    fn test_parse_single_color_unknown_defaults_to_white() {
        // Unknown colors should default to white
        parse_single_color("unknown");
        parse_single_color("notacolor");
    }

    // ===== parse_color tests =====

    #[test]
    fn test_parse_color_basic_combinations() {
        // Test combining background and foreground colors
        parse_color("black", "white");
        parse_color("red", "white");
        parse_color("blue", "yellow");
    }

    #[test]
    fn test_parse_color_hex_background() {
        parse_color("#FF0000", "white");
        parse_color("#F00", "black");
    }

    #[test]
    fn test_parse_color_hex_foreground() {
        parse_color("black", "#FFFFFF");
        parse_color("red", "#FFF");
    }

    #[test]
    fn test_parse_color_both_hex() {
        parse_color("#FF0000", "#FFFFFF");
        parse_color("#F00", "#FFF");
    }

    #[test]
    fn test_parse_color_case_insensitive() {
        parse_color("RED", "WHITE");
        parse_color("Blue", "Yellow");
    }

    // ===== parse_bg_color tests =====

    #[test]
    fn test_parse_bg_color_basic_colors() {
        parse_bg_color("black");
        parse_bg_color("red");
        parse_bg_color("green");
        parse_bg_color("yellow");
        parse_bg_color("blue");
        parse_bg_color("magenta");
        parse_bg_color("cyan");
        parse_bg_color("white");
    }

    #[test]
    fn test_parse_bg_color_bright_colors() {
        parse_bg_color("bright_black");
        parse_bg_color("bright_red");
        parse_bg_color("bright_green");
        parse_bg_color("bright_yellow");
        parse_bg_color("bright_blue");
        parse_bg_color("bright_magenta");
        parse_bg_color("bright_cyan");
        parse_bg_color("bright_white");
    }

    #[test]
    fn test_parse_bg_color_aliases() {
        parse_bg_color("gray");
        parse_bg_color("grey");
    }

    #[test]
    fn test_parse_bg_color_case_insensitive() {
        parse_bg_color("RED");
        parse_bg_color("Green");
        parse_bg_color("BLUE");
    }

    #[test]
    fn test_parse_bg_color_hex() {
        parse_bg_color("#FF0000");
        parse_bg_color("#F00");
    }

    #[test]
    fn test_parse_bg_color_unknown_defaults_to_black() {
        // Unknown colors should default to black
        parse_bg_color("unknown");
        parse_bg_color("notacolor");
    }

    // ===== TableStyleConfig deserialization tests =====

    #[test]
    fn test_deserialize_minimal_config() {
        let toml = r##"
            [table]
            padding_left = 1
            padding_right = 1
            padding_top = 0
            padding_bottom = 0

            [header]
            bg_color = "black"
            fg_color = "white"
            justification_char = " "

            [rows]
            colors = [
                { bg = "black", fg = "white" }
            ]

            [style]
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.table.padding_left, 1);
        assert_eq!(config.table.padding_right, 1);
        assert_eq!(config.header.bg_color, "black");
        assert_eq!(config.rows.colors.len(), 1);
    }

    #[test]
    fn test_deserialize_with_title_and_footer() {
        let toml = r##"
            [table]
            padding_left = 0
            padding_right = 0
            padding_top = 0
            padding_bottom = 0

            [title]
            enabled = true
            bg_color = "#FF0000"
            fg_color = "#FFFFFF"

            [header]
            bg_color = "blue"
            fg_color = "white"
            justification_char = " "

            [rows]
            colors = [
                { bg = "black", fg = "white" },
                { bg = "gray", fg = "white" }
            ]

            [style]

            [footer]
            enabled = true
            bg_color = "red"
            fg_color = "white"
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        assert!(config.title.is_some());
        assert!(config.footer.is_some());

        let title = config.title.unwrap();
        assert!(title.enabled);
        assert_eq!(title.bg_color.as_deref(), Some("#FF0000"));

        let footer = config.footer.unwrap();
        assert!(footer.enabled);
        assert_eq!(footer.bg_color.as_deref(), Some("red"));
    }

    #[test]
    fn test_deserialize_optional_fields() {
        let toml = r##"
            [table]
            padding_left = 0
            padding_right = 0
            padding_top = 0
            padding_bottom = 0

            [header]
            bg_color = "black"
            fg_color = "white"
            justification_char = " "
            vertical_char = "|"
            vertical_fg_color = "red"
            vertical_bg_color = "blue"

            [rows]
            colors = [{ bg = "black", fg = "white" }]
            justification_char = " "

            [style]
            vertical_char = "|"
            vertical_fg_color = "green"
            vertical_bg_color = "yellow"
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.header.vertical_char.as_deref(), Some("|"));
        assert_eq!(config.header.vertical_fg_color.as_deref(), Some("red"));
        assert_eq!(config.rows.justification_char.as_deref(), Some(" "));
        assert_eq!(config.style.vertical_char.as_deref(), Some("|"));
    }

    #[test]
    fn test_deserialize_without_optional_sections() {
        let toml = r##"
            [table]
            padding_left = 0
            padding_right = 0
            padding_top = 0
            padding_bottom = 0

            [header]
            bg_color = "black"
            fg_color = "white"
            justification_char = " "

            [rows]
            colors = [{ bg = "black", fg = "white" }]

            [style]
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        assert!(config.title.is_none());
        assert!(config.footer.is_none());
    }

    // ===== apply_to_table tests =====

    use tabled::{Table, Tabled};

    #[derive(Tabled)]
    struct TestRow {
        #[tabled(rename = "Col1")]
        col1: String,
        #[tabled(rename = "Col2")]
        col2: String,
    }

    #[test]
    fn test_apply_to_table_basic() {
        let config = TableStyleConfig::default();
        let data = vec![TestRow { col1: "A".into(), col2: "B".into() }];
        let mut table = Table::new(&data);

        config.apply_to_table::<TestRow>(&mut table);

        let output = table.to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_apply_to_table_no_title_or_footer() {
        let toml = r##"
            [table]
            padding_left = 1
            padding_right = 1
            padding_top = 0
            padding_bottom = 0

            [header]
            bg_color = "#FF0000"
            fg_color = "#FFFFFF"
            justification_char = " "

            [rows]
            colors = [
                { bg = "#000000", fg = "#FFFFFF" }
            ]

            [style]
            vertical_fg_color = "#00FF00"
            vertical_bg_color = "#0000FF"
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        let data = vec![TestRow { col1: "A".into(), col2: "B".into() }];
        let mut table = Table::new(&data);

        config.apply_to_table::<TestRow>(&mut table);

        let output = table.to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_apply_to_table_with_title_no_footer() {
        let toml = r##"
            [table]
            padding_left = 0
            padding_right = 0
            padding_top = 0
            padding_bottom = 0

            [title]
            enabled = true
            bg_color = "#FF0000"
            fg_color = "#FFFFFF"
            justification_char = " "
            vertical_fg_color = "#00FF00"
            vertical_bg_color = "#0000FF"

            [header]
            bg_color = "blue"
            fg_color = "white"
            justification_char = " "

            [rows]
            colors = [{ bg = "black", fg = "white" }]

            [style]
            vertical_fg_color = "red"
            vertical_bg_color = "green"
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        let data = vec![TestRow { col1: "A".into(), col2: "B".into() }];
        let mut table = Table::new(&data);

        config.apply_to_table::<TestRow>(&mut table);

        let output = table.to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_apply_to_table_with_footer_no_title() {
        let toml = r##"
            [table]
            padding_left = 0
            padding_right = 0
            padding_top = 0
            padding_bottom = 0

            [header]
            bg_color = "blue"
            fg_color = "white"
            justification_char = " "

            [rows]
            colors = [{ bg = "black", fg = "white" }]

            [style]
            vertical_fg_color = "red"
            vertical_bg_color = "green"

            [footer]
            enabled = true
            bg_color = "#FF0000"
            fg_color = "#FFFFFF"
            justification_char = " "
            vertical_fg_color = "#00FF00"
            vertical_bg_color = "#0000FF"
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        let data = vec![TestRow { col1: "A".into(), col2: "B".into() }];
        let mut table = Table::new(&data);

        config.apply_to_table::<TestRow>(&mut table);

        let output = table.to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_apply_to_table_with_both_title_and_footer() {
        let toml = r##"
            [table]
            padding_left = 0
            padding_right = 0
            padding_top = 0
            padding_bottom = 0

            [title]
            enabled = true
            bg_color = "#AA0000"
            fg_color = "#FFFFFF"

            [header]
            bg_color = "blue"
            fg_color = "white"
            justification_char = " "

            [rows]
            colors = [{ bg = "black", fg = "white" }]

            [style]
            vertical_fg_color = "red"
            vertical_bg_color = "green"

            [footer]
            enabled = true
            bg_color = "#00AA00"
            fg_color = "#FFFFFF"
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        let data = vec![TestRow { col1: "A".into(), col2: "B".into() }];
        let mut table = Table::new(&data);

        config.apply_to_table::<TestRow>(&mut table);

        let output = table.to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_apply_to_table_title_disabled() {
        let toml = r##"
            [table]
            padding_left = 0
            padding_right = 0
            padding_top = 0
            padding_bottom = 0

            [title]
            enabled = false
            bg_color = "#FF0000"
            fg_color = "#FFFFFF"

            [header]
            bg_color = "blue"
            fg_color = "white"
            justification_char = " "

            [rows]
            colors = [{ bg = "black", fg = "white" }]

            [style]
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        let data = vec![TestRow { col1: "A".into(), col2: "B".into() }];
        let mut table = Table::new(&data);

        config.apply_to_table::<TestRow>(&mut table);

        let output = table.to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_apply_to_table_footer_disabled() {
        let toml = r##"
            [table]
            padding_left = 0
            padding_right = 0
            padding_top = 0
            padding_bottom = 0

            [header]
            bg_color = "blue"
            fg_color = "white"
            justification_char = " "

            [rows]
            colors = [{ bg = "black", fg = "white" }]

            [style]

            [footer]
            enabled = false
            bg_color = "#FF0000"
            fg_color = "#FFFFFF"
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        let data = vec![TestRow { col1: "A".into(), col2: "B".into() }];
        let mut table = Table::new(&data);

        config.apply_to_table::<TestRow>(&mut table);

        let output = table.to_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_apply_to_table_with_empty_vertical_char() {
        let toml = r##"
            [table]
            padding_left = 0
            padding_right = 0
            padding_top = 0
            padding_bottom = 0

            [header]
            bg_color = "blue"
            fg_color = "white"
            justification_char = " "
            vertical_char = ""

            [rows]
            colors = [{ bg = "black", fg = "white" }]

            [style]
            vertical_char = ""
        "##;

        let config: TableStyleConfig = toml::from_str(toml).unwrap();
        let data = vec![TestRow { col1: "A".into(), col2: "B".into() }];
        let mut table = Table::new(&data);

        config.apply_to_table::<TestRow>(&mut table);

        let output = table.to_string();
        assert!(!output.is_empty());
    }
}
