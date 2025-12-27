/// Span represents a region in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub lo: u32,   // Start byte position
    pub hi: u32,   // End byte position
    pub ctxt: u32, // Syntax context for hygiene
}

impl Span {
    pub const DUMMY: Span = Span { lo: 0, hi: 0, ctxt: 0 };

    pub fn new(lo: u32, hi: u32) -> Self {
        Self { lo, hi, ctxt: 0 }
    }

    pub fn with_ctxt(lo: u32, hi: u32, ctxt: u32) -> Self {
        Self { lo, hi, ctxt }
    }
}

/// Module spans
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModSpans {
    pub inner_span: Span,
    pub inject_use_span: Span,
}

impl ModSpans {
    pub fn new(inner_span: Span) -> Self {
        Self { inner_span, inject_use_span: Span::DUMMY }
    }
}

/// Delimiter span (for macro arguments)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelSpan {
    pub open: Span,
    pub close: Span,
}

impl DelSpan {
    pub fn new(open: Span, close: Span) -> Self {
        Self { open, close }
    }
}
