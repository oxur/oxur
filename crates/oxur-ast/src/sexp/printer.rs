use crate::sexp::types::*;
use std::path::Path;

pub struct Printer {
    indent: usize,
}

impl Printer {
    pub fn new() -> Self {
        Self { indent: 2 }
    }

    pub fn with_indent(indent: usize) -> Self {
        Self { indent }
    }

    pub fn print(&self, sexp: &SExp) -> String {
        self.print_sexp(sexp, 0)
    }

    /// Write an S-expression to a file
    pub fn write_file<P: AsRef<Path>>(&self, sexp: &SExp, path: P) -> std::io::Result<()> {
        let content = self.print(sexp);
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    fn print_sexp(&self, sexp: &SExp, depth: usize) -> String {
        match sexp {
            SExp::Symbol(s) => s.value.clone(),
            SExp::Keyword(k) => format!(":{}", k.name),
            SExp::String(s) => format!("\"{}\"", escape_string(&s.value)),
            SExp::Number(n) => n.value.clone(),
            SExp::Nil(_) => "nil".to_string(),
            SExp::List(l) => self.print_list(l, depth),
        }
    }

    fn print_list(&self, list: &List, depth: usize) -> String {
        if list.elements.is_empty() {
            return "()".to_string();
        }

        // For simple lists (short, no nested lists), print on one line
        if self.is_simple_list(list) {
            let elements: Vec<String> =
                list.elements.iter().map(|e| self.print_sexp(e, depth + 1)).collect();
            return format!("({})", elements.join(" "));
        }

        // For complex lists, use indentation
        let mut result = String::from("(");

        for (i, element) in list.elements.iter().enumerate() {
            if i == 0 {
                result.push_str(&self.print_sexp(element, depth + 1));
            } else {
                result.push('\n');
                result.push_str(&self.current_indent(depth + 1));
                result.push_str(&self.print_sexp(element, depth + 1));
            }
        }

        result.push(')');
        result
    }

    fn is_simple_list(&self, list: &List) -> bool {
        // A list is simple if:
        // 1. It has fewer than 4 elements
        // 2. None of its elements are lists
        // 3. The total length is reasonable

        if list.elements.len() > 3 {
            return false;
        }

        for element in &list.elements {
            if matches!(element, SExp::List(_)) {
                return false;
            }
        }

        true
    }

    fn current_indent(&self, depth: usize) -> String {
        " ".repeat(depth * self.indent)
    }
}

impl Default for Printer {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape special characters in a string
fn escape_string(s: &str) -> String {
    s.chars()
        .flat_map(|ch| match ch {
            '\n' => vec!['\\', 'n'],
            '\t' => vec!['\\', 't'],
            '\r' => vec!['\\', 'r'],
            '\\' => vec!['\\', '\\'],
            '"' => vec!['\\', '"'],
            c => vec![c],
        })
        .collect()
}

/// Convenience function for printing S-expressions
pub fn print_sexp(sexp: &SExp) -> String {
    Printer::new().print(sexp)
}

/// Convenience function for writing S-expressions to a file
pub fn write_sexp_file<P: AsRef<Path>>(sexp: &SExp, path: P) -> std::io::Result<()> {
    Printer::new().write_file(sexp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Position;
    use crate::sexp::parser::Parser;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_file_simple() {
        let sexp = SExp::Symbol(Symbol::new("foo".to_string(), Position::new(0, 1, 1)));
        let temp_file = NamedTempFile::new().unwrap();

        let result = Printer::new().write_file(&sexp, temp_file.path());
        assert!(result.is_ok());

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(content, "foo");
    }

    #[test]
    fn test_write_file_list() {
        let elements = vec![
            SExp::Symbol(Symbol::new("foo".to_string(), Position::new(0, 1, 1))),
            SExp::Symbol(Symbol::new("bar".to_string(), Position::new(0, 1, 5))),
        ];
        let sexp = SExp::List(List::new(elements, Position::new(0, 1, 1)));
        let temp_file = NamedTempFile::new().unwrap();

        let result = Printer::new().write_file(&sexp, temp_file.path());
        assert!(result.is_ok());

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(content, "(foo bar)");
    }

    #[test]
    fn test_write_sexp_file_convenience() {
        let sexp = SExp::Number(Number::new("42".to_string(), Position::new(0, 1, 1)));
        let temp_file = NamedTempFile::new().unwrap();

        let result = write_sexp_file(&sexp, temp_file.path());
        assert!(result.is_ok());

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(content, "42");
    }

    #[test]
    fn test_round_trip_file_io() {
        // Create S-expression
        let elements = vec![
            SExp::Symbol(Symbol::new("Crate".to_string(), Position::new(0, 1, 1))),
            SExp::Keyword(Keyword::new("items".to_string(), Position::new(0, 1, 7))),
            SExp::List(List::new(vec![], Position::new(0, 1, 14))),
        ];
        let original = SExp::List(List::new(elements, Position::new(0, 1, 1)));

        // Write to file
        let temp_file = NamedTempFile::new().unwrap();
        write_sexp_file(&original, temp_file.path()).unwrap();

        // Read back
        let parsed = Parser::parse_file(temp_file.path()).unwrap();

        // Verify structure matches (positions will differ)
        match (&original, &parsed) {
            (SExp::List(orig_list), SExp::List(parsed_list)) => {
                assert_eq!(orig_list.elements.len(), parsed_list.elements.len());
            }
            _ => panic!("Expected lists"),
        }
    }
}
