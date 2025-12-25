//! Interactive prompting for user input

use anyhow::Result;
use std::io::{self, Write};

/// Prompt user for input with a default value
pub fn prompt_with_default(message: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", message, default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// Prompt user for input (required)
pub fn prompt_required(message: &str) -> Result<String> {
    loop {
        print!("{}: ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed = input.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }

        println!("This field is required. Please enter a value.");
    }
}

/// Prompt user to select from options
pub fn prompt_select(message: &str, options: &[&str], default_idx: usize) -> Result<String> {
    println!("{}", message);
    for (idx, opt) in options.iter().enumerate() {
        let marker = if idx == default_idx { "*" } else { " " };
        println!("  {}{}) {}", marker, idx + 1, opt);
    }

    print!("Select [{}]: ", default_idx + 1);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(options[default_idx].to_string());
    }

    if let Ok(idx) = trimmed.parse::<usize>() {
        if idx > 0 && idx <= options.len() {
            return Ok(options[idx - 1].to_string());
        }
    }

    println!("Invalid selection, using default.");
    Ok(options[default_idx].to_string())
}

/// Prompt user for yes/no confirmation
pub fn prompt_confirm(message: &str, default: bool) -> Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", message, default_str);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        Ok(default)
    } else {
        Ok(trimmed.starts_with('y'))
    }
}
