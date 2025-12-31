//! Embedded default themes

use crate::config::TableStyleConfig;

/// The default Oxur theme with warm orange sunset colors
pub const OXUR_DEFAULT: &str = r##"[table]
padding_left = 0
padding_right = 0
padding_top = 0
padding_bottom = 0

[title]
enabled = true
bg_color = "#F97316"
fg_color = "#451A03"
justification_char = " "
vertical_fg_color = "#F97316"

[header]
bg_color = "#D45500"
fg_color = "#451A03"
justification_char = " "
vertical_char = "│"
vertical_bg_color = "#D45500"
vertical_fg_color = "#D45500"

[rows]
colors = [
    { bg = "#451A03", fg = "#FED7AA" },
    { bg = "#451A03", fg = "#FDBA74" }
]
justification_char = " "

[style]
vertical_bg_color = "#451A03"
vertical_fg_color = "#803300"

[footer]
enabled = true
bg_color = "#803300"
fg_color = "#D45500"
vertical_bg_color = "#803300"
vertical_fg_color = "#803300"
"##;

impl Default for TableStyleConfig {
    fn default() -> Self {
        toml::from_str(OXUR_DEFAULT).expect("Default Oxur theme should be valid TOML")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_is_valid_toml() {
        // The default theme should parse without panicking
        let config = TableStyleConfig::default();

        // Verify basic structure
        assert_eq!(config.table.padding_left, 0);
        assert_eq!(config.table.padding_right, 0);
        assert_eq!(config.table.padding_top, 0);
        assert_eq!(config.table.padding_bottom, 0);
    }

    #[test]
    fn test_default_theme_has_title_config() {
        let config = TableStyleConfig::default();

        assert!(config.title.is_some());
        let title = config.title.unwrap();
        assert!(title.enabled);
        assert_eq!(title.bg_color.as_deref(), Some("#F97316"));
        assert_eq!(title.fg_color.as_deref(), Some("#451A03"));
    }

    #[test]
    fn test_default_theme_has_header_config() {
        let config = TableStyleConfig::default();

        assert_eq!(config.header.bg_color, "#D45500");
        assert_eq!(config.header.fg_color, "#451A03");
        assert_eq!(config.header.justification_char, " ");
    }

    #[test]
    fn test_default_theme_has_row_colors() {
        let config = TableStyleConfig::default();

        assert_eq!(config.rows.colors.len(), 2);
        assert_eq!(config.rows.colors[0].bg, "#451A03");
        assert_eq!(config.rows.colors[0].fg, "#FED7AA");
        assert_eq!(config.rows.colors[1].bg, "#451A03");
        assert_eq!(config.rows.colors[1].fg, "#FDBA74");
    }

    #[test]
    fn test_default_theme_has_footer_config() {
        let config = TableStyleConfig::default();

        assert!(config.footer.is_some());
        let footer = config.footer.unwrap();
        assert!(footer.enabled);
        assert_eq!(footer.bg_color.as_deref(), Some("#803300"));
        assert_eq!(footer.fg_color.as_deref(), Some("#D45500"));
    }

    #[test]
    fn test_oxur_default_const_parses() {
        // Ensure the constant string itself is valid TOML
        let result: Result<TableStyleConfig, _> = toml::from_str(OXUR_DEFAULT);
        assert!(result.is_ok());
    }
}
