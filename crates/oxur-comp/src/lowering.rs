//! Stage 3: Lower
//!
//! Converts Core Forms into Rust AST using the syn crate.

use crate::Result;
use oxur_lang::{CoreForm, NodeId};
use std::collections::HashMap;
use syn::{Block, FnArg};

/// Lowerer converts Core Forms to Rust AST
pub struct Lowerer {
    source_map: oxur_smap::SourceMap,
    #[allow(dead_code)] // Maintained for potential future use
    node_map: HashMap<NodeId, syn::Expr>,
}

impl Lowerer {
    pub fn new(source_map: oxur_smap::SourceMap) -> Self {
        Self { source_map, node_map: HashMap::new() }
    }

    /// Lower Core Forms to Rust AST
    ///
    /// Returns the generated syn::File and the complete SourceMap with both
    /// Surface → Core mappings (from expansion) and Core → Rust mappings (from lowering).
    pub fn lower(&mut self, forms: Vec<CoreForm>) -> Result<(syn::File, oxur_smap::SourceMap)> {
        let mut items = Vec::new();

        for form in forms {
            items.push(self.lower_top_level(form)?);
        }

        // Freeze the source map (no more modifications)
        self.source_map.freeze();

        Ok((syn::File { shebang: None, attrs: vec![], items }, self.source_map.clone()))
    }

    fn lower_top_level(&mut self, form: CoreForm) -> Result<syn::Item> {
        match form {
            CoreForm::DefineFunc { name, params, body, id } => {
                self.lower_function(name, params, *body, id)
            }
            _ => Err(crate::Error::Lowering(
                "Only function definitions are supported at top level".to_string(),
            )),
        }
    }

    fn lower_function(
        &mut self,
        name: String,
        params: Vec<String>,
        body: CoreForm,
        id: NodeId,
    ) -> Result<syn::Item> {
        use quote::format_ident;
        use syn::{ItemFn, ReturnType, Signature};

        // Generate virtual Rust NodeId for this function
        let rust_id = oxur_smap::new_node_id();
        self.source_map.record_lowering(id, rust_id);

        // Create function signature
        let fn_name = format_ident!("{}", name);
        let inputs = self.lower_params(params)?;

        let sig = Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: fn_name,
            generics: Default::default(),
            paren_token: Default::default(),
            inputs,
            variadic: None,
            output: ReturnType::Default,
        };

        // Create function body
        let block = self.lower_block(body)?;

        Ok(syn::Item::Fn(ItemFn {
            attrs: vec![],
            vis: syn::Visibility::Inherited,
            sig,
            block: Box::new(block),
        }))
    }

    fn lower_params(
        &self,
        _params: Vec<String>,
    ) -> Result<syn::punctuated::Punctuated<FnArg, syn::Token![,]>> {
        // For now, empty params (we'll handle typed params later)
        Ok(syn::punctuated::Punctuated::new())
    }

    fn lower_block(&mut self, body: CoreForm) -> Result<Block> {
        use syn::parse_quote;

        // Convert the body CoreForm into a statement
        let stmt = self.lower_to_stmt(body)?;

        Ok(parse_quote! {
            {
                #stmt
            }
        })
    }

    fn lower_to_stmt(&mut self, form: CoreForm) -> Result<syn::Stmt> {
        match form {
            CoreForm::List { elements, id } => {
                // Generate virtual Rust NodeId for this list
                let rust_id = oxur_smap::new_node_id();
                self.source_map.record_lowering(id, rust_id);

                // Check if this is a macro call (like println!)
                if !elements.is_empty() {
                    if let CoreForm::Symbol { name, .. } = &elements[0] {
                        if name.ends_with('!') {
                            return self.lower_macro_call(name.clone(), elements[1..].to_vec());
                        }
                    }
                }
                Err(crate::Error::Lowering("Unsupported list form".to_string()))
            }
            _ => Err(crate::Error::Lowering(
                "Only macro calls supported in function body for now".to_string(),
            )),
        }
    }

    fn lower_macro_call(&mut self, macro_name: String, args: Vec<CoreForm>) -> Result<syn::Stmt> {
        use quote::format_ident;
        use syn::{parse_quote, StmtMacro};

        // Remove the '!' from macro name
        let name = macro_name.trim_end_matches('!');
        let macro_ident = format_ident!("{}", name);

        // Convert arguments to token stream
        let arg_tokens = self.lower_macro_args(args)?;

        // Create the macro call
        let mac: syn::Macro = parse_quote! {
            #macro_ident!(#arg_tokens)
        };

        Ok(syn::Stmt::Macro(StmtMacro { attrs: vec![], mac, semi_token: Some(Default::default()) }))
    }

    fn lower_macro_args(&mut self, args: Vec<CoreForm>) -> Result<proc_macro2::TokenStream> {
        use quote::quote;

        if args.is_empty() {
            return Ok(quote! {});
        }

        // For now, just handle a single string argument (for println!)
        if args.len() == 1 {
            if let CoreForm::String { value, id } = &args[0] {
                // Generate virtual Rust NodeId for this string literal
                let rust_id = oxur_smap::new_node_id();
                self.source_map.record_lowering(*id, rust_id);

                let string_lit = value.as_str();
                return Ok(quote! { #string_lit });
            }
        }

        Err(crate::Error::Lowering("Only single string arguments supported for macros".to_string()))
    }
}

// Note: No Default implementation - Lowerer requires a SourceMap

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowerer_creation() {
        let source_map = oxur_smap::SourceMap::new();
        let lowerer = Lowerer::new(source_map);
        assert_eq!(lowerer.node_map.len(), 0);
    }

    #[test]
    fn test_lower_empty() {
        let source_map = oxur_smap::SourceMap::new();
        let mut lowerer = Lowerer::new(source_map);
        let result = lowerer.lower(vec![]);
        assert!(result.is_ok());
        let (file, _source_map) = result.unwrap();
        assert_eq!(file.items.len(), 0);
    }

    #[test]
    fn test_lower_returns_syn_file() {
        let source_map = oxur_smap::SourceMap::new();
        let mut lowerer = Lowerer::new(source_map);
        let result = lowerer.lower(vec![]);
        assert!(result.is_ok());
        let (file, _source_map) = result.unwrap();
        assert!(file.shebang.is_none());
        assert_eq!(file.attrs.len(), 0);
    }

    #[test]
    fn test_lower_hello_world() {
        use oxur_lang::{Expander, Parser};

        // Parse and expand the Oxur code
        let source = r#"(deffn main ()
  (println! "Hello, world!"))"#;

        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(surface_forms).unwrap();
        let source_map = expander.source_map().clone();

        // Lower to Rust AST
        let mut lowerer = Lowerer::new(source_map);
        let result = lowerer.lower(core_forms);

        assert!(result.is_ok());
        let (file, _source_map) = result.unwrap();

        // Should have one item (the main function)
        assert_eq!(file.items.len(), 1);

        // Check it's a function
        if let syn::Item::Fn(item_fn) = &file.items[0] {
            assert_eq!(item_fn.sig.ident.to_string(), "main");
            assert_eq!(item_fn.sig.inputs.len(), 0); // No parameters

            // Should have one statement in the body
            assert_eq!(item_fn.block.stmts.len(), 1);

            // Should be a macro call
            if let syn::Stmt::Macro(stmt_mac) = &item_fn.block.stmts[0] {
                // Check it's println!
                let path = &stmt_mac.mac.path;
                assert_eq!(quote::quote!(#path).to_string(), "println");
            } else {
                panic!("Expected macro statement");
            }
        } else {
            panic!("Expected function item");
        }
    }

    #[test]
    fn test_source_map_function_mapping() {
        use oxur_lang::{Expander, Parser};

        let source = r#"(deffn main ()
  (println! "Hello"))"#;
        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(surface_forms).unwrap();
        let source_map = expander.source_map().clone();

        // Lower with the source map
        let mut lowerer = Lowerer::new(source_map);
        let result = lowerer.lower(core_forms);
        assert!(result.is_ok(), "Lowering failed: {:?}", result.err());

        let (_file, source_map) = result.unwrap();
        let stats = source_map.stats();

        // Should have surface nodes from Stage 1.7
        assert!(stats.surface_nodes > 0, "Should have surface node mappings");

        // Should have lowering mappings from Stage 1.8
        assert!(stats.lowerings > 0, "Should have lowering mappings");
    }

    #[test]
    fn test_source_map_preserved_through_lowering() {
        use oxur_lang::{Expander, Parser};

        let source = r#"(deffn main ()
  (println! "Hello, world!"))"#;

        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(surface_forms).unwrap();

        // Get the Core NodeId for verification
        let core_id = if let oxur_lang::CoreForm::DefineFunc { id, .. } = &core_forms[0] {
            *id
        } else {
            panic!("Expected DefineFunc");
        };

        let source_map = expander.source_map().clone();
        let surface_pos = source_map.get_surface_position(&core_id);
        assert!(surface_pos.is_some(), "Core node should have surface position");

        // Lower and verify lowering mapping added
        let mut lowerer = Lowerer::new(source_map);
        let result = lowerer.lower(core_forms);
        assert!(result.is_ok());

        let (_file, final_source_map) = result.unwrap();

        // Should still have the surface position
        let surface_pos_after = final_source_map.get_surface_position(&core_id);
        assert!(surface_pos_after.is_some(), "Surface position should be preserved");

        // Should have lowering mapping
        let stats = final_source_map.stats();
        assert!(stats.lowerings > 0, "Should have lowering mappings");
    }

    #[test]
    fn test_source_map_frozen_after_lowering() {
        use oxur_lang::{Expander, Parser};

        let source = r#"(deffn main ()
  (println! "Test"))"#;
        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(surface_forms).unwrap();
        let source_map = expander.source_map().clone();

        let mut lowerer = Lowerer::new(source_map);
        let result = lowerer.lower(core_forms);
        assert!(result.is_ok(), "Lowering failed: {:?}", result.err());

        let (_file, source_map) = result.unwrap();
        assert!(source_map.is_frozen(), "SourceMap should be frozen after lowering");
    }
}
