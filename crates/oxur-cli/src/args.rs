//! Command-line argument definitions for Oxur CLI tools.
//!
//! This module contains shared argument structures that can be reused
//! across different Oxur binaries (e.g., `oxur` and `cargo-oxur`).

use clap::Args;

/// REPL command arguments
///
/// These arguments control how the Oxur REPL operates, including
/// connection modes, server configuration, and display options.
#[derive(Args, Debug, Default, Clone)]
pub struct ReplArgs {
    /// Start the default built-in REPL server and connect to it
    /// with the built-in client. This is the default behavior.
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Connect to a running REPL server with the built-in client.
    /// If no address given, connects to 127.0.0.1:5099
    #[arg(short = 'c', long = "connect", value_name = "HOST:PORT")]
    pub connect: Option<Option<String>>,

    /// Disable ANSI colors in interactive or connect modes.
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Start a REPL server only (no client).
    /// Use a path for Unix socket, or HOST:PORT for TCP.
    #[arg(short = 's', long = "serve", value_name = "PATH|HOST:PORT")]
    pub serve: Option<String>,

    /// Acknowledge the port of this server to another nREPL server
    /// running on ACK-PORT. Used for tooling integration.
    #[arg(long = "ack", value_name = "ACK-PORT")]
    pub ack: Option<u16>,

    /// The transport module to use.
    /// Default: oxur_repl::transport::tcp
    #[arg(short = 't', long = "transport", value_name = "TRANSPORT")]
    pub transport: Option<String>,
}
