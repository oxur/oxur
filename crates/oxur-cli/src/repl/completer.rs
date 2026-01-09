//! Command completion for Oxur REPL
//!
//! Provides tab completion for:
//! - Special commands: (help), (quit), (stats), (info)
//! - Help topics: basics, evaluation, keyboard, etc.
//! - Stats views: execution, cache, resources

use reedline::{Completer, Span, Suggestion};

/// Oxur REPL command completer
///
/// Implements reedline's `Completer` trait to provide tab completion
/// for REPL commands, help topics, and stats views.
#[derive(Clone)]
pub struct OxurCompleter;

impl OxurCompleter {
    /// Create a new Oxur completer
    pub fn new() -> Self {
        Self
    }

    /// Get list of special commands
    fn special_commands() -> Vec<&'static str> {
        vec!["(help)", "(quit)", "(q)", "(exit)", "(info)", "(stats)", "(sessions)", "(clear)", "(banner)"]
    }

    /// Get list of help topics with descriptions
    fn help_topics() -> Vec<(&'static str, &'static str)> {
        vec![
            ("basics", "Basic REPL usage and syntax"),
            ("evaluation", "How expressions are evaluated"),
            ("keyboard", "Keyboard shortcuts and navigation"),
            ("sessions", "Session management and history"),
            ("commands", "Special commands reference"),
            ("modes", "REPL modes (interactive, server, connect)"),
            ("performance", "Performance tips and optimization"),
            ("stats", "Statistics and metrics"),
        ]
    }

    /// Get list of stats views with descriptions
    fn stats_views() -> Vec<(&'static str, &'static str)> {
        vec![
            ("execution", "Execution tier statistics"),
            ("cache", "Cache hit rates and performance"),
            ("resources", "Memory and system resources"),
        ]
    }

    /// Find completions for the given partial input with descriptions
    fn find_completions(&self, partial: &str) -> Vec<(String, Option<String>)> {
        let mut completions = Vec::new();

        // Help topics: "(help <partial>"
        if let Some(help_prefix) = partial.strip_prefix("(help ") {
            let topic_partial = help_prefix.trim();
            for (topic, description) in Self::help_topics() {
                if topic.starts_with(topic_partial) {
                    completions.push((format!("(help {})", topic), Some(description.to_string())));
                }
            }
            return completions;
        }

        // Stats views: "(stats <partial>"
        if let Some(stats_prefix) = partial.strip_prefix("(stats ") {
            let view_partial = stats_prefix.trim();
            for (view, description) in Self::stats_views() {
                if view.starts_with(view_partial) {
                    completions.push((format!("(stats {})", view), Some(description.to_string())));
                }
            }
            return completions;
        }

        // Special commands (only if no space yet)
        if !partial.contains(' ') {
            for cmd in Self::special_commands() {
                if cmd.starts_with(partial) {
                    completions.push((cmd.to_string(), None));
                }
            }
        }

        completions
    }
}

impl Default for OxurCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for OxurCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let partial = &line[..pos];

        self.find_completions(partial)
            .into_iter()
            .map(|(value, description)| Suggestion {
                value,
                description,
                style: None,
                extra: None,
                span: Span::new(0, pos),
                append_whitespace: false,
                match_indices: None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_help_command() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(h", 2);
        assert!(suggestions.iter().any(|s| s.value == "(help)"));
    }

    #[test]
    fn test_complete_quit_commands() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(q", 2);
        let values: Vec<_> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"(quit)"));
        assert!(values.contains(&"(q)"));
    }

    #[test]
    fn test_complete_help_topic_basics() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(help ba", 8);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "(help basics)");
    }

    #[test]
    fn test_complete_help_topic_partial() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(help ev", 8);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "(help evaluation)");
    }

    #[test]
    fn test_complete_help_all_topics() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(help ", 6);
        assert_eq!(suggestions.len(), 8); // All 8 help topics
        let values: Vec<_> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"(help basics)"));
        assert!(values.contains(&"(help evaluation)"));
        assert!(values.contains(&"(help keyboard)"));
        assert!(values.contains(&"(help sessions)"));
        assert!(values.contains(&"(help commands)"));
        assert!(values.contains(&"(help modes)"));
        assert!(values.contains(&"(help performance)"));
        assert!(values.contains(&"(help stats)"));
    }

    #[test]
    fn test_complete_stats_command() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(sta", 4);
        assert!(suggestions.iter().any(|s| s.value == "(stats)"));
    }

    #[test]
    fn test_complete_stats_view_execution() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(stats ex", 9);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "(stats execution)");
    }

    #[test]
    fn test_complete_stats_view_cache() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(stats ca", 9);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "(stats cache)");
    }

    #[test]
    fn test_complete_stats_view_resources() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(stats re", 9);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "(stats resources)");
    }

    #[test]
    fn test_complete_stats_all_views() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(stats ", 7);
        assert_eq!(suggestions.len(), 3); // All 3 stats views
        let values: Vec<_> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(values.contains(&"(stats execution)"));
        assert!(values.contains(&"(stats cache)"));
        assert!(values.contains(&"(stats resources)"));
    }

    #[test]
    fn test_no_completion_for_regular_code() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(+ 1 2", 6);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_no_completion_after_space_in_regular_code() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(deffn foo ", 11);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_info_command_completion() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(inf", 4);
        assert!(suggestions.iter().any(|s| s.value == "(info)"));
    }

    #[test]
    fn test_exit_command_completion() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(exi", 4);
        assert!(suggestions.iter().any(|s| s.value == "(exit)"));
    }

    #[test]
    fn test_help_topic_has_description() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(help ba", 8);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "(help basics)");
        assert_eq!(suggestions[0].description, Some("Basic REPL usage and syntax".to_string()));
    }

    #[test]
    fn test_stats_view_has_description() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(stats ex", 9);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "(stats execution)");
        assert_eq!(suggestions[0].description, Some("Execution tier statistics".to_string()));
    }

    #[test]
    fn test_special_commands_no_description() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(h", 2);
        assert!(suggestions.iter().any(|s| s.value == "(help)"));
        let help_suggestion = suggestions.iter().find(|s| s.value == "(help)").unwrap();
        assert_eq!(help_suggestion.description, None);
    }

    #[test]
    fn test_clear_command_completion() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(cle", 4);
        assert!(suggestions.iter().any(|s| s.value == "(clear)"));
    }

    #[test]
    fn test_banner_command_completion() {
        let mut completer = OxurCompleter::new();
        let suggestions = completer.complete("(ban", 4);
        assert!(suggestions.iter().any(|s| s.value == "(banner)"));
    }
}
