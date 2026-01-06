//! Type inference for REPL variables
//!
//! Tracks types of REPL variables across evaluations.
//!
//! # Phase 3 Implementation
//!
//! This is a stub implementation that returns basic type information.
//! Full implementation (Phase 4+) will integrate with rust-analyzer
//! for proper type inference.
//!
//! Based on ODD-0026 Section 2.1 and ODD-0038 Decision 6

use std::collections::HashMap;
use thiserror::Error;

/// Type inference errors
#[derive(Debug, Error)]
pub enum TypeInferenceError {
    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Type inference failed: {0}")]
    InferenceFailed(String),

    #[error("Ambiguous type: {0}")]
    AmbiguousType(String),
}

pub type Result<T> = std::result::Result<T, TypeInferenceError>;

/// Type information for a variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    /// Rust type as a string (e.g., "i32", "String", "Vec<u8>")
    pub type_name: String,

    /// Whether the variable is mutable
    pub is_mutable: bool,

    /// Source location where variable was defined
    pub definition_location: Option<String>,
}

impl TypeInfo {
    /// Create new type info
    pub fn new(type_name: impl Into<String>) -> Self {
        Self { type_name: type_name.into(), is_mutable: false, definition_location: None }
    }

    /// Set mutability
    pub fn with_mutable(mut self, is_mutable: bool) -> Self {
        self.is_mutable = is_mutable;
        self
    }

    /// Set definition location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.definition_location = Some(location.into());
        self
    }
}

/// Type inference engine
///
/// # Phase 3 Stub Implementation
///
/// Currently provides basic type tracking without actual inference.
/// Variables must be explicitly registered with their types.
///
/// # Examples
///
/// ```
/// use oxur_repl::type_inference::{TypeInference, TypeInfo};
///
/// let mut inference = TypeInference::new();
///
/// // Register a variable type
/// inference.register_variable("x", TypeInfo::new("i32"));
///
/// // Look up variable type
/// let type_info = inference.get_variable_type("x").unwrap();
/// assert_eq!(type_info.type_name, "i32");
/// ```
#[derive(Debug, Clone)]
pub struct TypeInference {
    /// Map from variable name to type information
    variables: HashMap<String, TypeInfo>,

    /// Whether to enable debug logging
    debug: bool,
}

impl TypeInference {
    /// Create a new type inference engine
    pub fn new() -> Self {
        Self { variables: HashMap::new(), debug: false }
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Register a variable with its type
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    /// * `type_info` - Type information
    pub fn register_variable(&mut self, name: impl Into<String>, type_info: TypeInfo) {
        let name = name.into();
        if self.debug {
            eprintln!("TypeInference: Registering {} : {}", name, type_info.type_name);
        }
        self.variables.insert(name, type_info);
    }

    /// Get the type of a variable
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    ///
    /// # Returns
    ///
    /// Type information for the variable
    ///
    /// # Errors
    ///
    /// Returns error if variable is not found
    pub fn get_variable_type(&self, name: impl AsRef<str>) -> Result<&TypeInfo> {
        self.variables
            .get(name.as_ref())
            .ok_or_else(|| TypeInferenceError::VariableNotFound(name.as_ref().to_string()))
    }

    /// Infer types from Rust code
    ///
    /// # Phase 3 Stub
    ///
    /// Currently returns empty list. Full implementation will use rust-analyzer
    /// to extract type information from let bindings.
    ///
    /// # Arguments
    ///
    /// * `code` - Rust source code to analyze
    ///
    /// # Returns
    ///
    /// List of (variable_name, type_info) pairs
    pub fn infer_from_code(&self, code: impl AsRef<str>) -> Vec<(String, TypeInfo)> {
        let code = code.as_ref();
        let mut types = Vec::new();

        // Parse the code as a Rust file
        let parsed = match syn::parse_file(code) {
            Ok(file) => file,
            Err(_) => {
                if self.debug {
                    eprintln!("Failed to parse code as complete file, trying as statements");
                }
                // Try parsing as individual statements wrapped in a function
                let wrapped = format!("fn __oxur_wrapper() {{\n{}\n}}", code);
                match syn::parse_file(&wrapped) {
                    Ok(file) => file,
                    Err(_) => return types, // Can't parse, return empty
                }
            }
        };

        // Extract type information from the parsed file
        self.extract_types_from_file(&parsed, &mut types);

        types
    }

    /// Extract type information from a syn::File
    fn extract_types_from_file(&self, file: &syn::File, types: &mut Vec<(String, TypeInfo)>) {
        for item in &file.items {
            self.extract_types_from_item(item, types);
        }
    }

    /// Extract type information from a syn::Item
    fn extract_types_from_item(&self, item: &syn::Item, types: &mut Vec<(String, TypeInfo)>) {
        match item {
            syn::Item::Fn(func) => {
                // Extract from function body
                self.extract_types_from_block(&func.block, types);
            }
            syn::Item::Static(static_item) => {
                // Static variables
                let name = static_item.ident.to_string();
                let ty = &static_item.ty;
                let type_name = quote::quote!(#ty).to_string();
                types.push((
                    name,
                    TypeInfo::new(type_name).with_mutable(matches!(
                        static_item.mutability,
                        syn::StaticMutability::Mut(_)
                    )),
                ));
            }
            syn::Item::Const(const_item) => {
                // Constants
                let name = const_item.ident.to_string();
                let ty = &const_item.ty;
                let type_name = quote::quote!(#ty).to_string();
                types.push((name, TypeInfo::new(type_name)));
            }
            _ => {
                // Other items don't contain local variables
            }
        }
    }

    /// Extract type information from a block
    fn extract_types_from_block(&self, block: &syn::Block, types: &mut Vec<(String, TypeInfo)>) {
        for stmt in &block.stmts {
            self.extract_types_from_stmt(stmt, types);
        }
    }

    /// Extract type information from a statement
    fn extract_types_from_stmt(&self, stmt: &syn::Stmt, types: &mut Vec<(String, TypeInfo)>) {
        match stmt {
            syn::Stmt::Local(local) => {
                self.extract_types_from_local(local, types);
            }
            syn::Stmt::Expr(expr, _) => {
                // Check for nested blocks in expressions (if, loop, etc.)
                self.extract_types_from_expr(expr, types);
            }
            _ => {}
        }
    }

    /// Extract type information from a local variable declaration
    fn extract_types_from_local(&self, local: &syn::Local, types: &mut Vec<(String, TypeInfo)>) {
        // Check if pattern has explicit type annotation (let x: T = ...)
        let (pat, explicit_type) = match &local.pat {
            syn::Pat::Type(pat_type) => {
                // Pattern with type annotation
                let ty = &pat_type.ty;
                let ty_str = quote::quote!(#ty).to_string();
                (&*pat_type.pat, Some(ty_str))
            }
            other_pat => (other_pat, None),
        };

        // Extract variable name from pattern
        if let syn::Pat::Ident(pat_ident) = pat {
            let name = pat_ident.ident.to_string();
            let is_mutable = pat_ident.mutability.is_some();

            // Determine type: explicit annotation > inferred from init > unknown
            let type_name = if let Some(ty) = explicit_type {
                ty
            } else if let Some(init) = &local.init {
                // Try to infer from initialization expression
                self.infer_type_from_expr(&init.expr)
            } else {
                "unknown".to_string()
            };

            types.push((name, TypeInfo::new(type_name).with_mutable(is_mutable)));
        }
    }

    /// Extract types from expressions (for nested blocks)
    fn extract_types_from_expr(&self, expr: &syn::Expr, types: &mut Vec<(String, TypeInfo)>) {
        match expr {
            syn::Expr::Block(block_expr) => {
                self.extract_types_from_block(&block_expr.block, types);
            }
            syn::Expr::If(if_expr) => {
                self.extract_types_from_block(&if_expr.then_branch, types);
                if let Some((_, else_branch)) = &if_expr.else_branch {
                    self.extract_types_from_expr(else_branch, types);
                }
            }
            syn::Expr::Loop(loop_expr) => {
                self.extract_types_from_block(&loop_expr.body, types);
            }
            syn::Expr::While(while_expr) => {
                self.extract_types_from_block(&while_expr.body, types);
            }
            syn::Expr::ForLoop(for_expr) => {
                self.extract_types_from_block(&for_expr.body, types);
            }
            syn::Expr::Match(match_expr) => {
                for arm in &match_expr.arms {
                    self.extract_types_from_expr(&arm.body, types);
                }
            }
            _ => {}
        }
    }

    /// Attempt to infer type from an expression
    fn infer_type_from_expr(&self, expr: &syn::Expr) -> String {
        match expr {
            syn::Expr::Lit(lit_expr) => match &lit_expr.lit {
                syn::Lit::Str(_) => "&str".to_string(),
                syn::Lit::ByteStr(_) => "&[u8]".to_string(),
                syn::Lit::Byte(_) => "u8".to_string(),
                syn::Lit::Char(_) => "char".to_string(),
                syn::Lit::Int(int_lit) => {
                    // Check suffix
                    let suffix = int_lit.suffix();
                    if suffix.is_empty() {
                        "i32".to_string() // Default integer type
                    } else {
                        suffix.to_string()
                    }
                }
                syn::Lit::Float(float_lit) => {
                    let suffix = float_lit.suffix();
                    if suffix.is_empty() {
                        "f64".to_string() // Default float type
                    } else {
                        suffix.to_string()
                    }
                }
                syn::Lit::Bool(_) => "bool".to_string(),
                _ => "unknown".to_string(),
            },
            syn::Expr::Array(_) => "array".to_string(),
            syn::Expr::Tuple(_) => "tuple".to_string(),
            syn::Expr::Call(call) => {
                // Try to get type from function name
                if let syn::Expr::Path(path) = &*call.func {
                    if let Some(segment) = path.path.segments.last() {
                        let fn_name = segment.ident.to_string();
                        // Common constructors
                        return match fn_name.as_str() {
                            "String" => "String".to_string(),
                            "Vec" => "Vec<_>".to_string(),
                            "HashMap" => "HashMap<_, _>".to_string(),
                            "Box" => "Box<_>".to_string(),
                            _ => "unknown".to_string(),
                        };
                    }
                }
                "unknown".to_string()
            }
            syn::Expr::MethodCall(method) => {
                // Common method patterns
                match method.method.to_string().as_str() {
                    "to_string" => "String".to_string(),
                    "to_vec" => "Vec<_>".to_string(),
                    "clone" => "unknown".to_string(),
                    _ => "unknown".to_string(),
                }
            }
            _ => "unknown".to_string(),
        }
    }

    /// Get all tracked variables
    pub fn all_variables(&self) -> impl Iterator<Item = (&String, &TypeInfo)> {
        self.variables.iter()
    }

    /// Check if a variable is tracked
    pub fn has_variable(&self, name: impl AsRef<str>) -> bool {
        self.variables.contains_key(name.as_ref())
    }

    /// Remove a variable from tracking
    pub fn remove_variable(&mut self, name: impl AsRef<str>) -> Option<TypeInfo> {
        self.variables.remove(name.as_ref())
    }

    /// Clear all tracked variables
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Get count of tracked variables
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_info_creation() {
        let info = TypeInfo::new("i32");
        assert_eq!(info.type_name, "i32");
        assert!(!info.is_mutable);
        assert!(info.definition_location.is_none());
    }

    #[test]
    fn test_type_info_with_mutable() {
        let info = TypeInfo::new("String").with_mutable(true);
        assert_eq!(info.type_name, "String");
        assert!(info.is_mutable);
    }

    #[test]
    fn test_type_info_with_location() {
        let info = TypeInfo::new("Vec<u8>").with_location("line 42");
        assert_eq!(info.type_name, "Vec<u8>");
        assert_eq!(info.definition_location, Some("line 42".to_string()));
    }

    #[test]
    fn test_inference_creation() {
        let inference = TypeInference::new();
        assert_eq!(inference.variable_count(), 0);
        assert!(!inference.debug);
    }

    #[test]
    fn test_inference_with_debug() {
        let inference = TypeInference::new().with_debug(true);
        assert!(inference.debug);
    }

    #[test]
    fn test_register_and_get_variable() {
        let mut inference = TypeInference::new();

        inference.register_variable("x", TypeInfo::new("i32"));

        let type_info = inference.get_variable_type("x").expect("Variable not found");
        assert_eq!(type_info.type_name, "i32");
    }

    #[test]
    fn test_get_nonexistent_variable() {
        let inference = TypeInference::new();

        let result = inference.get_variable_type("nonexistent");
        assert!(result.is_err());

        if let Err(TypeInferenceError::VariableNotFound(name)) = result {
            assert_eq!(name, "nonexistent");
        }
    }

    #[test]
    fn test_register_multiple_variables() {
        let mut inference = TypeInference::new();

        inference.register_variable("x", TypeInfo::new("i32"));
        inference.register_variable("y", TypeInfo::new("String"));
        inference.register_variable("z", TypeInfo::new("bool"));

        assert_eq!(inference.variable_count(), 3);
    }

    #[test]
    fn test_has_variable() {
        let mut inference = TypeInference::new();

        inference.register_variable("x", TypeInfo::new("i32"));

        assert!(inference.has_variable("x"));
        assert!(!inference.has_variable("y"));
    }

    #[test]
    fn test_remove_variable() {
        let mut inference = TypeInference::new();

        inference.register_variable("x", TypeInfo::new("i32"));
        assert_eq!(inference.variable_count(), 1);

        let removed = inference.remove_variable("x");
        assert!(removed.is_some());
        assert_eq!(inference.variable_count(), 0);
    }

    #[test]
    fn test_clear() {
        let mut inference = TypeInference::new();

        inference.register_variable("x", TypeInfo::new("i32"));
        inference.register_variable("y", TypeInfo::new("String"));

        assert_eq!(inference.variable_count(), 2);

        inference.clear();
        assert_eq!(inference.variable_count(), 0);
    }

    #[test]
    fn test_all_variables() {
        let mut inference = TypeInference::new();

        inference.register_variable("x", TypeInfo::new("i32"));
        inference.register_variable("y", TypeInfo::new("String"));

        let all: Vec<_> = inference.all_variables().collect();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_infer_from_code_explicit_types() {
        let inference = TypeInference::new();

        let code = r#"
fn main() {
    let x: i32 = 42;
    let y: String = "hello".to_string();
}
"#;
        let inferred = inference.infer_from_code(code);

        assert_eq!(inferred.len(), 2);
        assert_eq!(inferred[0].0, "x");
        assert_eq!(inferred[0].1.type_name, "i32");
        assert!(!inferred[0].1.is_mutable);

        assert_eq!(inferred[1].0, "y");
        assert_eq!(inferred[1].1.type_name, "String");
        assert!(!inferred[1].1.is_mutable);
    }

    #[test]
    fn test_infer_from_code_inferred_literal() {
        let inference = TypeInference::new();

        let code = r#"
fn test() {
    let x = 42;
    let y = 3.14;
    let z = true;
    let s = "hello";
}
"#;
        let inferred = inference.infer_from_code(code);

        assert_eq!(inferred.len(), 4);
        assert_eq!(inferred[0].1.type_name, "i32"); // Default int
        assert_eq!(inferred[1].1.type_name, "f64"); // Default float
        assert_eq!(inferred[2].1.type_name, "bool");
        assert_eq!(inferred[3].1.type_name, "&str");
    }

    #[test]
    fn test_infer_from_code_mutable() {
        let inference = TypeInference::new();

        let code = r#"
fn test() {
    let mut x: i32 = 42;
    let y: i32 = 10;
}
"#;
        let inferred = inference.infer_from_code(code);

        assert_eq!(inferred.len(), 2);
        assert!(inferred[0].1.is_mutable);
        assert!(!inferred[1].1.is_mutable);
    }

    #[test]
    fn test_infer_from_code_typed_suffix() {
        let inference = TypeInference::new();

        let code = r#"
fn test() {
    let x = 42u64;
    let y = 3.14f32;
}
"#;
        let inferred = inference.infer_from_code(code);

        assert_eq!(inferred.len(), 2);
        assert_eq!(inferred[0].1.type_name, "u64");
        assert_eq!(inferred[1].1.type_name, "f32");
    }

    #[test]
    fn test_infer_from_code_method_calls() {
        let inference = TypeInference::new();

        let code = r#"
fn test() {
    let s = "hello".to_string();
    let v = vec![1, 2, 3].to_vec();
}
"#;
        let inferred = inference.infer_from_code(code);

        assert_eq!(inferred.len(), 2);
        assert_eq!(inferred[0].1.type_name, "String");
        assert_eq!(inferred[1].1.type_name, "Vec<_>");
    }

    #[test]
    fn test_infer_from_code_nested_blocks() {
        let inference = TypeInference::new();

        let code = r#"
fn test() {
    let x = 1;
    if true {
        let y = 2;
    }
    for i in 0..10 {
        let z = 3;
    }
}
"#;
        let inferred = inference.infer_from_code(code);

        assert_eq!(inferred.len(), 3);
        assert_eq!(inferred[0].0, "x");
        assert_eq!(inferred[1].0, "y");
        assert_eq!(inferred[2].0, "z");
    }

    #[test]
    fn test_infer_from_code_invalid_syntax() {
        let inference = TypeInference::new();

        let code = "this is not valid rust code {{{";
        let inferred = inference.infer_from_code(code);

        // Should return empty on parse failure
        assert_eq!(inferred.len(), 0);
    }

    #[test]
    fn test_infer_from_code_wrapped_statements() {
        let inference = TypeInference::new();

        // Code without function wrapper (should still parse)
        let code = "let x: i32 = 42;\nlet y: bool = true;";
        let inferred = inference.infer_from_code(code);

        // Should successfully parse by wrapping in function
        assert!(!inferred.is_empty()); // May find variables
    }

    #[test]
    fn test_infer_from_code_constants() {
        let inference = TypeInference::new();

        let code = r#"
const MAX: usize = 100;
static mut COUNTER: i32 = 0;
"#;
        let inferred = inference.infer_from_code(code);

        assert!(!inferred.is_empty());
        // Should find at least the constant or static
    }

    #[test]
    fn test_default() {
        let inference1 = TypeInference::default();
        let inference2 = TypeInference::new();

        assert_eq!(inference1.variable_count(), inference2.variable_count());
        assert_eq!(inference1.debug, inference2.debug);
    }

    #[test]
    fn test_overwrite_variable_type() {
        let mut inference = TypeInference::new();

        inference.register_variable("x", TypeInfo::new("i32"));
        assert_eq!(inference.get_variable_type("x").unwrap().type_name, "i32");

        // Overwrite with different type
        inference.register_variable("x", TypeInfo::new("String"));
        assert_eq!(inference.get_variable_type("x").unwrap().type_name, "String");
    }
}
