use crate::ast::*;
use crate::error::Result;
use crate::generator::helpers::*;
use crate::sexp::SExp;

pub struct Generator;

impl Generator {
    pub fn new() -> Self {
        Self
    }

    /// Generate S-expression from Crate
    pub fn generate_crate(&self, crate_node: &Crate) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("attrs", self.generate_attr_vec(&crate_node.attrs)?),
            kwarg("items", self.generate_items(&crate_node.items)?),
            kwarg("spans", self.generate_mod_spans(&crate_node.spans)?),
            kwarg("id", self.generate_node_id(crate_node.id)),
            kwarg(
                "is-placeholder",
                sym(if crate_node.is_placeholder {
                    "true"
                } else {
                    "false"
                }),
            ),
        ]);

        Ok(typed_node("Crate", fields))
    }

    pub(crate) fn generate_attr_vec(&self, _attrs: &AttrVec) -> Result<SExp> {
        // Phase 2: Just empty list for now
        Ok(empty_list())
    }

    fn generate_items(&self, items: &[Item]) -> Result<SExp> {
        let item_sexps: Result<Vec<SExp>> =
            items.iter().map(|item| self.generate_item(item)).collect();
        Ok(list(item_sexps?))
    }

    fn generate_mod_spans(&self, spans: &ModSpans) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("inner-span", self.generate_span(spans.inner_span)),
            kwarg("inject-use-span", self.generate_span(spans.inject_use_span)),
        ]);

        Ok(typed_node("ModSpans", fields))
    }

    pub fn generate_span(&self, span: Span) -> SExp {
        let fields = kwargs(vec![
            kwarg("lo", num(span.lo)),
            kwarg("hi", num(span.hi)),
        ]);

        // Only include ctxt if non-zero
        let fields = if span.ctxt != 0 {
            let mut f = fields;
            f.extend(kwarg("ctxt", num(span.ctxt)));
            f
        } else {
            fields
        };

        typed_node("Span", fields)
    }

    pub(crate) fn generate_node_id(&self, id: NodeId) -> SExp {
        num(id.0)
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}
