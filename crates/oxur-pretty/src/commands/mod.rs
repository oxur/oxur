//! Command implementations for the oxur-fmt CLI tool.

pub mod check;
pub mod format;

// Re-export execute functions
pub use check::execute as check;
pub use format::execute as format;
