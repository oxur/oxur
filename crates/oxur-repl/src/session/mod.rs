//! Session management for REPL
//!
//! Manages session-specific state including temporary directories,
//! compilation artifacts, and cleanup.

mod dir;

pub use dir::{SessionDir, SessionDirError};
