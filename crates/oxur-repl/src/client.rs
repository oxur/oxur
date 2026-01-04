//! REPL Client (Stub)
//!
//! This is a placeholder stub for the old ReplClient API.
//! The new architecture uses TCP-based client/server (see server/mod.rs and transport/mod.rs).
//!
//! This stub exists only to satisfy external dependencies until they are updated
//! to use the new architecture.

use crate::Result;

/// Stub REPL client for backwards compatibility.
///
/// **Note:** This is a placeholder. Use the TCP-based client architecture instead.
/// See `crates/oxur-repl/src/transport/tcp.rs` for the actual client implementation.
#[derive(Debug, Default)]
pub struct ReplClient {
    _placeholder: (),
}

impl ReplClient {
    /// Creates a new stub REPL client.
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    /// Runs the REPL (stub - not implemented).
    ///
    /// # Errors
    ///
    /// Always returns an error indicating this is a stub.
    pub fn run(&mut self) -> Result<()> {
        Err(crate::Error::Protocol(
            "ReplClient is a stub. Use the TCP-based server/client architecture instead."
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let _client = ReplClient::new();
    }

    #[test]
    fn test_run_returns_error() {
        let mut client = ReplClient::new();
        assert!(client.run().is_err());
    }
}
