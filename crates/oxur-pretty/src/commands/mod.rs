//! Command implementations for the oxur-fmt CLI tool.

pub mod check;
pub mod format;

pub use check::execute as check;
pub use format::execute as format;
