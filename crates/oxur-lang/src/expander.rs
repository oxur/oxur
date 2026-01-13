//! Stage 2: Expand
//!
//! Converts Surface Forms into Core Forms through macro expansion and desugaring.
//! This is where syntactic sugar gets transformed into canonical forms.

use crate::core_forms::CoreForm;
use crate::parser::SurfaceForm;
use crate::Result;
use oxur_smap::SourceMap;

/// Expander handles macro expansion and desugaring
pub struct Expander {
    source_map: SourceMap,
}

impl Expander {
    pub fn new() -> Self {
        Self { source_map: SourceMap::new() }
    }

    /// Expand Surface Forms into Core Forms
    pub fn expand(&mut self, forms: Vec<SurfaceForm>) -> Result<Vec<CoreForm>> {
        let mut core_forms = Vec::new();

        for form in forms {
            core_forms.push(self.expand_form(form)?);
        }

        Ok(core_forms)
    }

    fn expand_form(&mut self, form: SurfaceForm) -> Result<CoreForm> {
        match form {
            SurfaceForm::Symbol { name, span } => {
                let id = oxur_smap::new_node_id();
                let pos = Self::span_to_source_pos(&span);
                self.source_map.record_surface_node(id, pos);
                Ok(CoreForm::Symbol { id, name })
            }
            SurfaceForm::Number { value, span } => {
                let id = oxur_smap::new_node_id();
                let pos = Self::span_to_source_pos(&span);
                self.source_map.record_surface_node(id, pos);
                Ok(CoreForm::Number { id, value })
            }
            SurfaceForm::String { value, span } => {
                let id = oxur_smap::new_node_id();
                let pos = Self::span_to_source_pos(&span);
                self.source_map.record_surface_node(id, pos);
                Ok(CoreForm::String { id, value })
            }
            SurfaceForm::List { elements, span } => {
                // Check if this is a special form (like deffn)
                if !elements.is_empty() {
                    if let SurfaceForm::Symbol { ref name, .. } = elements[0] {
                        if name == "deffn" {
                            return self.expand_deffn(elements, span);
                        }
                    }
                }

                // Otherwise, expand as a regular list
                self.expand_list(elements, span)
            }
        }
    }

    /// Convert a Span to SourcePos, using start position for multi-line spans
    fn span_to_source_pos(span: &oxur_smap::Span) -> oxur_smap::SourcePos {
        // Use start position and length 1 (exact length not critical for error reporting)
        oxur_smap::SourcePos::new(
            span.file.clone(),
            span.start_line,
            span.start_column,
            1, // Length placeholder
        )
    }

    fn expand_deffn(
        &mut self,
        elements: Vec<SurfaceForm>,
        span: oxur_smap::Span,
    ) -> Result<CoreForm> {
        // (deffn name params body...)
        if elements.len() < 4 {
            return Err(crate::Error::Syntax(
                "deffn requires at least name, params, and body".to_string(),
            ));
        }

        let id = oxur_smap::new_node_id();
        let pos = Self::span_to_source_pos(&span);
        self.source_map.record_surface_node(id, pos);

        // Extract function name
        let name = match &elements[1] {
            SurfaceForm::Symbol { name, .. } => name.clone(),
            _ => return Err(crate::Error::Syntax("deffn name must be a symbol".to_string())),
        };

        // Extract parameters (empty list for now, we'll handle typed params later)
        let params = match &elements[2] {
            SurfaceForm::List { elements: param_list, .. } => {
                let mut params = Vec::new();
                for param in param_list {
                    if let SurfaceForm::Symbol { name, .. } = param {
                        params.push(name.clone());
                    } else {
                        return Err(crate::Error::Syntax("Parameter must be a symbol".to_string()));
                    }
                }
                params
            }
            _ => return Err(crate::Error::Syntax("deffn params must be a list".to_string())),
        };

        // Body is the remaining forms (for now, just take the first body form)
        let body = if elements.len() > 3 {
            Box::new(self.expand_form(elements[3].clone())?)
        } else {
            return Err(crate::Error::Syntax("deffn requires a body".to_string()));
        };

        Ok(CoreForm::DefineFunc { id, name, params, body })
    }

    fn expand_list(
        &mut self,
        elements: Vec<SurfaceForm>,
        span: oxur_smap::Span,
    ) -> Result<CoreForm> {
        let id = oxur_smap::new_node_id();
        let pos = Self::span_to_source_pos(&span);
        self.source_map.record_surface_node(id, pos);

        let mut expanded_elements = Vec::new();
        for element in elements {
            expanded_elements.push(self.expand_form(element)?);
        }

        Ok(CoreForm::List { id, elements: expanded_elements })
    }

    /// Get the source map after expansion
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Check if the source map is empty
    pub fn is_empty(&self) -> bool {
        let stats = self.source_map.stats();
        stats.surface_nodes == 0 && stats.expansions == 0 && stats.lowerings == 0
    }
}

impl Default for Expander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expander_creation() {
        let expander = Expander::new();
        assert!(expander.is_empty());
    }

    #[test]
    fn test_expander_default() {
        let expander = Expander::default();
        assert!(expander.is_empty());
    }

    #[test]
    fn test_expand_empty() {
        let mut expander = Expander::new();
        let result = expander.expand(vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_source_map_access() {
        let expander = Expander::new();
        let _map = expander.source_map();
        assert!(expander.is_empty());
    }

    #[test]
    fn test_expand_deffn_hello_world() {
        use crate::parser::Parser;

        let source = r#"(deffn main ()
  (println! "Hello, world!"))"#;

        let mut parser = Parser::new(source.to_string());
        let surface_forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(surface_forms).unwrap();

        assert_eq!(core_forms.len(), 1);

        // Should be a DefineFunc
        if let CoreForm::DefineFunc { name, params, body, .. } = &core_forms[0] {
            assert_eq!(name, "main");
            assert_eq!(params.len(), 0); // No parameters

            // Body should be a list (println! "Hello, world!")
            if let CoreForm::List { elements, .. } = &**body {
                assert_eq!(elements.len(), 2);

                // First element should be println! symbol
                if let CoreForm::Symbol { name, .. } = &elements[0] {
                    assert_eq!(name, "println!");
                } else {
                    panic!("Expected Symbol(println!)");
                }

                // Second element should be the string
                if let CoreForm::String { value, .. } = &elements[1] {
                    assert_eq!(value, "Hello, world!");
                } else {
                    panic!("Expected String");
                }
            } else {
                panic!("Expected List as body");
            }
        } else {
            panic!("Expected DefineFunc");
        }
    }

    #[test]
    fn test_expand_symbol() {
        use oxur_smap::Span;

        let mut expander = Expander::new();
        let span = Span::repl(1, 1, 1, 5);
        let surface = SurfaceForm::Symbol { span, name: "test".to_string() };
        let result = expander.expand_form(surface).unwrap();

        if let CoreForm::Symbol { name, .. } = result {
            assert_eq!(name, "test");
        } else {
            panic!("Expected Symbol");
        }
    }

    #[test]
    fn test_expand_number() {
        use oxur_smap::Span;

        let mut expander = Expander::new();
        let span = Span::repl(1, 1, 1, 3);
        let surface = SurfaceForm::Number { span, value: 42 };
        let result = expander.expand_form(surface).unwrap();

        if let CoreForm::Number { value, .. } = result {
            assert_eq!(value, 42);
        } else {
            panic!("Expected Number");
        }
    }

    #[test]
    fn test_expand_string() {
        use oxur_smap::Span;

        let mut expander = Expander::new();
        let span = Span::repl(1, 1, 1, 7);
        let surface = SurfaceForm::String { span, value: "hello".to_string() };
        let result = expander.expand_form(surface).unwrap();

        if let CoreForm::String { value, .. } = result {
            assert_eq!(value, "hello");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_source_map_symbol() {
        use crate::parser::Parser;

        let source = "hello";
        let mut parser = Parser::new(source.to_string());
        let forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(forms).unwrap();

        // Get the NodeId from the CoreForm
        if let CoreForm::Symbol { id, .. } = &core_forms[0] {
            // Look up the source position via the SourceMap
            let source_map = expander.source_map();
            let pos = source_map.get_surface_position(id);

            assert!(pos.is_some(), "Mapping should be recorded");
            let pos = pos.unwrap();
            assert_eq!(pos.line, 1);
            assert_eq!(pos.column, 1);
        } else {
            panic!("Expected Symbol");
        }
    }

    #[test]
    fn test_source_map_number() {
        use crate::parser::Parser;

        let source = "42";
        let mut parser = Parser::new(source.to_string());
        let forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(forms).unwrap();

        if let CoreForm::Number { id, .. } = &core_forms[0] {
            let source_map = expander.source_map();
            let pos = source_map.get_surface_position(id);

            assert!(pos.is_some());
            let pos = pos.unwrap();
            assert_eq!(pos.line, 1);
            assert_eq!(pos.column, 1);
        } else {
            panic!("Expected Number");
        }
    }

    #[test]
    fn test_source_map_list() {
        use crate::parser::Parser;

        let source = "(+ 1 2)";
        let mut parser = Parser::new(source.to_string());
        let forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(forms).unwrap();

        if let CoreForm::List { id, elements, .. } = &core_forms[0] {
            let source_map = expander.source_map();

            // Check list node mapping
            let list_pos = source_map.get_surface_position(id);
            assert!(list_pos.is_some());
            assert_eq!(list_pos.unwrap().line, 1);

            // Check first element ('+' symbol) mapping
            if let CoreForm::Symbol { id: elem_id, .. } = &elements[0] {
                let elem_pos = source_map.get_surface_position(elem_id);
                assert!(elem_pos.is_some());
                assert_eq!(elem_pos.unwrap().column, 2); // After '('
            } else {
                panic!("Expected Symbol");
            }
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_source_map_deffn() {
        use crate::parser::Parser;

        let source = "(deffn main () 42)";
        let mut parser = Parser::new(source.to_string());
        let forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let core_forms = expander.expand(forms).unwrap();

        if let CoreForm::DefineFunc { id, .. } = &core_forms[0] {
            let source_map = expander.source_map();
            let pos = source_map.get_surface_position(id);

            assert!(pos.is_some());
            assert_eq!(pos.unwrap().line, 1);
            assert_eq!(pos.unwrap().column, 1);
        } else {
            panic!("Expected DefineFunc");
        }
    }

    #[test]
    fn test_source_map_stats() {
        use crate::parser::Parser;

        let source = "(+ 1 2)"; // 4 nodes: list, +, 1, 2
        let mut parser = Parser::new(source.to_string());
        let forms = parser.parse().unwrap();

        let mut expander = Expander::new();
        let _core_forms = expander.expand(forms).unwrap();

        let source_map = expander.source_map();
        let stats = source_map.stats();

        // Should have 4 surface nodes recorded
        assert_eq!(stats.surface_nodes, 4);
        assert_eq!(stats.expansions, 0); // No expansion chains yet (Stage 1.7 only)
        assert_eq!(stats.lowerings, 0); // No lowerings yet
    }
}
