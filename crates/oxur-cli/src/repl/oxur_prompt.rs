//! Custom prompt with multi-line support
//!
//! Provides the Oxur REPL prompt with support for continuation lines
//! when entering multi-line S-expressions.

use reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus};
use std::borrow::Cow;

/// Custom prompt for Oxur REPL
///
/// Implements reedline's `Prompt` trait to provide:
/// - Custom primary prompt (e.g., "oxur> ")
/// - Continuation prompt for multi-line input (e.g., "....> ")
/// - History search indicator
pub struct OxurPrompt {
    primary: String,
    continuation: String,
}

impl OxurPrompt {
    /// Create a new Oxur prompt
    ///
    /// # Arguments
    ///
    /// * `primary` - The main prompt string (e.g., "oxur> ")
    /// * `continuation` - The continuation prompt for multi-line input (e.g., "....> ")
    pub fn new(primary: String, continuation: String) -> Self {
        Self {
            primary,
            continuation,
        }
    }
}

impl Prompt for OxurPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.primary)
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.continuation)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        let status = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            status, history_search.term
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reedline::PromptEditMode;

    #[test]
    fn test_new_prompt() {
        let prompt = OxurPrompt::new("oxur> ".to_string(), "....> ".to_string());
        assert_eq!(prompt.render_prompt_left(), "oxur> ");
        assert_eq!(prompt.render_prompt_multiline_indicator(), "....> ");
    }

    #[test]
    fn test_right_prompt_empty() {
        let prompt = OxurPrompt::new("oxur> ".to_string(), "....> ".to_string());
        assert_eq!(prompt.render_prompt_right(), "");
    }

    #[test]
    fn test_prompt_indicator_empty() {
        let prompt = OxurPrompt::new("oxur> ".to_string(), "....> ".to_string());
        assert_eq!(
            prompt.render_prompt_indicator(PromptEditMode::Default),
            ""
        );
    }

    #[test]
    fn test_history_search_passing() {
        let prompt = OxurPrompt::new("oxur> ".to_string(), "....> ".to_string());
        let search = PromptHistorySearch {
            status: PromptHistorySearchStatus::Passing,
            term: "test".to_string(),
        };
        let result = prompt.render_prompt_history_search_indicator(search);
        assert!(result.contains("reverse-search"));
        assert!(result.contains("test"));
        assert!(!result.contains("failing"));
    }

    #[test]
    fn test_history_search_failing() {
        let prompt = OxurPrompt::new("oxur> ".to_string(), "....> ".to_string());
        let search = PromptHistorySearch {
            status: PromptHistorySearchStatus::Failing,
            term: "test".to_string(),
        };
        let result = prompt.render_prompt_history_search_indicator(search);
        assert!(result.contains("failing"));
        assert!(result.contains("reverse-search"));
        assert!(result.contains("test"));
    }

    #[test]
    fn test_custom_prompts() {
        let prompt = OxurPrompt::new(">>> ".to_string(), "... ".to_string());
        assert_eq!(prompt.render_prompt_left(), ">>> ");
        assert_eq!(prompt.render_prompt_multiline_indicator(), "... ");
    }
}
