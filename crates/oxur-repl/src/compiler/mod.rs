//! Compilation pipeline for Oxur REPL
//!
//! Manages the compilation of Oxur code to dynamic libraries with caching.
//!
//! Based on ODD-0026 Section 2.1 (Compilation Pipeline)

mod cached;
mod error_translator;

pub use cached::CachedCompiler;
pub use error_translator::{
    DiagnosticLevel, ErrorTranslator, RustcDiagnostic, TranslatedDiagnostic, TranslatedSpan,
    TranslationError,
};
