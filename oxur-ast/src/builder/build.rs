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
