use serde::Deserialize;
use tabled::{
    settings::{
        formatting::Justification,
        object::{Object, Rows, Segment},
        style::BorderColor,
        style::Style,
        themes::Colorization,
        Color, Padding,
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
/// To create a title row, add it as the FIRST data row:
/// ```
/// MyStruct { field1: "My Table Title".into(), field2: " ".into(), ... }
/// ```
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
/// To create a visual footer bar, add a row with spaces to your data:
/// ```
/// MyStruct { field1: " ".into(), field2: " ".into(), ... }
/// ```
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

        // Apply alternating row colors
        let row_colors: Vec<Color> = self
            .rows
            .colors
            .iter()
            .map(|rc| parse_color(&rc.bg, &rc.fg))
            .collect();
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
        table.modify(
            Rows::new(1..2),
            Justification::new(just_char).color(header_bg.clone()),
        );

        // Determine if footer is enabled (we need this for data row styling)
        let footer_enabled = self.footer.as_ref().map(|f| f.enabled).unwrap_or(false);

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
                    table.modify(
                        Segment::all().not(Rows::first()),
                        BorderColor::filled(vert_color),
                    );
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
            let bg = self
                .header
                .vertical_bg_color
                .as_deref()
                .unwrap_or_else(|| &self.header.bg_color);
            let header_vert_color = parse_color(bg, fg);

            table.modify(
                Segment::all().intersect(Rows::new(1..2)),
                BorderColor::filled(header_vert_color),
            );
        } else {
            // Default: match header background
            table.modify(
                Segment::all().intersect(Rows::new(1..2)),
                BorderColor::filled(header_bg),
            );
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
fn parse_bg_color(bg: &str) -> Color {
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
