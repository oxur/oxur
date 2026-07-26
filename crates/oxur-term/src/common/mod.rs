//! Common utilities for Oxur CLI tools
//!
//! This module provides shared functionality for building consistent CLI
//! tools in the Oxur project.

pub mod io;
pub mod output;
pub mod progress;

// Re-exports for convenience
pub use progress::ProgressTracker;
