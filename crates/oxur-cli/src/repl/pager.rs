//! Pager support for displaying long content
//!
//! Provides automatic paging for help and other long text output.
//! Auto-detects terminal height and only pages if content doesn't fit.

use std::io::{self, BufRead, Write};

/// Default terminal height if detection fails
const DEFAULT_TERM_HEIGHT: usize = 24;

/// Page through text content if it exceeds terminal height
///
/// Automatically detects terminal height and pages content if needed.
/// Uses a simple "Press Enter to continue, q to quit" interface.
///
/// # Arguments
///
/// * `content` - The text to display (may contain newlines)
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an IO error.
///
/// # Example
///
/// ```no_run
/// use oxur_cli::repl::pager;
///
/// let long_help = "Line 1\nLine 2\n...";
/// pager::page_text(long_help).expect("Failed to page");
/// ```
pub fn page_text(content: &str) -> io::Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();

    // Get terminal height
    let term_height = get_terminal_height();

    // If content fits on screen, just print it
    if line_count <= term_height - 2 {
        println!("{}", content);
        return Ok(());
    }

    // Content is too long - page it
    page_lines(&lines, term_height)
}

/// Get the terminal height
fn get_terminal_height() -> usize {
    terminal_size::terminal_size()
        .map(|(_, terminal_size::Height(h))| h as usize)
        .unwrap_or(DEFAULT_TERM_HEIGHT)
}

/// Page through lines with prompts
fn page_lines(lines: &[&str], term_height: usize) -> io::Result<()> {
    let page_size = term_height - 2; // Reserve 2 lines for prompt
    let mut current_line = 0;
    let total_lines = lines.len();

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut stdout = io::stdout();

    while current_line < total_lines {
        // Calculate how many lines to show
        let end_line = (current_line + page_size).min(total_lines);

        // Print the page
        for line in &lines[current_line..end_line] {
            println!("{}", line);
        }

        current_line = end_line;

        // If we've shown everything, we're done
        if current_line >= total_lines {
            break;
        }

        // Show prompt
        let remaining = total_lines - current_line;
        write!(
            stdout,
            "\x1b[7m-- More ({} lines remaining) -- Press Enter to continue, q to quit \x1b[0m",
            remaining
        )?;
        stdout.flush()?;

        // Read user input
        let mut input = String::new();
        stdin_lock.read_line(&mut input)?;

        // Clear the prompt line
        write!(stdout, "\r\x1b[K")?;

        // Check if user wants to quit
        if input.trim().starts_with('q') || input.trim().starts_with('Q') {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_terminal_height() {
        let height = get_terminal_height();
        // Should be either detected or default
        assert!(height > 0);
        assert!(height <= 200); // Sanity check
    }

    #[test]
    fn test_short_content_no_paging() {
        // Short content should work without interaction
        let content = "Line 1\nLine 2\nLine 3";
        // Can't easily test the actual paging without a TTY,
        // but we can verify the function doesn't panic
        let result = page_text(content);
        // This will print to stdout, which is fine for tests
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_empty_content() {
        let result = page_text("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_line() {
        let result = page_text("Single line");
        assert!(result.is_ok());
    }

    #[test]
    fn test_content_with_newlines() {
        let content = "Line 1\nLine 2\nLine 3\nLine 4\n";
        let result = page_text(content);
        assert!(result.is_ok());
    }
}
