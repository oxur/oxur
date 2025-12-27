//! Design documentation library for Oxur
//!
//! This library provides types and utilities for managing design documents
//! in the Oxur project.

pub mod config;
pub mod doc;
pub mod errors;
pub mod extract;
pub mod filename;
pub mod git;
pub mod index;
pub mod index_sync;
pub mod normalize;
pub mod prompt;
pub mod state;
pub mod theme;

pub use doc::{
    add_missing_headers, add_number_prefix, build_yaml_frontmatter, ensure_valid_headers,
    extract_number_from_filename, extract_title_from_content, has_frontmatter, has_number_prefix,
    has_placeholder_values, is_in_project_dir, is_in_state_dir, move_to_project, move_to_state_dir,
    state_from_directory, sync_state_with_directory, DesignDoc, DocError, DocMetadata, DocState,
};
pub use index::DocumentIndex;

/// Re-export common error types
pub use anyhow::{Error, Result};
