//! System information display for the REPL
//!
//! Provides display functions for system metadata using OxurTable formatting.
//! Core data capture is in oxur-repl::metadata; this module handles presentation.

use crate::table::{OxurTable, Tabled};
use oxur_repl::metadata::SystemMetadata;

/// Show system information in a formatted table
///
/// Displays system metadata captured at startup including versions,
/// platform info, process details, and uptime.
pub fn show_system_info(metadata: &SystemMetadata, _color_enabled: bool) -> String {
    let mut output = String::new();

    output.push_str(&header("System Information", _color_enabled));
    output.push('\n');

    #[derive(Tabled)]
    struct InfoRow {
        #[tabled(rename = "Property")]
        property: String,
        #[tabled(rename = "Value")]
        value: String,
    }

    // Build the info rows
    let rows = vec![
        InfoRow { property: "Oxur Version".to_string(), value: metadata.oxur_version.clone() },
        InfoRow { property: "Rust Version".to_string(), value: metadata.rust_version.clone() },
        InfoRow { property: "Cargo Version".to_string(), value: metadata.cargo_version.clone() },
        InfoRow {
            property: "Operating System".to_string(),
            value: format!("{} {}", metadata.os_name, metadata.os_version),
        },
        InfoRow { property: "Architecture".to_string(), value: metadata.arch.clone() },
        InfoRow { property: "Hostname".to_string(), value: metadata.hostname.clone() },
        InfoRow { property: "Process ID".to_string(), value: metadata.pid.to_string() },
        InfoRow {
            property: "Working Directory".to_string(),
            value: metadata.cwd.display().to_string(),
        },
        InfoRow { property: "Uptime".to_string(), value: format_uptime(metadata.uptime_seconds()) },
    ];

    output.push_str(&OxurTable::new(rows).with_title("SYSTEM INFO").with_footer().render());
    output.push('\n');

    output
}

/// Format header with optional color
fn header(text: &str, color_enabled: bool) -> String {
    if color_enabled {
        format!("\x1b[1;36m{}\x1b[0m\n{}\n", text, "═".repeat(text.len()))
    } else {
        format!("{}\n{}\n", text, "=".repeat(text.len()))
    }
}

/// Format uptime in human-readable form
fn format_uptime(seconds: f64) -> String {
    let secs = seconds as u64;
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;

    if secs >= DAY {
        let days = secs / DAY;
        let hours = (secs % DAY) / HOUR;
        format!("{}d {}h", days, hours)
    } else if secs >= HOUR {
        let hours = secs / HOUR;
        let mins = (secs % HOUR) / MINUTE;
        format!("{}h {}m", hours, mins)
    } else if secs >= MINUTE {
        let mins = secs / MINUTE;
        let s = secs % MINUTE;
        format!("{}m {}s", mins, s)
    } else {
        format!("{:.1}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_seconds() {
        assert_eq!(format_uptime(0.0), "0.0s");
        assert_eq!(format_uptime(5.5), "5.5s");
        assert_eq!(format_uptime(30.0), "30.0s");
    }

    #[test]
    fn test_format_uptime_minutes() {
        assert_eq!(format_uptime(60.0), "1m 0s");
        assert_eq!(format_uptime(90.0), "1m 30s");
        assert_eq!(format_uptime(3599.0), "59m 59s");
    }

    #[test]
    fn test_format_uptime_hours() {
        assert_eq!(format_uptime(3600.0), "1h 0m");
        assert_eq!(format_uptime(7200.0), "2h 0m");
        assert_eq!(format_uptime(5400.0), "1h 30m");
    }

    #[test]
    fn test_format_uptime_days() {
        assert_eq!(format_uptime(86400.0), "1d 0h");
        assert_eq!(format_uptime(172800.0), "2d 0h");
        assert_eq!(format_uptime(90000.0), "1d 1h");
    }

    #[test]
    fn test_show_system_info() {
        let metadata = SystemMetadata::capture();
        let output = show_system_info(&metadata, false);

        // Should contain expected sections
        assert!(output.contains("System Information"));
        assert!(output.contains("SYSTEM INFO"));
        assert!(output.contains("Oxur Version"));
        assert!(output.contains("Rust Version"));
        assert!(output.contains("Process ID"));
        assert!(output.contains("Working Directory"));
        assert!(output.contains("Uptime"));
    }

    #[test]
    fn test_show_system_info_with_color() {
        let metadata = SystemMetadata::capture();
        let output = show_system_info(&metadata, true);

        // Should contain ANSI escape codes
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn test_header_with_color() {
        let header = header("Test", true);
        assert!(header.contains("\x1b[1;36m"));
        assert!(header.contains("Test"));
    }

    #[test]
    fn test_header_without_color() {
        let header = header("Test", false);
        assert!(!header.contains("\x1b["));
        assert!(header.contains("Test"));
        assert!(header.contains("===="));
    }
}
