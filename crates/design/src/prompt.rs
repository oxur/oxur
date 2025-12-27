//! Interactive prompting for user input

use anyhow::Result;
use std::io::{self, BufRead, Write};

/// Prompt user for input with a default value (generic version for testing)
fn prompt_with_default_impl<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    message: &str,
    default: &str,
) -> Result<String> {
    write!(writer, "{} [{}]: ", message, default)?;
    writer.flush()?;

    let mut input = String::new();
    reader.read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// Prompt user for input with a default value
pub fn prompt_with_default(message: &str, default: &str) -> Result<String> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();
    prompt_with_default_impl(&mut reader, &mut stdout, message, default)
}

/// Prompt user for input (required) - generic version for testing
fn prompt_required_impl<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    message: &str,
) -> Result<String> {
    loop {
        write!(writer, "{}: ", message)?;
        writer.flush()?;

        let mut input = String::new();
        reader.read_line(&mut input)?;

        let trimmed = input.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }

        writeln!(writer, "This field is required. Please enter a value.")?;
    }
}

/// Prompt user for input (required)
pub fn prompt_required(message: &str) -> Result<String> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();
    prompt_required_impl(&mut reader, &mut stdout, message)
}

/// Prompt user to select from options - generic version for testing
fn prompt_select_impl<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    message: &str,
    options: &[&str],
    default_idx: usize,
) -> Result<String> {
    writeln!(writer, "{}", message)?;
    for (idx, opt) in options.iter().enumerate() {
        let marker = if idx == default_idx { "*" } else { " " };
        writeln!(writer, "  {}{}) {}", marker, idx + 1, opt)?;
    }

    write!(writer, "Select [{}]: ", default_idx + 1)?;
    writer.flush()?;

    let mut input = String::new();
    reader.read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(options[default_idx].to_string());
    }

    if let Ok(idx) = trimmed.parse::<usize>() {
        if idx > 0 && idx <= options.len() {
            return Ok(options[idx - 1].to_string());
        }
    }

    writeln!(writer, "Invalid selection, using default.")?;
    Ok(options[default_idx].to_string())
}

/// Prompt user to select from options
pub fn prompt_select(message: &str, options: &[&str], default_idx: usize) -> Result<String> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();
    prompt_select_impl(&mut reader, &mut stdout, message, options, default_idx)
}

/// Prompt user for yes/no confirmation - generic version for testing
fn prompt_confirm_impl<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    message: &str,
    default: bool,
) -> Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    write!(writer, "{} [{}]: ", message, default_str)?;
    writer.flush()?;

    let mut input = String::new();
    reader.read_line(&mut input)?;

    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        Ok(default)
    } else {
        Ok(trimmed.starts_with('y'))
    }
}

/// Prompt user for yes/no confirmation
pub fn prompt_confirm(message: &str, default: bool) -> Result<bool> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();
    prompt_confirm_impl(&mut reader, &mut stdout, message, default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_prompt_with_default_uses_default_on_empty() {
        let input = b"\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result =
            prompt_with_default_impl(&mut reader, &mut writer, "Name", "default_name").unwrap();
        assert_eq!(result, "default_name");

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Name [default_name]:"));
    }

    #[test]
    fn test_prompt_with_default_uses_provided_value() {
        let input = b"custom_value\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result =
            prompt_with_default_impl(&mut reader, &mut writer, "Name", "default_name").unwrap();
        assert_eq!(result, "custom_value");
    }

    #[test]
    fn test_prompt_with_default_trims_whitespace() {
        let input = b"  custom_value  \n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result =
            prompt_with_default_impl(&mut reader, &mut writer, "Name", "default_name").unwrap();
        assert_eq!(result, "custom_value");
    }

    #[test]
    fn test_prompt_required_accepts_valid_input() {
        let input = b"required_value\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_required_impl(&mut reader, &mut writer, "Required field").unwrap();
        assert_eq!(result, "required_value");
    }

    #[test]
    fn test_prompt_required_retries_on_empty() {
        let input = b"\n\nvalid_value\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_required_impl(&mut reader, &mut writer, "Required field").unwrap();
        assert_eq!(result, "valid_value");

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("This field is required"));
    }

    #[test]
    fn test_prompt_required_trims_whitespace() {
        let input = b"  value  \n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_required_impl(&mut reader, &mut writer, "Field").unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn test_prompt_select_uses_default_on_empty() {
        let input = b"\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();
        let options = vec!["option1", "option2", "option3"];

        let result = prompt_select_impl(&mut reader, &mut writer, "Choose", &options, 1).unwrap();
        assert_eq!(result, "option2");

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("*2) option2"));
    }

    #[test]
    fn test_prompt_select_accepts_valid_selection() {
        let input = b"1\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();
        let options = vec!["option1", "option2", "option3"];

        let result = prompt_select_impl(&mut reader, &mut writer, "Choose", &options, 1).unwrap();
        assert_eq!(result, "option1");
    }

    #[test]
    fn test_prompt_select_accepts_last_option() {
        let input = b"3\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();
        let options = vec!["option1", "option2", "option3"];

        let result = prompt_select_impl(&mut reader, &mut writer, "Choose", &options, 0).unwrap();
        assert_eq!(result, "option3");
    }

    #[test]
    fn test_prompt_select_uses_default_on_invalid_number() {
        let input = b"99\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();
        let options = vec!["option1", "option2"];

        let result = prompt_select_impl(&mut reader, &mut writer, "Choose", &options, 0).unwrap();
        assert_eq!(result, "option1");

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Invalid selection"));
    }

    #[test]
    fn test_prompt_select_uses_default_on_zero() {
        let input = b"0\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();
        let options = vec!["option1", "option2"];

        let result = prompt_select_impl(&mut reader, &mut writer, "Choose", &options, 1).unwrap();
        assert_eq!(result, "option2");

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Invalid selection"));
    }

    #[test]
    fn test_prompt_select_uses_default_on_non_numeric() {
        let input = b"abc\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();
        let options = vec!["option1", "option2"];

        let result = prompt_select_impl(&mut reader, &mut writer, "Choose", &options, 0).unwrap();
        assert_eq!(result, "option1");
    }

    #[test]
    fn test_prompt_confirm_default_true_on_empty() {
        let input = b"\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", true).unwrap();
        assert!(result);

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("[Y/n]"));
    }

    #[test]
    fn test_prompt_confirm_default_false_on_empty() {
        let input = b"\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", false).unwrap();
        assert!(!result);

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("[y/N]"));
    }

    #[test]
    fn test_prompt_confirm_yes_lowercase() {
        let input = b"yes\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", false).unwrap();
        assert!(result);
    }

    #[test]
    fn test_prompt_confirm_yes_uppercase() {
        let input = b"YES\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", false).unwrap();
        assert!(result);
    }

    #[test]
    fn test_prompt_confirm_y() {
        let input = b"y\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", false).unwrap();
        assert!(result);
    }

    #[test]
    fn test_prompt_confirm_no_lowercase() {
        let input = b"no\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", true).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_prompt_confirm_no_uppercase() {
        let input = b"NO\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", true).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_prompt_confirm_n() {
        let input = b"n\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", true).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_prompt_confirm_arbitrary_text_starting_with_y() {
        let input = b"yeah\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", false).unwrap();
        assert!(result);
    }

    #[test]
    fn test_prompt_confirm_arbitrary_text_not_starting_with_y() {
        let input = b"nope\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = prompt_confirm_impl(&mut reader, &mut writer, "Confirm?", true).unwrap();
        assert!(!result);
    }
}
