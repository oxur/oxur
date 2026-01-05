//! S-expression Mode Evaluation
//!
//! Evaluates canonical s-expressions as defined in ODD-0003.
//! This format is used for Rust AST representation with keyword-based fields.
//!
//! Based on ODD-0026: Oxur REPL Evaluation Strategy

use crate::eval::{EvalError, Result};
use oxur_lang::{CoreForm, NodeId};
use std::collections::HashMap;

/// S-expression mode evaluator
///
/// Handles canonical s-expression format with keyword fields.
/// Format: `(NodeType :field1 value1 :field2 value2)`
#[derive(Clone)]
pub struct SexprEvaluator {
    /// Symbol table for variable bindings
    symbols: HashMap<String, SexprValue>,
}

impl SexprEvaluator {
    /// Create a new s-expression evaluator
    pub fn new() -> Self {
        Self { symbols: HashMap::new() }
    }

    /// Try to evaluate code in calculator mode (Tier 1)
    ///
    /// For s-expression mode, calculator tier handles:
    /// - Simple numeric expressions
    /// - Keyword-value pairs
    /// - List construction
    ///
    /// Returns `Some(result)` if successful, `None` if not calculator-eligible.
    pub fn try_eval_calculator(&mut self, code: &str) -> Option<String> {
        // Parse and evaluate
        match self.eval(code) {
            Ok(value) => Some(self.value_to_string(&value)),
            Err(_) => None,
        }
    }

    /// Evaluate s-expression code (internal method)
    fn eval(&mut self, code: &str) -> Result<SexprValue> {
        let expr = self.parse(code)?;
        self.eval_expr(&expr)
    }

    /// Parse s-expression from string
    fn parse(&self, code: &str) -> Result<SexprExpr> {
        let trimmed = code.trim();

        if trimmed.is_empty() {
            return Err(EvalError::SyntaxError("Empty expression".to_string()));
        }

        // Try to parse as number
        if let Ok(num) = trimmed.parse::<i64>() {
            return Ok(SexprExpr::Number(num));
        }

        // Try to parse as string literal
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            let content = &trimmed[1..trimmed.len() - 1];
            return Ok(SexprExpr::String(content.to_string()));
        }

        // Check for keyword
        if let Some(stripped) = trimmed.strip_prefix(':') {
            return Ok(SexprExpr::Keyword(stripped.to_string()));
        }

        // Check for list
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            return self.parse_list(trimmed);
        }

        // Otherwise, it's a symbol
        Ok(SexprExpr::Symbol(trimmed.to_string()))
    }

    /// Parse a list expression
    fn parse_list(&self, code: &str) -> Result<SexprExpr> {
        let inner = &code[1..code.len() - 1].trim();

        if inner.is_empty() {
            return Ok(SexprExpr::List(vec![]));
        }

        // Tokenize
        let tokens = self.tokenize(inner)?;

        // Parse each token
        let mut elements = Vec::new();
        for token in tokens {
            elements.push(self.parse(&token)?);
        }

        Ok(SexprExpr::List(elements))
    }

    /// Tokenize s-expression content
    fn tokenize(&self, input: &str) -> Result<Vec<String>> {
        let mut tokens = Vec::new();
        let mut current_start = 0;
        let mut depth = 0;
        let mut in_string = false;
        let bytes = input.as_bytes();

        for (i, &byte) in bytes.iter().enumerate() {
            match byte {
                b'"' => {
                    in_string = !in_string;
                }
                b'(' if !in_string => {
                    if depth == 0 && i > current_start {
                        let token = input[current_start..i].trim();
                        if !token.is_empty() {
                            tokens.push(token.to_string());
                        }
                        current_start = i;
                    }
                    depth += 1;
                }
                b')' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        let token = input[current_start..=i].trim();
                        if !token.is_empty() {
                            tokens.push(token.to_string());
                        }
                        current_start = i + 1;
                    }
                }
                b' ' | b'\t' | b'\n' if !in_string && depth == 0 => {
                    if i > current_start {
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
            return Err(EvalError::SyntaxError("Mismatched parentheses".to_string()));
        }

        if in_string {
            return Err(EvalError::SyntaxError("Unterminated string".to_string()));
        }

        Ok(tokens)
    }

    /// Evaluate an s-expression
    fn eval_expr(&mut self, expr: &SexprExpr) -> Result<SexprValue> {
        match expr {
            SexprExpr::Number(n) => Ok(SexprValue::Number(*n)),
            SexprExpr::String(s) => Ok(SexprValue::String(s.clone())),
            SexprExpr::Keyword(k) => Ok(SexprValue::Keyword(k.clone())),
            SexprExpr::Symbol(s) => {
                // Look up symbol in symbol table
                self.symbols
                    .get(s)
                    .cloned()
                    .ok_or_else(|| EvalError::RuntimeError(format!("Undefined symbol: {}", s)))
            }
            SexprExpr::List(elements) => {
                if elements.is_empty() {
                    return Ok(SexprValue::List(vec![]));
                }

                // Check if first element is a special form or function
                match &elements[0] {
                    SexprExpr::Symbol(name) => self.eval_special_form(name, &elements[1..]),
                    _ => {
                        // Regular list - evaluate all elements
                        let values: Result<Vec<_>> =
                            elements.iter().map(|e| self.eval_expr(e)).collect();
                        Ok(SexprValue::List(values?))
                    }
                }
            }
        }
    }

    /// Evaluate special forms
    fn eval_special_form(&mut self, name: &str, args: &[SexprExpr]) -> Result<SexprValue> {
        match name {
            // Arithmetic operators
            "+" => self.eval_arithmetic("+", args),
            "-" => self.eval_arithmetic("-", args),
            "*" => self.eval_arithmetic("*", args),
            "/" => self.eval_arithmetic("/", args),

            // Define a variable
            "define" => {
                if args.len() != 2 {
                    return Err(EvalError::SyntaxError("define requires 2 arguments".to_string()));
                }

                let name = match &args[0] {
                    SexprExpr::Symbol(s) => s,
                    _ => {
                        return Err(EvalError::SyntaxError(
                            "First argument to define must be a symbol".to_string(),
                        ))
                    }
                };

                let value = self.eval_expr(&args[1])?;
                self.symbols.insert(name.clone(), value.clone());
                Ok(value)
            }

            // Node construction (for AST)
            "Node" | "Item" | "Expr" | "Type" => {
                // Parse keyword-value pairs
                let mut fields = HashMap::new();
                let mut i = 0;

                while i < args.len() {
                    match &args[i] {
                        SexprExpr::Keyword(k) => {
                            if i + 1 >= args.len() {
                                return Err(EvalError::SyntaxError(format!(
                                    "Keyword :{} missing value",
                                    k
                                )));
                            }
                            let value = self.eval_expr(&args[i + 1])?;
                            fields.insert(k.clone(), value);
                            i += 2;
                        }
                        _ => {
                            return Err(EvalError::SyntaxError(
                                "Expected keyword in node construction".to_string(),
                            ))
                        }
                    }
                }

                Ok(SexprValue::Node { type_name: name.to_string(), fields })
            }

            // Unknown form - treat as list
            _ => {
                let mut values = Vec::new();
                values.push(SexprValue::Symbol(name.to_string()));

                for arg in args {
                    values.push(self.eval_expr(arg)?);
                }

                Ok(SexprValue::List(values))
            }
        }
    }

    /// Evaluate arithmetic operations
    fn eval_arithmetic(&mut self, op: &str, args: &[SexprExpr]) -> Result<SexprValue> {
        if args.is_empty() {
            return Err(EvalError::SyntaxError(format!(
                "Operator '{}' requires at least one argument",
                op
            )));
        }

        // Evaluate all arguments to numbers
        let nums: Result<Vec<i64>> = args
            .iter()
            .map(|e| {
                let val = self.eval_expr(e)?;
                match val {
                    SexprValue::Number(n) => Ok(n),
                    _ => Err(EvalError::TypeError(format!("Expected number, got {:?}", val))),
                }
            })
            .collect();

        let nums = nums?;

        let result = match op {
            "+" => nums.iter().sum(),
            "-" => {
                if nums.len() == 1 {
                    -nums[0]
                } else {
                    nums.iter().skip(1).fold(nums[0], |acc, &x| acc - x)
                }
            }
            "*" => nums.iter().product(),
            "/" => {
                if nums.len() == 1 {
                    return Err(EvalError::RuntimeError(
                        "Division requires at least two arguments".to_string(),
                    ));
                }
                nums.iter().skip(1).try_fold(nums[0], |acc, &x| {
                    if x == 0 {
                        Err(EvalError::RuntimeError("Division by zero".to_string()))
                    } else {
                        Ok(acc / x)
                    }
                })?
            }
            _ => unreachable!(),
        };

        Ok(SexprValue::Number(result))
    }

    /// Convert value to string representation
    #[allow(clippy::only_used_in_recursion)]
    fn value_to_string(&self, value: &SexprValue) -> String {
        match value {
            SexprValue::Number(n) => n.to_string(),
            SexprValue::String(s) => format!("\"{}\"", s),
            SexprValue::Keyword(k) => format!(":{}", k),
            SexprValue::Symbol(s) => s.clone(),
            SexprValue::List(items) => {
                let strs: Vec<_> = items.iter().map(|v| self.value_to_string(v)).collect();
                format!("({})", strs.join(" "))
            }
            SexprValue::Node { type_name, fields } => {
                let mut parts = vec![type_name.clone()];
                for (k, v) in fields {
                    parts.push(format!(":{}", k));
                    parts.push(self.value_to_string(v));
                }
                format!("({})", parts.join(" "))
            }
        }
    }

    /// Parse code to CoreForm using oxur-lang
    pub fn parse_to_core(&mut self, code: &str) -> Result<CoreForm> {
        let expr = self.parse(code)?;
        self.expr_to_core(&expr)
    }

    /// Convert expression to CoreForm
    fn expr_to_core(&mut self, expr: &SexprExpr) -> Result<CoreForm> {
        let id = self.next_node_id();

        match expr {
            SexprExpr::Number(n) => Ok(CoreForm::Number { id, value: *n }),
            SexprExpr::String(s) => Ok(CoreForm::String { id, value: s.clone() }),
            SexprExpr::Symbol(s) => Ok(CoreForm::Symbol { id, name: s.clone() }),
            SexprExpr::Keyword(_) => Err(EvalError::SyntaxError(
                "Keywords not supported in CoreForm conversion".to_string(),
            )),
            SexprExpr::List(elements) => {
                let elements: Result<Vec<_>> =
                    elements.iter().map(|e| self.expr_to_core(e)).collect();
                Ok(CoreForm::List { id, elements: elements? })
            }
        }
    }

    /// Generate next node ID
    fn next_node_id(&mut self) -> NodeId {
        use oxur_smap::new_node_id;
        new_node_id()
    }

    /// Clear the symbol table
    pub fn clear_symbols(&mut self) {
        self.symbols.clear();
    }
}

impl Default for SexprEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// S-expression value types
#[derive(Debug, Clone, PartialEq)]
enum SexprValue {
    Number(i64),
    String(String),
    Keyword(String),
    Symbol(String),
    List(Vec<SexprValue>),
    Node { type_name: String, fields: HashMap<String, SexprValue> },
}

/// S-expression AST
#[derive(Debug, Clone, PartialEq)]
enum SexprExpr {
    Number(i64),
    String(String),
    Keyword(String),
    Symbol(String),
    List(Vec<SexprExpr>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let eval = SexprEvaluator::new();
        let expr = eval.parse("42").unwrap();
        assert_eq!(expr, SexprExpr::Number(42));
    }

    #[test]
    fn test_parse_string() {
        let eval = SexprEvaluator::new();
        let expr = eval.parse("\"hello\"").unwrap();
        assert_eq!(expr, SexprExpr::String("hello".to_string()));
    }

    #[test]
    fn test_parse_keyword() {
        let eval = SexprEvaluator::new();
        let expr = eval.parse(":name").unwrap();
        assert_eq!(expr, SexprExpr::Keyword("name".to_string()));
    }

    #[test]
    fn test_parse_symbol() {
        let eval = SexprEvaluator::new();
        let expr = eval.parse("foo").unwrap();
        assert_eq!(expr, SexprExpr::Symbol("foo".to_string()));
    }

    #[test]
    fn test_parse_empty_list() {
        let eval = SexprEvaluator::new();
        let expr = eval.parse("()").unwrap();
        assert_eq!(expr, SexprExpr::List(vec![]));
    }

    #[test]
    fn test_parse_simple_list() {
        let eval = SexprEvaluator::new();
        let expr = eval.parse("(+ 1 2)").unwrap();
        assert_eq!(
            expr,
            SexprExpr::List(vec![
                SexprExpr::Symbol("+".to_string()),
                SexprExpr::Number(1),
                SexprExpr::Number(2),
            ])
        );
    }

    #[test]
    fn test_eval_number() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("42").unwrap();
        assert_eq!(result, SexprValue::Number(42));
    }

    #[test]
    fn test_eval_string() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("\"test\"").unwrap();
        assert_eq!(result, SexprValue::String("test".to_string()));
    }

    #[test]
    fn test_eval_addition() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("(+ 1 2)").unwrap();
        assert_eq!(result, SexprValue::Number(3));
    }

    #[test]
    fn test_eval_subtraction() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("(- 10 5)").unwrap();
        assert_eq!(result, SexprValue::Number(5));
    }

    #[test]
    fn test_eval_multiplication() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("(* 3 4)").unwrap();
        assert_eq!(result, SexprValue::Number(12));
    }

    #[test]
    fn test_eval_division() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("(/ 10 2)").unwrap();
        assert_eq!(result, SexprValue::Number(5));
    }

    #[test]
    fn test_eval_division_by_zero() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("(/ 10 0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_nested() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("(+ (* 2 3) 4)").unwrap();
        assert_eq!(result, SexprValue::Number(10));
    }

    #[test]
    fn test_define_and_lookup() {
        let mut eval = SexprEvaluator::new();
        eval.eval("(define x 42)").unwrap();
        let result = eval.eval("x").unwrap();
        assert_eq!(result, SexprValue::Number(42));
    }

    #[test]
    fn test_define_with_expression() {
        let mut eval = SexprEvaluator::new();
        eval.eval("(define y (+ 10 20))").unwrap();
        let result = eval.eval("y").unwrap();
        assert_eq!(result, SexprValue::Number(30));
    }

    #[test]
    fn test_node_construction() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("(Node :name \"test\" :value 42)").unwrap();

        match result {
            SexprValue::Node { type_name, fields } => {
                assert_eq!(type_name, "Node");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields.get("name"), Some(&SexprValue::String("test".to_string())));
                assert_eq!(fields.get("value"), Some(&SexprValue::Number(42)));
            }
            _ => panic!("Expected Node value"),
        }
    }

    #[test]
    fn test_calculator_mode() {
        let mut eval = SexprEvaluator::new();
        assert_eq!(eval.try_eval_calculator("(+ 1 2)"), Some("3".to_string()));
        assert_eq!(eval.try_eval_calculator("(* 3 4)"), Some("12".to_string()));
    }

    #[test]
    fn test_tokenize_with_strings() {
        let eval = SexprEvaluator::new();
        let tokens = eval.tokenize("name \"hello world\" 42").unwrap();
        assert_eq!(
            tokens,
            vec!["name".to_string(), "\"hello world\"".to_string(), "42".to_string()]
        );
    }

    #[test]
    fn test_tokenize_nested() {
        let eval = SexprEvaluator::new();
        let tokens = eval.tokenize("+ (- 5 2) 3").unwrap();
        assert_eq!(tokens, vec!["+".to_string(), "(- 5 2)".to_string(), "3".to_string()]);
    }

    #[test]
    fn test_value_to_string_number() {
        let eval = SexprEvaluator::new();
        assert_eq!(eval.value_to_string(&SexprValue::Number(42)), "42");
    }

    #[test]
    fn test_value_to_string_list() {
        let eval = SexprEvaluator::new();
        let value = SexprValue::List(vec![
            SexprValue::Symbol("+".to_string()),
            SexprValue::Number(1),
            SexprValue::Number(2),
        ]);
        assert_eq!(eval.value_to_string(&value), "(+ 1 2)");
    }

    #[test]
    fn test_expr_to_core_number() {
        let mut eval = SexprEvaluator::new();
        let expr = SexprExpr::Number(42);
        let core = eval.expr_to_core(&expr).unwrap();

        match core {
            CoreForm::Number { value, .. } => assert_eq!(value, 42),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_expr_to_core_list() {
        let mut eval = SexprEvaluator::new();
        let expr = SexprExpr::List(vec![
            SexprExpr::Symbol("+".to_string()),
            SexprExpr::Number(1),
            SexprExpr::Number(2),
        ]);
        let core = eval.expr_to_core(&expr).unwrap();

        match core {
            CoreForm::List { elements, .. } => assert_eq!(elements.len(), 3),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_clear_symbols() {
        let mut eval = SexprEvaluator::new();
        eval.eval("(define x 42)").unwrap();
        assert!(eval.symbols.contains_key("x"));

        eval.clear_symbols();
        assert!(!eval.symbols.contains_key("x"));
    }

    #[test]
    fn test_undefined_symbol() {
        let mut eval = SexprEvaluator::new();
        let result = eval.eval("undefined");
        assert!(result.is_err());
    }
}
