//! Lisp Mode Evaluation
//!
//! Integrates with oxur-lang to parse and evaluate Lisp syntax.
//! Provides Tier 1 (Calculator) fast-path evaluation for simple arithmetic.
//!
//! Based on ODD-0026: Oxur REPL Evaluation Strategy

use crate::eval::{EvalError, Result};
use oxur_lang::{CoreForm, Expander, Parser};

/// Lisp mode evaluator
///
/// Integrates with oxur-lang for parsing and evaluation.
/// Provides fast-path calculator mode for simple arithmetic.
#[derive(Debug, Clone)]
pub struct LispEvaluator {}

impl LispEvaluator {
    /// Create a new Lisp evaluator
    pub fn new() -> Self {
        Self {}
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
    pub fn try_eval_calculator(&mut self, code: impl AsRef<str>) -> Option<String> {
        // Parse the code
        let expr = self.parse_simple(code.as_ref()).ok()?;

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
            return Err(EvalError::SyntaxError {
                msg: "Empty expression".to_string(),
                pos: oxur_smap::SourcePos::repl(1, 1, 1),
            });
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
            return Err(EvalError::SyntaxError {
                msg: "Mismatched parentheses".to_string(),
                pos: oxur_smap::SourcePos::repl(1, 1, 1),
            });
        }

        Ok(tokens)
    }

    /// Evaluate a calculator expression
    fn eval_calculator_expr(&self, expr: &Expr) -> Result<String> {
        match expr {
            Expr::Number(n) => Ok(n.to_string()),

            Expr::Symbol(s) => Err(EvalError::UnsupportedOperation {
                msg: format!("Variables not supported in calculator mode: {}", s),
                pos: oxur_smap::SourcePos::repl(1, 1, 1),
            }),

            Expr::List(elements) => {
                if elements.is_empty() {
                    return Err(EvalError::SyntaxError {
                        msg: "Empty list".to_string(),
                        pos: oxur_smap::SourcePos::repl(1, 1, 1),
                    });
                }

                // First element should be an operator
                let op = match &elements[0] {
                    Expr::Symbol(s) => s.as_str(),
                    _ => {
                        return Err(EvalError::SyntaxError {
                            msg: "First element must be an operator".to_string(),
                            pos: oxur_smap::SourcePos::repl(1, 1, 1),
                        })
                    }
                };

                // Evaluate arguments
                let args: Result<Vec<i64>> = elements[1..]
                    .iter()
                    .map(|e| {
                        let result = self.eval_calculator_expr(e)?;
                        result.parse::<i64>().map_err(|_| EvalError::RuntimeError {
                            msg: format!("Not a number: {}", result),
                            pos: oxur_smap::SourcePos::repl(1, 1, 1),
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
            return Err(EvalError::SyntaxError {
                msg: format!("Operator '{}' requires at least one argument", op),
                pos: oxur_smap::SourcePos::repl(1, 1, 1),
            });
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
                    return Err(EvalError::RuntimeError {
                        msg: "Division requires at least two arguments".to_string(),
                        pos: oxur_smap::SourcePos::repl(1, 1, 1),
                    });
                }

                let result = args.iter().skip(1).try_fold(args[0], |acc, &x| {
                    if x == 0 {
                        Err(EvalError::RuntimeError {
                            msg: "Division by zero".to_string(),
                            pos: oxur_smap::SourcePos::repl(1, 1, 1),
                        })
                    } else {
                        Ok(acc / x)
                    }
                })?;

                Ok(result.to_string())
            }
            _ => Err(EvalError::UnsupportedOperation {
                msg: format!("Unknown operator in calculator mode: {}", op),
                pos: oxur_smap::SourcePos::repl(1, 1, 1),
            }),
        }
    }

    /// Parse code using the full oxur-lang compilation pipeline
    ///
    /// This integrates with oxur-lang's Parser (Stage 1) and Expander (Stage 2)
    /// to convert Oxur source code into Core Forms (IR).
    pub fn parse(&mut self, code: impl AsRef<str>) -> Result<Vec<CoreForm>> {
        // Stage 1: Parse source → SurfaceForm
        let mut parser = Parser::new(code.as_ref().to_string());
        let surface_forms = parser.parse().map_err(|e| EvalError::SyntaxError {
            msg: format!("Parse error: {}", e),
            pos: oxur_smap::SourcePos::repl(1, 1, 1),
        })?;

        // Stage 2: Expand SurfaceForm → CoreForm (macro expansion, deffn, etc.)
        let mut expander = Expander::new();
        expander.expand(surface_forms).map_err(|e| EvalError::SyntaxError {
            msg: format!("Expansion error: {}", e),
            pos: oxur_smap::SourcePos::repl(1, 1, 1),
        })
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
        assert_eq!(eval.try_eval_calculator("(* 2 3 4)"), Some("24".to_string()));
    }

    #[test]
    fn test_calculator_nested_expressions() {
        let mut eval = LispEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(+ (* 2 3) 4)"), Some("10".to_string()));
        assert_eq!(eval.try_eval_calculator("(* (+ 1 2) (- 10 5))"), Some("15".to_string()));
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
            Expr::List(vec![Expr::Symbol("+".to_string()), Expr::Number(1), Expr::Number(2),])
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
                Expr::List(vec![Expr::Symbol("*".to_string()), Expr::Number(2), Expr::Number(3),]),
                Expr::Number(4),
            ])
        );
    }

    #[test]
    fn test_tokenize_simple() {
        let eval = LispEvaluator::new();
        let tokens = eval.tokenize("+ 1 2").unwrap();
        assert_eq!(tokens, vec!["+".to_string(), "1".to_string(), "2".to_string()]);
    }

    #[test]
    fn test_tokenize_nested() {
        let eval = LispEvaluator::new();
        let tokens = eval.tokenize("+ (* 2 3) 4").unwrap();
        assert_eq!(tokens, vec!["+".to_string(), "(* 2 3)".to_string(), "4".to_string()]);
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
    fn test_parse_simple_expression() {
        let mut eval = LispEvaluator::new();
        let forms = eval.parse("42").unwrap();

        assert_eq!(forms.len(), 1);
        match &forms[0] {
            CoreForm::Number { value, .. } => assert_eq!(value, &42),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_parse_symbol() {
        let mut eval = LispEvaluator::new();
        let forms = eval.parse("test-symbol").unwrap();

        assert_eq!(forms.len(), 1);
        match &forms[0] {
            CoreForm::Symbol { name, .. } => assert_eq!(name, "test-symbol"),
            _ => panic!("Expected Symbol"),
        }
    }

    #[test]
    fn test_parse_string() {
        let mut eval = LispEvaluator::new();
        let forms = eval.parse(r#""hello world""#).unwrap();

        assert_eq!(forms.len(), 1);
        match &forms[0] {
            CoreForm::String { value, .. } => assert_eq!(value, "hello world"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_parse_list() {
        let mut eval = LispEvaluator::new();
        let forms = eval.parse("(+ 1 2)").unwrap();

        assert_eq!(forms.len(), 1);
        match &forms[0] {
            CoreForm::List { elements, .. } => assert_eq!(elements.len(), 3),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_deffn_expands_to_define_func() {
        let mut eval = LispEvaluator::new();
        let forms = eval.parse(r#"(deffn main () (println! "Hello"))"#).unwrap();

        assert_eq!(forms.len(), 1);

        // deffn should expand to DefineFunc, not remain as List
        match &forms[0] {
            CoreForm::DefineFunc { name, params, body, .. } => {
                assert_eq!(name, "main");
                assert!(params.is_empty());

                // Body should be the (println! "Hello") list
                match &**body {
                    CoreForm::List { elements, .. } => {
                        assert_eq!(elements.len(), 2);
                        match &elements[0] {
                            CoreForm::Symbol { name, .. } => assert_eq!(name, "println!"),
                            _ => panic!("Expected println! symbol"),
                        }
                    }
                    _ => panic!("Expected List as body"),
                }
            }
            other => panic!("Expected DefineFunc, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_deffn_with_params() {
        let mut eval = LispEvaluator::new();
        let forms = eval.parse("(deffn add (a b) (+ a b))").unwrap();

        assert_eq!(forms.len(), 1);

        match &forms[0] {
            CoreForm::DefineFunc { name, params, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], "a");
                assert_eq!(params[1], "b");
            }
            other => panic!("Expected DefineFunc, got {:?}", other),
        }
    }
}
