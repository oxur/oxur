//! Server Layer for REPL
//!
//! Provides session management, message handling, and server infrastructure.
//! Based on ODD-0018: Oxur Remote REPL Protocol Design.

mod handler;
mod session;

// Re-export public types
pub use handler::MessageHandler;
pub use session::{SessionError, SessionInfo, SessionManager};
