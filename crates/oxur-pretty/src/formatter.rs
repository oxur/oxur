//! S-expression formatter - the core formatting engine.

use crate::config::FormatConfig;
use crate::parser::SExpr;
use crate::rules::FormattingRules;

/// The formatter transforms S-expressions into pretty-printed strings.
pub struct Formatter {
    config: FormatConfig,
    rules: FormattingRules,
}

impl Formatter {
    /// Create a new formatter with the given configuration.
    pub fn new(config: FormatConfig) -> Self {
        let rules = FormattingRules::new(&config);
        Self { config, rules }
    }

    /// Format an S-expression to a string.
    /// This is the main entry point for formatting.
    pub fn format(&self, expr: &SExpr) -> String {
        let mut output = String::new();
        self.format_expr(expr, 0, &mut output);
        output
    }

    /// Format an S-expression at a given indentation level.
    fn format_expr(&self, expr: &SExpr, indent: usize, output: &mut String) {
        match expr {
            SExpr::Atom(s) => {
                output.push_str(s);
            }
            SExpr::List(items) => {
                self.format_list(items, indent, output);
            }
        }
    }

    /// Format a list using the appropriate strategy.
    fn format_list(&self, items: &[SExpr], indent: usize, output: &mut String) {
        output.push('(');

        if items.is_empty() {
            output.push(')');
            return;
        }

        // Decision tree for formatting strategy
        if self.rules.should_inline(&SExpr::List(items.to_vec())) {
            // Format everything on one line
            self.format_inline(items, output);
        } else if self.rules.should_keyword_align(items) {
            // Format with keyword-aligned multiline
            self.format_keyword_aligned(items, indent, output);
        } else {
            // Format with general multiline
            self.format_multiline_smart(items, indent, output);
        }

        output.push(')');
    }

    /// Format all items on a single line with spaces between them.
    /// Example: `(a b c)` or `(Span :lo 0 :hi 10)`
    fn format_inline(&self, items: &[SExpr], output: &mut String) {
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                output.push(' ');
            }
            self.format_expr(item, 0, output);
        }
    }

    /// Format with keyword-aligned multiline structure.
    /// Example:
    /// ```lisp
    /// (TypeName
    ///   :key1 value1
    ///   :key2 value2)
    /// ```
    fn format_keyword_aligned(&self, items: &[SExpr], indent: usize, output: &mut String) {
        let new_indent = indent + self.config.tab_spaces;

        // First item (type name) stays on same line as opening paren
        if let Some(first) = items.first() {
            self.format_expr(first, new_indent, output);
        }

        // Process keyword-value pairs
        let mut i = 1;
        while i < items.len() {
            output.push('\n');
            self.write_indent(new_indent, output);

            // Format keyword
            if let Some(keyword) = items.get(i) {
                self.format_expr(keyword, new_indent, output);
                i += 1;
            }

            // Format value (if present)
            if i < items.len() {
                output.push(' ');
                let value = &items[i];

                // If value is simple, put it on same line as keyword
                // Otherwise, it gets its own indented line
                self.format_expr(value, new_indent, output);
                i += 1;
            }
        }
    }

    /// Format with general multiline structure.
    /// Each item gets its own line with proper indentation.
    /// Example:
    /// ```lisp
    /// (list
    ///   item1
    ///   item2
    ///   item3)
    /// ```
    fn format_multiline_smart(&self, items: &[SExpr], indent: usize, output: &mut String) {
        let new_indent = indent + self.config.tab_spaces;

        // First item on same line as opening paren
        if let Some(first) = items.first() {
            self.format_expr(first, new_indent, output);
        }

        // Rest of items on their own lines
        for item in items.iter().skip(1) {
            output.push('\n');
            self.write_indent(new_indent, output);
            self.format_expr(item, new_indent, output);
        }
    }

    /// Write indentation spaces.
    fn write_indent(&self, indent: usize, output: &mut String) {
        for _ in 0..indent {
            output.push(' ');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_atom() {
        let formatter = Formatter::new(FormatConfig::default());
        let atom = SExpr::Atom("hello".to_string());
        assert_eq!(formatter.format(&atom), "hello");
    }

    #[test]
    fn test_format_keyword() {
        let formatter = Formatter::new(FormatConfig::default());
        let keyword = SExpr::Atom(":name".to_string());
        assert_eq!(formatter.format(&keyword), ":name");
    }

    #[test]
    fn test_format_string() {
        let formatter = Formatter::new(FormatConfig::default());
        let string = SExpr::Atom(r#""hello world""#.to_string());
        assert_eq!(formatter.format(&string), r#""hello world""#);
    }

    #[test]
    fn test_format_empty_list() {
        let formatter = Formatter::new(FormatConfig::default());
        let list = SExpr::List(vec![]);
        assert_eq!(formatter.format(&list), "()");
    }

    #[test]
    fn test_format_simple_inline_list() {
        let formatter = Formatter::new(FormatConfig::default());
        let list = SExpr::List(vec![
            SExpr::Atom("a".to_string()),
            SExpr::Atom("b".to_string()),
            SExpr::Atom("c".to_string()),
        ]);
        assert_eq!(formatter.format(&list), "(a b c)");
    }

    #[test]
    fn test_format_compact_span() {
        let formatter = Formatter::new(FormatConfig::default());
        let span = SExpr::List(vec![
            SExpr::Atom("Span".to_string()),
            SExpr::Atom(":lo".to_string()),
            SExpr::Atom("0".to_string()),
            SExpr::Atom(":hi".to_string()),
            SExpr::Atom("10".to_string()),
        ]);
        assert_eq!(formatter.format(&span), "(Span :lo 0 :hi 10)");
    }

    #[test]
    fn test_format_compact_with_nested() {
        let formatter = Formatter::new(FormatConfig::default());
        let ident = SExpr::List(vec![
            SExpr::Atom("Ident".to_string()),
            SExpr::Atom(":name".to_string()),
            SExpr::Atom(r#""use""#.to_string()),
            SExpr::Atom(":span".to_string()),
            SExpr::List(vec![
                SExpr::Atom("Span".to_string()),
                SExpr::Atom(":lo".to_string()),
                SExpr::Atom("0".to_string()),
                SExpr::Atom(":hi".to_string()),
                SExpr::Atom("3".to_string()),
            ]),
        ]);

        let result = formatter.format(&ident);
        // Short enough to be formatted inline
        let expected = r#"(Ident :name "use" :span (Span :lo 0 :hi 3))"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_keyword_aligned() {
        let formatter = Formatter::new(FormatConfig::default());
        let item = SExpr::List(vec![
            SExpr::Atom("Item".to_string()),
            SExpr::Atom(":id".to_string()),
            SExpr::Atom("0".to_string()),
            SExpr::Atom(":attrs".to_string()),
            SExpr::List(vec![]),
            SExpr::Atom(":vis".to_string()),
            SExpr::List(vec![SExpr::Atom("Inherited".to_string())]),
        ]);

        let result = formatter.format(&item);
        let expected = "(Item\n  :id 0\n  :attrs ()\n  :vis (Inherited))";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_multiline_general() {
        let formatter = Formatter::new(FormatConfig::default());
        let list = SExpr::List(vec![
            SExpr::Atom("list".to_string()),
            SExpr::Atom("a".to_string()),
            SExpr::Atom("b".to_string()),
            SExpr::Atom("c".to_string()),
            SExpr::Atom("d".to_string()),
            SExpr::Atom("e".to_string()),
        ]);

        let result = formatter.format(&list);
        let expected = "(list\n  a\n  b\n  c\n  d\n  e)";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_nested_lists() {
        let formatter = Formatter::new(FormatConfig::default());
        let list = SExpr::List(vec![
            SExpr::Atom("outer".to_string()),
            SExpr::List(vec![
                SExpr::Atom("inner1".to_string()),
                SExpr::Atom("a".to_string()),
            ]),
            SExpr::List(vec![
                SExpr::Atom("inner2".to_string()),
                SExpr::Atom("b".to_string()),
            ]),
        ]);

        let result = formatter.format(&list);
        let expected = "(outer\n  (inner1 a)\n  (inner2 b))";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_custom_indent() {
        let config = FormatConfig::default().with_tab_spaces(4).unwrap();
        let formatter = Formatter::new(config);

        let list = SExpr::List(vec![
            SExpr::Atom("Item".to_string()),
            SExpr::Atom(":id".to_string()),
            SExpr::Atom("0".to_string()),
        ]);

        let result = formatter.format(&list);
        // Short enough to be inline
        let expected = "(Item :id 0)";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_custom_max_width() {
        let config = FormatConfig::default().with_max_width(20).unwrap();
        let formatter = Formatter::new(config);

        let span = SExpr::List(vec![
            SExpr::Atom("VeryLongTypeName".to_string()),
            SExpr::Atom(":key".to_string()),
            SExpr::Atom("value".to_string()),
        ]);

        let result = formatter.format(&span);
        // Should be multiline because it exceeds max_width
        let expected = "(VeryLongTypeName\n  :key value)";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_format_idempotent() {
        let formatter = Formatter::new(FormatConfig::default());

        let span = SExpr::List(vec![
            SExpr::Atom("Span".to_string()),
            SExpr::Atom(":lo".to_string()),
            SExpr::Atom("0".to_string()),
            SExpr::Atom(":hi".to_string()),
            SExpr::Atom("10".to_string()),
        ]);

        let formatted1 = formatter.format(&span);

        // Parse the formatted output and format again
        let parsed = crate::parser::parse(&formatted1).unwrap();
        let formatted2 = formatter.format(&parsed);

        // Should be identical (idempotent)
        assert_eq!(formatted1, formatted2);
    }

    #[test]
    fn test_format_complex_nested() {
        let formatter = Formatter::new(FormatConfig::default());

        let crate_expr = SExpr::List(vec![
            SExpr::Atom("Crate".to_string()),
            SExpr::Atom(":attrs".to_string()),
            SExpr::List(vec![]),
            SExpr::Atom(":items".to_string()),
            SExpr::List(vec![SExpr::List(vec![
                SExpr::Atom("Item".to_string()),
                SExpr::Atom(":id".to_string()),
                SExpr::Atom("0".to_string()),
                SExpr::Atom(":span".to_string()),
                SExpr::List(vec![
                    SExpr::Atom("Span".to_string()),
                    SExpr::Atom(":lo".to_string()),
                    SExpr::Atom("0".to_string()),
                    SExpr::Atom(":hi".to_string()),
                    SExpr::Atom("0".to_string()),
                ]),
            ])]),
        ]);

        let result = formatter.format(&crate_expr);
        // Inner Item is compact enough to be inline
        let expected = r#"(Crate
  :attrs ()
  :items ((Item :id 0 :span (Span :lo 0 :hi 0))))"#;
        assert_eq!(result, expected);
    }
}
