//! oxurfmt - Format S-expression files
//!
//! A command-line tool for formatting S-expression files, following rustfmt conventions.

use anyhow::Result;
use oxur_cli::common::output;
use oxur_pretty::cli::Cli;
use oxur_pretty::commands;
use oxur_pretty::config::FormatConfig;

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    // Build configuration from flags
    let config = build_config(&cli)?;

    // Execute the appropriate command
    let result = if cli.check { execute_check(&cli, config) } else { execute_format(&cli, config) };

    // Handle errors
    if let Err(e) = result {
        output::error(&format!("{:#}", e));
        std::process::exit(2);
    }

    Ok(())
}

/// Execute the format command.
fn execute_format(cli: &Cli, config: FormatConfig) -> Result<()> {
    commands::format(cli, config)
}

/// Execute the check command.
///
/// Exits with code 1 if files need formatting, 0 if all are formatted.
fn execute_check(cli: &Cli, config: FormatConfig) -> Result<()> {
    let all_formatted = commands::check(cli, config)?;

    if !all_formatted {
        std::process::exit(1);
    }

    Ok(())
}

/// Build configuration from CLI flags.
///
/// Parses the --config flag (if present) and applies settings to the default config.
fn build_config(cli: &Cli) -> Result<FormatConfig> {
    let mut config = FormatConfig::default();

    if let Some(config_str) = &cli.config {
        config = parse_config_string(config_str, config)?;
    }

    Ok(config)
}

/// Parse configuration from a --config string.
///
/// Format: key1=val1,key2=val2
/// Example: max_width=120,tab_spaces=4
fn parse_config_string(config_str: &str, mut config: FormatConfig) -> Result<FormatConfig> {
    for pair in config_str.split(',') {
        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid config format '{}'. Expected 'key=value'", pair);
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        match key {
            "max_width" => {
                let width: usize = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid max_width value: {}", value))?;
                config = config.with_max_width(width)?;
            }
            "tab_spaces" => {
                let spaces: usize = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid tab_spaces value: {}", value))?;
                config = config.with_tab_spaces(spaces)?;
            }
            "align_keywords" => {
                let align: bool = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid align_keywords value: {}", value))?;
                config = config.with_align_keywords(align);
            }
            "compact_simple_values" => {
                let compact: bool = value.parse().map_err(|_| {
                    anyhow::anyhow!("Invalid compact_simple_values value: {}", value)
                })?;
                config = config.with_compact_simple_values(compact);
            }
            "max_inline_items" => {
                let max: usize = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid max_inline_items value: {}", value))?;
                config = config.with_max_inline_items(max);
            }
            _ => {
                anyhow::bail!("Unknown config key: {}", key);
            }
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_default_config() {
        let cli = Cli {
            files: vec![],
            check: false,
            emit: None,
            backup: false,
            config: None,
            color: None,
            files_with_diff: false,
            verbose: false,
            quiet: false,
        };

        let config = build_config(&cli).unwrap();
        assert_eq!(config, FormatConfig::default());
    }

    #[test]
    fn test_parse_config_string_max_width() {
        let config = FormatConfig::default();
        let result = parse_config_string("max_width=120", config).unwrap();
        assert_eq!(result.max_width, 120);
    }

    #[test]
    fn test_parse_config_string_tab_spaces() {
        let config = FormatConfig::default();
        let result = parse_config_string("tab_spaces=4", config).unwrap();
        assert_eq!(result.tab_spaces, 4);
    }

    #[test]
    fn test_parse_config_string_multiple() {
        let config = FormatConfig::default();
        let result = parse_config_string("max_width=80,tab_spaces=4", config).unwrap();
        assert_eq!(result.max_width, 80);
        assert_eq!(result.tab_spaces, 4);
    }

    #[test]
    fn test_parse_config_string_boolean() {
        let config = FormatConfig::default();
        let result = parse_config_string("align_keywords=false", config).unwrap();
        assert!(!result.align_keywords);
    }

    #[test]
    fn test_parse_config_string_invalid_format() {
        let config = FormatConfig::default();
        let result = parse_config_string("invalid", config);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_config_string_unknown_key() {
        let config = FormatConfig::default();
        let result = parse_config_string("unknown_key=value", config);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_config_string_invalid_value() {
        let config = FormatConfig::default();
        let result = parse_config_string("max_width=not_a_number", config);
        assert!(result.is_err());
    }
}
