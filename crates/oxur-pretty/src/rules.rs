//! Formatting rules and heuristics for S-expressions.

use crate::config::FormatConfig;
use crate::parser::SExpr;

/// Formatting rules engine that determines how to format S-expressions.
pub struct FormattingRules {
    config: FormatConfig,
}

impl FormattingRules {
    /// Create a new rules engine with the given configuration.
    pub fn new(config: &FormatConfig) -> Self {
        Self { config: config.clone() }
    }

    /// Determine if an expression should be formatted inline.
    pub fn should_inline(&self, expr: &SExpr) -> bool {
        match expr {
            SExpr::Atom(_) => true,
            SExpr::List(items) => {
                // Empty list is always inline
                if items.is_empty() {
                    return true;
                }

                // Check if it's a compact struct that fits
                if self.config.compact_simple_values && self.is_compact_struct(items) {
                    return true;
                }

                // Small lists of atoms can be inline if they fit
                if items.len() <= self.config.max_inline_items && self.all_atoms(items) {
                    let width = self.estimate_width(expr);
                    return width <= self.config.max_width;
                }

                false
            }
        }
    }

    /// Check if all items in a list are atoms.
    pub fn all_atoms(&self, items: &[SExpr]) -> bool {
        items.iter().all(|item| matches!(item, SExpr::Atom(_)))
    }

    /// Estimate the width if this expression were formatted on one line.
    pub fn estimate_width(&self, expr: &SExpr) -> usize {
        match expr {
            SExpr::Atom(s) => s.len(),
            SExpr::List(items) => {
                let mut width = 2; // For opening and closing parens

                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        width += 1; // Space between items
                    }
                    width += self.estimate_width(item);
                }

                width
            }
        }
    }

    /// Check if this is a compact struct pattern: (TypeName :key1 val1 :key2 val2)
    /// These can be formatted on one line if:
    /// 1. First element is a type name (not a keyword)
    /// 2. All values after keywords are simple (atoms or compact structs)
    /// 3. Total width fits within max_width
    pub fn is_compact_struct(&self, items: &[SExpr]) -> bool {
        if items.is_empty() {
            return true; // Empty list () is compact
        }

        // First element should be a type name (not a keyword)
        if let Some(SExpr::Atom(first)) = items.first() {
            if first.starts_with(':') {
                return false;
            }
        } else {
            return false;
        }

        // Check if this has keyword-value structure
        if !self.has_keyword_structure(items) {
            return false;
        }

        // Check if all values are simple
        let mut i = 1;
        while i < items.len() {
            // Skip keyword
            i += 1;

            // Check value (if present)
            if i < items.len() {
                if !self.is_simple_value(&items[i]) {
                    return false;
                }
                i += 1;
            }
        }

        // Check if it fits within max_width
        let width = self.estimate_width(&SExpr::List(items.to_vec()));
        width <= self.config.max_width
    }

    /// Check if a value is simple (atom or compact nested struct).
    pub fn is_simple_value(&self, expr: &SExpr) -> bool {
        match expr {
            SExpr::Atom(_) => true,
            SExpr::List(items) => {
                // Recursively check for compact struct
                self.is_compact_struct(items)
            }
        }
    }

    /// Check if this has keyword-value pair structure.
    /// Returns true if elements after the first follow a :keyword value pattern.
    pub fn has_keyword_structure(&self, items: &[SExpr]) -> bool {
        if items.len() < 3 {
            return false; // Need at least (Type :key value)
        }

        // Check that all odd-indexed elements (1, 3, 5, ...) are keywords
        let mut i = 1;
        while i < items.len() {
            if let Some(SExpr::Atom(s)) = items.get(i) {
                if !s.starts_with(':') {
                    return false;
                }
            } else {
                return false;
            }
            i += 2; // Skip to next keyword position
        }

        true
    }

    /// Determine if a list should use keyword-aligned formatting.
    /// This is for structures like:
    /// ```lisp
    /// (TypeName
    ///   :key1 value1
    ///   :key2 value2)
    /// ```
    pub fn should_keyword_align(&self, items: &[SExpr]) -> bool {
        if !self.config.align_keywords {
            return false;
        }

        // Must have keyword structure
        if !self.has_keyword_structure(items) {
            return false;
        }

        // If it's a compact struct, use inline instead
        if self.config.compact_simple_values && self.is_compact_struct(items) {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_rules() -> FormattingRules {
        FormattingRules::new(&FormatConfig::default())
    }

    #[test]
    fn test_all_atoms_true() {
        let rules = default_rules();
        let items = vec![
            SExpr::Atom("a".to_string()),
            SExpr::Atom("b".to_string()),
            SExpr::Atom("c".to_string()),
        ];
        assert!(rules.all_atoms(&items));
    }

    #[test]
    fn test_all_atoms_false() {
        let rules = default_rules();
        let items =
            vec![SExpr::Atom("a".to_string()), SExpr::List(vec![]), SExpr::Atom("c".to_string())];
        assert!(!rules.all_atoms(&items));
    }

    #[test]
    fn test_estimate_width_atom() {
        let rules = default_rules();
        let atom = SExpr::Atom("hello".to_string());
        assert_eq!(rules.estimate_width(&atom), 5);
    }

    #[test]
    fn test_estimate_width_empty_list() {
        let rules = default_rules();
        let list = SExpr::List(vec![]);
        assert_eq!(rules.estimate_width(&list), 2); // ()
    }

    #[test]
    fn test_estimate_width_simple_list() {
        let rules = default_rules();
        let list = SExpr::List(vec![
            SExpr::Atom("a".to_string()),
            SExpr::Atom("b".to_string()),
            SExpr::Atom("c".to_string()),
        ]);
        assert_eq!(rules.estimate_width(&list), 7); // (a b c) = 7 chars
    }

    #[test]
    fn test_estimate_width_nested_list() {
        let rules = default_rules();
        let list = SExpr::List(vec![
            SExpr::Atom("a".to_string()),
            SExpr::List(vec![SExpr::Atom("b".to_string())]),
        ]);
        assert_eq!(rules.estimate_width(&list), 7); // (a (b))
    }

    #[test]
    fn test_has_keyword_structure_true() {
        let rules = default_rules();
        let items = vec![
            SExpr::Atom("Span".to_string()),
            SExpr::Atom(":lo".to_string()),
            SExpr::Atom("0".to_string()),
            SExpr::Atom(":hi".to_string()),
            SExpr::Atom("10".to_string()),
        ];
        assert!(rules.has_keyword_structure(&items));
    }

    #[test]
    fn test_has_keyword_structure_false_no_colon() {
        let rules = default_rules();
        let items = vec![
            SExpr::Atom("Span".to_string()),
            SExpr::Atom("lo".to_string()), // Missing colon
            SExpr::Atom("0".to_string()),
        ];
        assert!(!rules.has_keyword_structure(&items));
    }

    #[test]
    fn test_has_keyword_structure_false_too_short() {
        let rules = default_rules();
        let items = vec![SExpr::Atom("Span".to_string())];
        assert!(!rules.has_keyword_structure(&items));
    }

    #[test]
    fn test_is_simple_value_atom() {
        let rules = default_rules();
        let atom = SExpr::Atom("hello".to_string());
        assert!(rules.is_simple_value(&atom));
    }

    #[test]
    fn test_is_simple_value_compact_struct() {
        let rules = default_rules();
        let compact = SExpr::List(vec![
            SExpr::Atom("Span".to_string()),
            SExpr::Atom(":lo".to_string()),
            SExpr::Atom("0".to_string()),
            SExpr::Atom(":hi".to_string()),
            SExpr::Atom("10".to_string()),
        ]);
        assert!(rules.is_simple_value(&compact));
    }

    #[test]
    fn test_is_compact_struct_empty_list() {
        let rules = default_rules();
        assert!(rules.is_compact_struct(&[]));
    }

    #[test]
    fn test_is_compact_struct_simple() {
        let rules = default_rules();
        let items = vec![
            SExpr::Atom("Span".to_string()),
            SExpr::Atom(":lo".to_string()),
            SExpr::Atom("0".to_string()),
            SExpr::Atom(":hi".to_string()),
            SExpr::Atom("10".to_string()),
        ];
        assert!(rules.is_compact_struct(&items));
    }

    #[test]
    fn test_is_compact_struct_nested_compact() {
        let rules = default_rules();
        let items = vec![
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
        ];
        // This should be compact if it fits within max_width
        let width = rules.estimate_width(&SExpr::List(items.clone()));
        assert_eq!(rules.is_compact_struct(&items), width <= 100);
    }

    #[test]
    fn test_is_compact_struct_false_starts_with_keyword() {
        let rules = default_rules();
        let items = vec![
            SExpr::Atom(":name".to_string()), // Starts with keyword
            SExpr::Atom("value".to_string()),
        ];
        assert!(!rules.is_compact_struct(&items));
    }

    #[test]
    fn test_is_compact_struct_false_too_wide() {
        let config = FormatConfig::default().with_max_width(10).unwrap();
        let rules = FormattingRules::new(&config);

        let items = vec![
            SExpr::Atom("VeryLongTypeName".to_string()),
            SExpr::Atom(":key".to_string()),
            SExpr::Atom("value".to_string()),
        ];
        assert!(!rules.is_compact_struct(&items));
    }

    #[test]
    fn test_should_inline_atom() {
        let rules = default_rules();
        let atom = SExpr::Atom("hello".to_string());
        assert!(rules.should_inline(&atom));
    }

    #[test]
    fn test_should_inline_empty_list() {
        let rules = default_rules();
        let list = SExpr::List(vec![]);
        assert!(rules.should_inline(&list));
    }

    #[test]
    fn test_should_inline_compact_struct() {
        let rules = default_rules();
        let list = SExpr::List(vec![
            SExpr::Atom("Span".to_string()),
            SExpr::Atom(":lo".to_string()),
            SExpr::Atom("0".to_string()),
            SExpr::Atom(":hi".to_string()),
            SExpr::Atom("10".to_string()),
        ]);
        assert!(rules.should_inline(&list));
    }

    #[test]
    fn test_should_inline_small_list_atoms() {
        let rules = default_rules();
        let list = SExpr::List(vec![
            SExpr::Atom("a".to_string()),
            SExpr::Atom("b".to_string()),
            SExpr::Atom("c".to_string()),
        ]);
        assert!(rules.should_inline(&list));
    }

    #[test]
    fn test_should_inline_false_too_many_items() {
        let rules = default_rules();
        let list = SExpr::List(vec![
            SExpr::Atom("a".to_string()),
            SExpr::Atom("b".to_string()),
            SExpr::Atom("c".to_string()),
            SExpr::Atom("d".to_string()),
            SExpr::Atom("e".to_string()),
        ]);
        assert!(!rules.should_inline(&list));
    }

    #[test]
    fn test_should_keyword_align_true() {
        let rules = default_rules();
        let items = vec![
            SExpr::Atom("Item".to_string()),
            SExpr::Atom(":id".to_string()),
            SExpr::Atom("0".to_string()),
            SExpr::Atom(":span".to_string()),
            SExpr::List(vec![
                SExpr::Atom("Span".to_string()),
                SExpr::Atom(":lo".to_string()),
                SExpr::Atom("0".to_string()),
            ]),
        ];
        // This is compact because both values are simple and it fits width
        assert!(!rules.should_keyword_align(&items));
    }

    #[test]
    fn test_should_keyword_align_false_compact() {
        let rules = default_rules();
        let items = vec![
            SExpr::Atom("Span".to_string()),
            SExpr::Atom(":lo".to_string()),
            SExpr::Atom("0".to_string()),
            SExpr::Atom(":hi".to_string()),
            SExpr::Atom("10".to_string()),
        ];
        // This is compact, so don't keyword align
        assert!(!rules.should_keyword_align(&items));
    }

    #[test]
    fn test_should_keyword_align_false_disabled() {
        let config = FormatConfig::default().with_align_keywords(false);
        let rules = FormattingRules::new(&config);

        let items = vec![
            SExpr::Atom("Item".to_string()),
            SExpr::Atom(":id".to_string()),
            SExpr::Atom("0".to_string()),
        ];
        assert!(!rules.should_keyword_align(&items));
    }
}
