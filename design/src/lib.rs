//! Design documentation library for Oxur
//!
//! This library provides types and utilities for managing design documents
//! in the Oxur project.

pub mod doc;
pub mod git;
pub mod index;

pub use doc::{
    add_missing_headers, build_yaml_frontmatter, extract_number_from_filename,
    extract_title_from_content, DesignDoc, DocError, DocMetadata, DocState,
};
pub use index::DocumentIndex;

/// Re-export common error types
pub use anyhow::{Error, Result};
