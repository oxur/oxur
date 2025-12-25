//! Design documentation library for Oxur
//!
//! This library provides types and utilities for managing design documents
//! in the Oxur project.

pub mod doc;
pub mod index;

pub use doc::{DesignDoc, DocMetadata, DocState};
pub use index::DocumentIndex;

/// Re-export common error types
pub use anyhow::{Error, Result};
