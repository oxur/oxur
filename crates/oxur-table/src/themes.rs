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

[header]
bg_color = "#D45500"
fg_color = "#451A03"
justification_char = " "
vertical_char = " "
vertical_bg_color = "#D45500"
vertical_fg_color = "#D45500"

[rows]
colors = [
    { bg = "#451A03", fg = "#FED7AA" },
    { bg = "#451A03", fg = "#FDBA74" }
]

[style]
vertical_bg_color = "#451A03"
vertical_fg_color = "#D45500"

[footer]
enabled = true
bg_color = "#803300"
fg_color = "#803300"
vertical_bg_color = "#803300"
vertical_fg_color = "#803300"
"##;

impl Default for TableStyleConfig {
    fn default() -> Self {
        toml::from_str(OXUR_DEFAULT).expect("Default Oxur theme should be valid TOML")
    }
}
