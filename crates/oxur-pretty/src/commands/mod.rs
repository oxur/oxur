//! Command implementations for the oxurfmt CLI tool.

pub mod check;
pub mod format;

// Re-export execute functions
pub use check::execute as check;
pub use format::execute as format;
