use super::helpers::*;
use crate::ast::*;
use crate::error::{ParseError, Result};
use crate::sexp::SExp;

pub struct AstBuilder {
    next_id: u32,
}

impl AstBuilder {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        NodeId(id)
    }

    pub fn build_crate(&mut self, sexp: &SExp) -> Result<Crate> {
        let list = expect_list(sexp)?;

        if list.elements.is_empty() {
            return Err(ParseError::Expected {
                expected: "Crate node".to_string(),
                found: "empty list".to_string(),
                pos: list.pos,
            });
        }

        let node_type = expect_symbol(&list.elements[0])?;
        if node_type.value != "Crate" {
            return Err(ParseError::Expected {
                expected: "Crate".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        // Parse items
        let items_sexp = kwargs.get("items").ok_or_else(|| ParseError::Expected {
            expected: ":items field".to_string(),
            found: "missing field".to_string(),
            pos: list.pos,
        })?;

        let items = self.build_items_list(items_sexp)?;

        // Parse spans (simplified for Phase 1)
        let spans = if let Some(spans_sexp) = kwargs.get("spans") {
            self.build_mod_spans(spans_sexp)?
        } else {
            ModSpans::new(Span::DUMMY)
        };

        // Parse id
        let id = if let Some(id_sexp) = kwargs.get("id") {
            NodeId(expect_number(id_sexp)? as u32)
        } else {
            self.next_id()
        };

        Ok(Crate::new(items, spans, id))
    }

    fn build_items_list(&mut self, sexp: &SExp) -> Result<Vec<Item>> {
        let list = expect_list(sexp)?;
        let mut items = Vec::new();

        for element in &list.elements {
            items.push(self.build_item(element)?);
        }

        Ok(items)
    }

    fn build_mod_spans(&mut self, sexp: &SExp) -> Result<ModSpans> {
        let list = expect_list(sexp)?;
        let kwargs = parse_kwargs(list)?;

        let inner_span = if let Some(inner_sexp) = kwargs.get("inner-span") {
            self.build_span(inner_sexp)?
        } else {
            Span::DUMMY
        };

        Ok(ModSpans::new(inner_span))
    }

    pub(super) fn build_span(&mut self, sexp: &SExp) -> Result<Span> {
        let list = expect_list(sexp)?;

        if list.elements.is_empty() {
            return Ok(Span::DUMMY);
        }

        let node_type = expect_symbol(&list.elements[0])?;
        if node_type.value != "Span" {
            return Err(ParseError::Expected {
                expected: "Span".to_string(),
                found: node_type.value.clone(),
                pos: node_type.pos,
            });
        }

        let kwargs = parse_kwargs(list)?;

        let lo =
            kwargs.get("lo").map(|s| expect_number(s).map(|n| n as u32)).transpose()?.unwrap_or(0);

        let hi =
            kwargs.get("hi").map(|s| expect_number(s).map(|n| n as u32)).transpose()?.unwrap_or(0);

        Ok(Span::new(lo, hi))
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexp::{Keyword, List, Number, Symbol};
    use crate::Position;

    fn dummy_pos() -> Position {
        Position { line: 1, column: 1, offset: 0 }
    }

    #[test]
    fn test_new() {
        let builder = AstBuilder::new();
        assert_eq!(builder.next_id, 0);
    }

    #[test]
    fn test_default() {
        let builder = AstBuilder::default();
        assert_eq!(builder.next_id, 0);
    }

    #[test]
    fn test_next_id_sequential() {
        let mut builder = AstBuilder::new();

        let id1 = builder.next_id();
        assert_eq!(id1.0, 0);

        let id2 = builder.next_id();
        assert_eq!(id2.0, 1);

        let id3 = builder.next_id();
        assert_eq!(id3.0, 2);
    }

    #[test]
    fn test_build_span_success() {
        let mut builder = AstBuilder::new();

        let span_sexp = SExp::List(List {
            elements: vec![
                SExp::Symbol(Symbol { value: "Span".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "lo".to_string(), pos: dummy_pos() }),
                SExp::Number(Number { value: "10".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "hi".to_string(), pos: dummy_pos() }),
                SExp::Number(Number { value: "20".to_string(), pos: dummy_pos() }),
            ],
            pos: dummy_pos(),
        });

        let result = builder.build_span(&span_sexp);
        assert!(result.is_ok());

        let span = result.unwrap();
        assert_eq!(span.lo, 10);
        assert_eq!(span.hi, 20);
    }

    #[test]
    fn test_build_span_empty_list() {
        let mut builder = AstBuilder::new();

        let span_sexp = SExp::List(List {
            elements: vec![],
            pos: dummy_pos(),
        });

        let result = builder.build_span(&span_sexp);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Span::DUMMY);
    }

    #[test]
    fn test_build_span_defaults() {
        let mut builder = AstBuilder::new();

        let span_sexp = SExp::List(List {
            elements: vec![
                SExp::Symbol(Symbol { value: "Span".to_string(), pos: dummy_pos() }),
            ],
            pos: dummy_pos(),
        });

        let result = builder.build_span(&span_sexp);
        assert!(result.is_ok());

        let span = result.unwrap();
        assert_eq!(span.lo, 0);
        assert_eq!(span.hi, 0);
    }

    #[test]
    fn test_build_span_wrong_type() {
        let mut builder = AstBuilder::new();

        let span_sexp = SExp::List(List {
            elements: vec![
                SExp::Symbol(Symbol { value: "NotSpan".to_string(), pos: dummy_pos() }),
            ],
            pos: dummy_pos(),
        });

        let result = builder.build_span(&span_sexp);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_mod_spans_with_inner_span() {
        let mut builder = AstBuilder::new();

        let mod_spans_sexp = SExp::List(List {
            elements: vec![
                SExp::Symbol(Symbol { value: "ModSpans".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "inner-span".to_string(), pos: dummy_pos() }),
                SExp::List(List {
                    elements: vec![
                        SExp::Symbol(Symbol { value: "Span".to_string(), pos: dummy_pos() }),
                        SExp::Keyword(Keyword { name: "lo".to_string(), pos: dummy_pos() }),
                        SExp::Number(Number { value: "5".to_string(), pos: dummy_pos() }),
                        SExp::Keyword(Keyword { name: "hi".to_string(), pos: dummy_pos() }),
                        SExp::Number(Number { value: "15".to_string(), pos: dummy_pos() }),
                    ],
                    pos: dummy_pos(),
                }),
            ],
            pos: dummy_pos(),
        });

        let result = builder.build_mod_spans(&mod_spans_sexp);
        assert!(result.is_ok());

        let mod_spans = result.unwrap();
        assert_eq!(mod_spans.inner_span.lo, 5);
        assert_eq!(mod_spans.inner_span.hi, 15);
    }

    #[test]
    fn test_build_mod_spans_default() {
        let mut builder = AstBuilder::new();

        let mod_spans_sexp = SExp::List(List {
            elements: vec![
                SExp::Symbol(Symbol { value: "ModSpans".to_string(), pos: dummy_pos() }),
            ],
            pos: dummy_pos(),
        });

        let result = builder.build_mod_spans(&mod_spans_sexp);
        assert!(result.is_ok());

        let mod_spans = result.unwrap();
        assert_eq!(mod_spans.inner_span, Span::DUMMY);
    }

    #[test]
    fn test_build_items_list_empty() {
        let mut builder = AstBuilder::new();

        let items_sexp = SExp::List(List {
            elements: vec![],
            pos: dummy_pos(),
        });

        let result = builder.build_items_list(&items_sexp);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_build_crate_empty_list_error() {
        let mut builder = AstBuilder::new();

        let crate_sexp = SExp::List(List {
            elements: vec![],
            pos: dummy_pos(),
        });

        let result = builder.build_crate(&crate_sexp);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_crate_wrong_type() {
        let mut builder = AstBuilder::new();

        let crate_sexp = SExp::List(List {
            elements: vec![
                SExp::Symbol(Symbol { value: "NotCrate".to_string(), pos: dummy_pos() }),
            ],
            pos: dummy_pos(),
        });

        let result = builder.build_crate(&crate_sexp);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_crate_missing_items() {
        let mut builder = AstBuilder::new();

        let crate_sexp = SExp::List(List {
            elements: vec![
                SExp::Symbol(Symbol { value: "Crate".to_string(), pos: dummy_pos() }),
            ],
            pos: dummy_pos(),
        });

        let result = builder.build_crate(&crate_sexp);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_crate_minimal() {
        let mut builder = AstBuilder::new();

        let crate_sexp = SExp::List(List {
            elements: vec![
                SExp::Symbol(Symbol { value: "Crate".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "items".to_string(), pos: dummy_pos() }),
                SExp::List(List { elements: vec![], pos: dummy_pos() }),
            ],
            pos: dummy_pos(),
        });

        let result = builder.build_crate(&crate_sexp);
        assert!(result.is_ok());

        let krate = result.unwrap();
        assert_eq!(krate.items.len(), 0);
        assert_eq!(krate.id.0, 0); // Auto-generated ID
        assert_eq!(krate.spans.inner_span, Span::DUMMY);
    }

    #[test]
    fn test_build_crate_with_id_and_spans() {
        let mut builder = AstBuilder::new();

        let crate_sexp = SExp::List(List {
            elements: vec![
                SExp::Symbol(Symbol { value: "Crate".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "items".to_string(), pos: dummy_pos() }),
                SExp::List(List { elements: vec![], pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "id".to_string(), pos: dummy_pos() }),
                SExp::Number(Number { value: "42".to_string(), pos: dummy_pos() }),
                SExp::Keyword(Keyword { name: "spans".to_string(), pos: dummy_pos() }),
                SExp::List(List {
                    elements: vec![
                        SExp::Symbol(Symbol { value: "ModSpans".to_string(), pos: dummy_pos() }),
                        SExp::Keyword(Keyword { name: "inner-span".to_string(), pos: dummy_pos() }),
                        SExp::List(List {
                            elements: vec![
                                SExp::Symbol(Symbol { value: "Span".to_string(), pos: dummy_pos() }),
                                SExp::Keyword(Keyword { name: "lo".to_string(), pos: dummy_pos() }),
                                SExp::Number(Number { value: "1".to_string(), pos: dummy_pos() }),
                                SExp::Keyword(Keyword { name: "hi".to_string(), pos: dummy_pos() }),
                                SExp::Number(Number { value: "100".to_string(), pos: dummy_pos() }),
                            ],
                            pos: dummy_pos(),
                        }),
                    ],
                    pos: dummy_pos(),
                }),
            ],
            pos: dummy_pos(),
        });

        let result = builder.build_crate(&crate_sexp);
        assert!(result.is_ok());

        let krate = result.unwrap();
        assert_eq!(krate.id.0, 42);
        assert_eq!(krate.spans.inner_span.lo, 1);
        assert_eq!(krate.spans.inner_span.hi, 100);
    }
}
