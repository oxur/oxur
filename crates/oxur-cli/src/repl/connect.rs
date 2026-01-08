//! Connect mode REPL client implementation
//!
//! Provides REPL client that connects to a remote server via TCP.

use crate::repl::help::HelpSystem;
use crate::repl::terminal::ReplTerminal;
use anyhow::{Context, Result};
use oxur_cli::config::ReplConfig;
use oxur_repl::protocol::{MessageId, Operation, OperationResult, ReplMode, Request, SessionId};
use oxur_repl::transport::{TcpTransport, Transport};
use rustyline::error::ReadlineError;
use std::sync::atomic::{AtomicU64, Ordering};

/// Run the connect mode REPL
///
/// Connects to an existing REPL server and provides terminal interface
/// for sending commands and receiving results.
pub async fn run(addr: &str, config: ReplConfig) -> Result<()> {
    // Connect to server
    let mut client = TcpTransport::connect(addr)
        .await
        .context(format!("Failed to connect to REPL server at {}", addr))?;

    // Generate unique session ID
    let session_id = SessionId::new(format!("connect-{}", std::process::id()));

    // Message ID counter
    let msg_id_counter = AtomicU64::new(1);

    // Create session on server
    let create_req = Request {
        id: MessageId::new(msg_id_counter.fetch_add(1, Ordering::SeqCst)),
        session_id: session_id.clone(),
        operation: Operation::CreateSession { mode: ReplMode::Lisp },
    };

    client.send_request(&create_req).await.context("Failed to send create session request")?;

    let _response = client.recv_response().await.context("Failed to receive create response")?;

    // Create terminal interface with configuration
    let mut terminal = ReplTerminal::with_config(config.terminal, config.history)
        .context("Failed to create terminal")?;

    // Print welcome banner (respects terminal configuration)
    terminal.print_banner();

    // Show connection info
    if terminal.config().color_enabled {
        println!("\x1b[36mConnected to {}\x1b[0m\n", addr);
    } else {
        println!("Connected to {}\n", addr);
    }

    // Main REPL loop
    loop {
        // Read input from user
        let line = match terminal.read_line_default() {
            Ok(Some(line)) => line,
            Ok(None) => {
                // Ctrl-C - just print newline and continue
                println!();
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D - exit
                break;
            }
            Err(e) => {
                terminal.print_error(&format!("Input error: {}", e));
                break;
            }
        };

        // Skip empty lines
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check for special commands
        if trimmed == "(quit)" || trimmed == "(q)" || trimmed == "(exit)" {
            break;
        }

        // Check for help commands
        if let Some(help_output) = parse_help_command(trimmed, terminal.config().color_enabled) {
            terminal.print_help(&help_output);
            continue;
        }

        // Create eval request
        let eval_req = Request {
            id: MessageId::new(msg_id_counter.fetch_add(1, Ordering::SeqCst)),
            session_id: session_id.clone(),
            operation: Operation::Eval { code: trimmed.to_string(), mode: ReplMode::Lisp },
        };

        // Send request to server
        if let Err(e) = client.send_request(&eval_req).await {
            terminal.print_error(&format!("Failed to send request: {}", e));
            continue;
        }

        // Receive response
        let response = match client.recv_response().await {
            Ok(r) => r,
            Err(e) => {
                terminal.print_error(&format!("Failed to receive response: {}", e));
                continue;
            }
        };

        // Display result
        match response.result {
            OperationResult::Success { value, stdout, stderr, .. } => {
                // Print stdout if any
                if let Some(out) = stdout {
                    if !out.is_empty() {
                        terminal.print_output(&out);
                    }
                }

                // Print return value if any
                if let Some(val) = value {
                    if !val.is_empty() {
                        terminal.print_result(&val);
                    }
                }

                // Print stderr if any
                if let Some(err) = stderr {
                    if !err.is_empty() {
                        eprintln!("{}", err);
                    }
                }
            }
            OperationResult::Error { error, stdout, stderr } => {
                // Print any stdout before the error
                if let Some(out) = stdout {
                    if !out.is_empty() {
                        terminal.print_output(&out);
                    }
                }

                // Print the error message
                terminal.print_error(&error.message);

                // Print stderr if any
                if let Some(err) = stderr {
                    if !err.is_empty() {
                        eprintln!("{}", err);
                    }
                }
            }
            OperationResult::Sessions { .. } | OperationResult::HistoryEntries { .. } => {
                // These don't produce output in interactive eval mode
            }
            _ => {
                // Handle any future OperationResult variants
            }
        }
    }

    // Save history before exit
    if let Err(e) = terminal.save_history() {
        eprintln!("Warning: Failed to save command history: {}", e);
    }

    terminal.print_goodbye();

    // Close session
    let close_req = Request {
        id: MessageId::new(msg_id_counter.fetch_add(1, Ordering::SeqCst)),
        session_id: session_id.clone(),
        operation: Operation::Close,
    };

    let _ = client.send_request(&close_req).await;
    let _ = client.recv_response().await;

    // Close transport
    let _ = client.close().await;

    Ok(())
}

/// Parse help commands and return formatted help output
///
/// Recognizes:
/// - `(help)` - Returns overview help
/// - `(help <topic>)` - Returns topic-specific help or error message
///
/// Returns `None` if input is not a help command.
fn parse_help_command(input: &str, color_enabled: bool) -> Option<String> {
    let help_system = HelpSystem::new(color_enabled);

    if input == "(help)" {
        return Some(help_system.show_overview());
    }

    // Parse (help <topic>)
    if input.starts_with("(help ") && input.ends_with(')') {
        let topic = &input[6..input.len() - 1].trim();
        return help_system.show_topic(topic).or_else(|| {
            Some(format!("Unknown help topic: {}. Try (help) for available topics.", topic))
        });
    }

    None
}
