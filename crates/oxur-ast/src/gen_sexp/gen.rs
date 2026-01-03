use crate::ast::*;
use crate::error::Result;
use crate::gen_sexp::helpers::*;
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
            kwarg("is-placeholder", sym(if crate_node.is_placeholder { "true" } else { "false" })),
        ]);

        Ok(typed_node("Crate", fields))
    }

    pub(crate) fn generate_attr_vec(&self, attrs: &AttrVec) -> Result<SExp> {
        let attr_sexps: Result<Vec<SExp>> = attrs
            .iter()
            .map(|attr| self.generate_attribute(attr))
            .collect();
        Ok(list(attr_sexps?))
    }

    fn generate_attribute(&self, attr: &Attribute) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("kind", self.generate_attr_kind(&attr.kind)?),
            kwarg("id", num(attr.id)),
            kwarg("style", self.generate_attr_style(attr.style)),
            kwarg("span", self.generate_span(attr.span)),
        ]);

        Ok(typed_node("Attribute", fields))
    }

    fn generate_attr_style(&self, style: AttrStyle) -> SExp {
        match style {
            AttrStyle::Outer => sym("Outer"),
            AttrStyle::Inner => sym("Inner"),
        }
    }

    fn generate_attr_kind(&self, kind: &AttrKind) -> Result<SExp> {
        match kind {
            AttrKind::Normal(normal) => {
                Ok(list(vec![sym("Normal"), self.generate_attr_item(&normal.item)?]))
            }
            AttrKind::DocComment(comment_kind, text) => Ok(list(vec![
                sym("DocComment"),
                self.generate_comment_kind(*comment_kind),
                string(text),
            ])),
        }
    }

    fn generate_attr_item(&self, item: &AttrItem) -> Result<SExp> {
        let fields = kwargs(vec![
            kwarg("path", self.generate_path(&item.path)),
            kwarg("args", self.generate_mac_args_for_attr(&item.args)?),
        ]);

        Ok(typed_node("AttrItem", fields))
    }

    fn generate_comment_kind(&self, kind: CommentKind) -> SExp {
        match kind {
            CommentKind::Line => sym("Line"),
            CommentKind::Block => sym("Block"),
        }
    }

    // Helper to generate MacArgs - delegates to expr module's implementation
    fn generate_mac_args_for_attr(&self, args: &MacArgs) -> Result<SExp> {
        // Import the function from expr module via self
        self.generate_mac_args(args)
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
        let fields = kwargs(vec![kwarg("lo", num(span.lo)), kwarg("hi", num(span.hi))]);

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
