//! Source mapping for Oxur language
//!
//! This crate provides source mapping capabilities for tracking code
//! transformations through the Oxur compilation pipeline. It enables
//! rustc compiler errors to be translated back to the original Oxur
//! source code positions.
//!
//! # Architecture
//!
//! The source map tracks three transformation stages:
//!
//! 1. **Surface Forms** → **Core Forms** (macro expansion)
//! 2. **Core Forms** → **Rust AST** (lowering)
//! 3. **Rust errors** → **Original Source** (backward lookup)
//!
//! # Example
//!
//! ```
//! use oxur_smap::{SourceMap, new_node_id, SourcePos, Span};
//!
//! let mut map = SourceMap::new();
//!
//! // Parser creates surface node with span
//! let surface = new_node_id();
//! let span = Span::repl(1, 1, 1, 10);
//! // Convert span to SourcePos for recording
//! let pos: SourcePos = (&span).into();
//! map.record_surface_node(surface, pos);
//!
//! // Expander creates core node
//! let core = new_node_id();
//! map.record_expansion(surface, core);
//!
//! // Lowering creates rust node
//! let rust = new_node_id();
//! map.record_lowering(core, rust);
//!
//! // Error translator looks up original position
//! let original = map.lookup(&rust).unwrap();
//! assert_eq!(original.line, 1);
//! assert_eq!(original.column, 1);
//! ```

mod node_id;
mod source_map;
mod source_pos;
mod span;

// Re-export public API
pub use node_id::{new_node_id, NodeId, NodeIdGenerator};
pub use source_map::{LookupStats, SourceMap, SourceMapStats};
pub use source_pos::SourcePos;
pub use span::Span;
