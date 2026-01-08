//! Interactive REPL mode implementation
//!
//! Provides the default REPL experience with in-memory client/server.

use crate::repl::terminal::ReplTerminal;
use anyhow::{Context, Result};
use oxur_cli::config::ReplConfig;
use oxur_repl::protocol::{MessageId, Operation, OperationResult, ReplMode, Request, SessionId};
use oxur_repl::server::{MessageHandler, SessionManager};
use oxur_repl::transport::{inprocess_channel, Transport};
use rustyline::error::ReadlineError;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Run the interactive REPL mode
///
/// Creates an in-process server and client connected via channels,
/// providing the fastest possible REPL experience with:
/// - Zero serialization overhead
/// - Line editing via rustyline
/// - Command history persistence
/// - Ctrl-C interrupt handling
/// - Ctrl-D exit handling
pub async fn run(config: ReplConfig) -> Result<()> {
    // Create in-process transport pair
    let (mut client, mut server_transport) = inprocess_channel();

    // Create session manager and message handler
    let session_manager = Arc::new(SessionManager::new());
    let handler = MessageHandler::new((*session_manager).clone());

    // Generate unique session ID
    let session_id = SessionId::new(format!("interactive-{}", std::process::id()));

    // Message ID counter
    let msg_id_counter = AtomicU64::new(1);

    // Create session
    let create_req = Request {
        id: MessageId::new(msg_id_counter.fetch_add(1, Ordering::SeqCst)),
        session_id: session_id.clone(),
        operation: Operation::CreateSession { mode: ReplMode::Lisp },
    };

    client.send_request(&create_req).await.context("Failed to send create session request")?;

    // Process create request on server side
    let create_resp = handler.handle(create_req).await;
    server_transport.send_response(&create_resp).await.context("Failed to send create response")?;

    let _response = client.recv_response().await.context("Failed to receive create response")?;

    // Create terminal interface with configuration
    let mut terminal = ReplTerminal::with_config(config.terminal, config.history)
        .context("Failed to create terminal")?;

    // Print welcome banner
    terminal.print_banner();

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
        if trimmed == "(quit)" || trimmed == "(exit)" {
            break;
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

        // Process request on server side
        let response = handler.handle(eval_req).await;

        // Send response back to client
        if let Err(e) = server_transport.send_response(&response).await {
            terminal.print_error(&format!("Failed to send response: {}", e));
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
    let _ = handler.handle(close_req).await;

    Ok(())
}
