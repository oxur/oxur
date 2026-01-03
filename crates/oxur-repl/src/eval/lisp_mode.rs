//! Lisp Mode Evaluation
//!
//! Integrates with oxur-lang to parse and evaluate Lisp syntax.
//! Provides Tier 1 (Calculator) fast-path evaluation for simple arithmetic.
//!
//! Based on ODD-0026: Oxur REPL Evaluation Strategy

use crate::eval::{EvalError, Result};
use oxur_lang::{CoreForm, NodeId, Parser};

/// Lisp mode evaluator
///
/// Integrates with oxur-lang for parsing and evaluation.
/// Provides fast-path calculator mode for simple arithmetic.
#[derive(Clone)]
pub struct LispEvaluator {
    /// Next node ID for AST construction
    next_id: u64,
}

impl LispEvaluator {
    /// Create a new Lisp evaluator
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    /// Try to evaluate code in calculator mode (Tier 1)
    ///
    /// Only handles simple arithmetic expressions:
    /// - `(+ 1 2)` → `"3"`
    /// - `(- 10 5)` → `"5"`
    /// - `(* 3 4)` → `"12"`
    /// - `(/ 10 2)` → `"5"`
    /// - Nested: `(+ (* 2 3) 4)` → `"10"`
    ///
    /// Returns `Some(result)` if successful, `None` if not calculator-eligible.
    pub fn try_eval_calculator(&mut self, code: &str) -> Option<String> {
        // Parse the code
        let expr = self.parse_simple(code).ok()?;

        // Evaluate as calculator expression
        self.eval_calculator_expr(&expr).ok()
    }

    /// Parse simple Lisp expression
    ///
    /// Handles basic s-expression syntax:
    /// - Numbers: `42`, `-5`, `123`
    /// - Symbols: `+`, `-`, `*`, `/`, `foo`, `bar`
    /// - Lists: `(+ 1 2)`, `(foo bar baz)`
    fn parse_simple(&mut self, code: &str) -> Result<Expr> {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Err(EvalError::SyntaxError("Empty expression".to_string()));
        }

        // Try to parse as number first
        if let Ok(num) = trimmed.parse::<i64>() {
            return Ok(Expr::Number(num));
        }

        // Check if it's a list
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            return self.parse_list(trimmed);
        }

        // Otherwise, it's a symbol
        Ok(Expr::Symbol(trimmed.to_string()))
    }

    /// Parse a list expression: `(op arg1 arg2 ...)`
    fn parse_list(&mut self, code: &str) -> Result<Expr> {
        // Remove outer parentheses
        let inner = &code[1..code.len() - 1].trim();

        if inner.is_empty() {
            return Ok(Expr::List(vec![]));
        }

        // Tokenize the inner content (returns owned strings)
        let tokens = self.tokenize(inner)?;

        // Parse each token
        let mut elements = Vec::new();
        for token in tokens {
            elements.push(self.parse_simple(&token)?);
        }

        Ok(Expr::List(elements))
    }

    /// Tokenize a string into s-expression tokens
    ///
    /// Handles nested parentheses correctly:
    /// `(+ (* 2 3) 4)` → `["+", "(* 2 3)", "4"]`
    fn tokenize(&self, input: &str) -> Result<Vec<String>> {
        let mut tokens = Vec::new();
        let mut current_start = 0;
        let mut depth = 0;
        let bytes = input.as_bytes();

        for (i, &byte) in bytes.iter().enumerate() {
            match byte {
                b'(' => {
                    if depth == 0 && i > current_start {
                        let token = input[current_start..i].trim();
                        if !token.is_empty() {
                            tokens.push(token.to_string());
                        }
                        current_start = i;
                    }
                    depth += 1;
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        let token = input[current_start..=i].trim();
                        if !token.is_empty() {
                            tokens.push(token.to_string());
                        }
                        current_start = i + 1;
                    }
                }
                b' ' | b'\t' | b'\n' => {
                    if depth == 0 && i > current_start {
                        let token = input[current_start..i].trim();
                        if !token.is_empty() {
                            tokens.push(token.to_string());
                        }
                        current_start = i + 1;
                    }
                }
                _ => {}
            }
        }

        // Handle final token
        if current_start < input.len() {
            let token = input[current_start..].trim();
            if !token.is_empty() {
                tokens.push(token.to_string());
            }
        }

        if depth != 0 {
            return Err(EvalError::SyntaxError(
                "Mismatched parentheses".to_string(),
            ));
        }

        Ok(tokens)
    }

    /// Evaluate a calculator expression
    fn eval_calculator_expr(&self, expr: &Expr) -> Result<String> {
        match expr {
            Expr::Number(n) => Ok(n.to_string()),

            Expr::Symbol(s) => {
                Err(EvalError::UnsupportedOperation(format!(
                    "Variables not supported in calculator mode: {}",
                    s
                )))
            }

            Expr::List(elements) => {
                if elements.is_empty() {
                    return Err(EvalError::SyntaxError("Empty list".to_string()));
                }

                // First element should be an operator
                let op = match &elements[0] {
                    Expr::Symbol(s) => s.as_str(),
                    _ => {
                        return Err(EvalError::SyntaxError(
                            "First element must be an operator".to_string(),
                        ))
                    }
                };

                // Evaluate arguments
                let args: Result<Vec<i64>> = elements[1..]
                    .iter()
                    .map(|e| {
                        let result = self.eval_calculator_expr(e)?;
                        result.parse::<i64>().map_err(|_| {
                            EvalError::RuntimeError(format!("Not a number: {}", result))
                        })
                    })
                    .collect();

                let args = args?;

                // Apply operator
                self.apply_operator(op, &args)
            }
        }
    }

    /// Apply an arithmetic operator to arguments
    fn apply_operator(&self, op: &str, args: &[i64]) -> Result<String> {
        if args.is_empty() {
            return Err(EvalError::SyntaxError(format!(
                "Operator '{}' requires at least one argument",
                op
            )));
        }

        match op {
            "+" => {
                let sum: i64 = args.iter().sum();
                Ok(sum.to_string())
            }
            "-" => {
                if args.len() == 1 {
                    Ok((-args[0]).to_string())
                } else {
                    let result = args.iter().skip(1).fold(args[0], |acc, &x| acc - x);
                    Ok(result.to_string())
                }
            }
            "*" => {
                let product: i64 = args.iter().product();
                Ok(product.to_string())
            }
            "/" => {
                if args.len() == 1 {
                    return Err(EvalError::RuntimeError(
                        "Division requires at least two arguments".to_string(),
                    ));
                }

                let result = args.iter().skip(1).try_fold(args[0], |acc, &x| {
                    if x == 0 {
                        Err(EvalError::RuntimeError("Division by zero".to_string()))
                    } else {
                        Ok(acc / x)
                    }
                })?;

                Ok(result.to_string())
            }
            _ => Err(EvalError::UnsupportedOperation(format!(
                "Unknown operator in calculator mode: {}",
                op
            ))),
        }
    }

    /// Parse code using oxur-lang Parser
    ///
    /// This integrates with the full oxur-lang compilation pipeline.
    /// Currently a placeholder until Parser is fully implemented.
    pub fn parse(&mut self, code: &str) -> Result<Vec<CoreForm>> {
        // Use oxur-lang Parser
        let mut parser = Parser::new(code.to_string());
        let surface_forms = parser
            .parse()
            .map_err(|e| EvalError::SyntaxError(format!("Parse error: {}", e)))?;

        // For now, convert to CoreForm manually
        // When Expander is ready, we'll use: expander.expand(surface_forms)
        self.surface_to_core(surface_forms)
    }

    /// Convert surface forms to core forms
    ///
    /// Temporary implementation until Expander is ready.
    fn surface_to_core(
        &mut self,
        forms: Vec<oxur_lang::parser::SurfaceForm>,
    ) -> Result<Vec<CoreForm>> {
        forms.into_iter().map(|f| self.convert_form(f)).collect()
    }

    /// Convert a single surface form to core form
    fn convert_form(&mut self, form: oxur_lang::parser::SurfaceForm) -> Result<CoreForm> {
        use oxur_lang::parser::SurfaceForm;

        let id = self.next_node_id();

        match form {
            SurfaceForm::Symbol(name) => Ok(CoreForm::Symbol { id, name }),
            SurfaceForm::Number(value) => Ok(CoreForm::Number { id, value }),
            SurfaceForm::String(value) => Ok(CoreForm::String { id, value }),
            SurfaceForm::List(elements) => {
                let elements: Result<Vec<_>> = elements
                    .into_iter()
                    .map(|e| self.convert_form(e))
                    .collect();
                Ok(CoreForm::List {
                    id,
                    elements: elements?,
                })
            }
        }
    }

    /// Generate next node ID
    fn next_node_id(&mut self) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id += 1;
        id
    }
}

impl Default for LispEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple expression type for calculator mode
#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Number(i64),
    Symbol(String),
    List(Vec<Expr>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_simple_addition() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(+ 1 2)"), Some("3".to_string()));
    }

    #[test]
    fn test_calculator_simple_subtraction() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(- 10 5)"), Some("5".to_string()));
    }

    #[test]
    fn test_calculator_simple_multiplication() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(* 3 4)"), Some("12".to_string()));
    }

    #[test]
    fn test_calculator_simple_division() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(/ 10 2)"), Some("5".to_string()));
    }

    #[test]
    fn test_calculator_multiple_args() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(+ 1 2 3 4)"), Some("10".to_string()));
        assert_eq!(
            eval.try_eval_calculator("(* 2 3 4)"),
            Some("24".to_string())
        );
    }

    #[test]
    fn test_calculator_nested_expressions() {
        let mut eval = LispEvaluator::new();
        assert_eq!(
            eval.try_eval_calculator("(+ (* 2 3) 4)"),
            Some("10".to_string())
        );
        assert_eq!(
            eval.try_eval_calculator("(* (+ 1 2) (- 10 5))"),
            Some("15".to_string())
        );
    }

    #[test]
    fn test_calculator_unary_minus() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(- 5)"), Some("-5".to_string()));
    }

    #[test]
    fn test_calculator_division_by_zero() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(/ 10 0)"), None);
    }

    #[test]
    fn test_calculator_invalid_operator() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(foo 1 2)"), None);
    }

    #[test]
    fn test_calculator_variables_not_supported() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(+ x 2)"), None);
    }

    #[test]
    fn test_calculator_empty_list() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("()"), None);
    }

    #[test]
    fn test_parse_simple_number() {
        let mut eval = LispEvaluator::new();
        let expr = eval.parse_simple("42").unwrap();
        assert_eq!(expr, Expr::Number(42));
    }

    #[test]
    fn test_parse_simple_symbol() {
        let mut eval = LispEvaluator::new();
        let expr = eval.parse_simple("+").unwrap();
        assert_eq!(expr, Expr::Symbol("+".to_string()));
    }

    #[test]
    fn test_parse_simple_list() {
        let mut eval = LispEvaluator::new();
        let expr = eval.parse_simple("(+ 1 2)").unwrap();
        assert_eq!(
            expr,
            Expr::List(vec![
                Expr::Symbol("+".to_string()),
                Expr::Number(1),
                Expr::Number(2),
            ])
        );
    }

    #[test]
    fn test_parse_nested_list() {
        let mut eval = LispEvaluator::new();
        let expr = eval.parse_simple("(+ (* 2 3) 4)").unwrap();
        assert_eq!(
            expr,
            Expr::List(vec![
                Expr::Symbol("+".to_string()),
                Expr::List(vec![
                    Expr::Symbol("*".to_string()),
                    Expr::Number(2),
                    Expr::Number(3),
                ]),
                Expr::Number(4),
            ])
        );
    }

    #[test]
    fn test_tokenize_simple() {
        let eval = LispEvaluator::new();
        let tokens = eval.tokenize("+ 1 2").unwrap();
        assert_eq!(
            tokens,
            vec!["+".to_string(), "1".to_string(), "2".to_string()]
        );
    }

    #[test]
    fn test_tokenize_nested() {
        let eval = LispEvaluator::new();
        let tokens = eval.tokenize("+ (* 2 3) 4").unwrap();
        assert_eq!(
            tokens,
            vec!["+".to_string(), "(* 2 3)".to_string(), "4".to_string()]
        );
    }

    #[test]
    fn test_tokenize_mismatched_parens() {
        let eval = LispEvaluator::new();
        let result = eval.tokenize("(+ 1 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_operator_addition() {
        let eval = LispEvaluator::new();
        assert_eq!(eval.apply_operator("+", &[1, 2, 3]).unwrap(), "6");
    }

    #[test]
    fn test_apply_operator_subtraction() {
        let eval = LispEvaluator::new();
        assert_eq!(eval.apply_operator("-", &[10, 3, 2]).unwrap(), "5");
    }

    #[test]
    fn test_apply_operator_multiplication() {
        let eval = LispEvaluator::new();
        assert_eq!(eval.apply_operator("*", &[2, 3, 4]).unwrap(), "24");
    }

    #[test]
    fn test_apply_operator_division() {
        let eval = LispEvaluator::new();
        assert_eq!(eval.apply_operator("/", &[20, 2, 2]).unwrap(), "5");
    }

    #[test]
    fn test_apply_operator_unknown() {
        let eval = LispEvaluator::new();
        assert!(eval.apply_operator("foo", &[1, 2]).is_err());
    }

    #[test]
    fn test_surface_to_core_conversion() {
        use oxur_lang::parser::SurfaceForm;

        let mut eval = LispEvaluator::new();
        let surface = vec![SurfaceForm::Number(42)];
        let core = eval.surface_to_core(surface).unwrap();

        assert_eq!(core.len(), 1);
        match &core[0] {
            CoreForm::Number { value, .. } => assert_eq!(value, &42),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_convert_form_symbol() {
        use oxur_lang::parser::SurfaceForm;

        let mut eval = LispEvaluator::new();
        let form = SurfaceForm::Symbol("test".to_string());
        let core = eval.convert_form(form).unwrap();

        match core {
            CoreForm::Symbol { name, .. } => assert_eq!(name, "test"),
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_convert_form_list() {
        use oxur_lang::parser::SurfaceForm;

        let mut eval = LispEvaluator::new();
        let form = SurfaceForm::List(vec![
            SurfaceForm::Symbol("+".to_string()),
            SurfaceForm::Number(1),
            SurfaceForm::Number(2),
        ]);
        let core = eval.convert_form(form).unwrap();

        match core {
            CoreForm::List { elements, .. } => assert_eq!(elements.len(), 3),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_node_id_generation() {
        let mut eval = LispEvaluator::new();
        let id1 = eval.next_node_id();
        let id2 = eval.next_node_id();
        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
    }
}
