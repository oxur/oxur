use crate::sexp::types::*;

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
