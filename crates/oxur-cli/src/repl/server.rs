//! Server mode REPL implementation
//!
//! Starts a REPL server that listens for client connections.

use anyhow::{Context, Result};
use oxur_repl::server::ReplServer;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::signal;

/// Run the REPL server mode
///
/// Starts a REPL server listening on the specified address.
/// Optionally sends an ACK to another port for tooling integration.
pub async fn run(addr: &str, ack_port: Option<u16>) -> Result<()> {
    // Create server
    let mut server = ReplServer::new(addr).with_shutdown_timeout(Duration::from_secs(30));

    // Get shutdown handle before starting
    let shutdown_handle = server.shutdown_handle();

    // Send ACK if requested (for tooling integration like nREPL)
    if let Some(port) = ack_port {
        send_ack(addr, port).await?;
    }

    // Spawn shutdown signal handler
    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        eprintln!("\n[INFO] Received shutdown signal, stopping server...");
        shutdown_handle.shutdown();
    });

    // Start server (blocks until shutdown)
    server.start().await.context("Server error")?;

    Ok(())
}

/// Wait for Ctrl-C or other shutdown signal
async fn wait_for_shutdown_signal() {
    // Wait for Ctrl-C
    let _ = signal::ctrl_c().await;
}

/// Send ACK to another nREPL server
///
/// This is used for tooling integration where another tool needs to know
/// that this server has started and what port it's listening on.
async fn send_ack(our_addr: &str, ack_port: u16) -> Result<()> {
    let ack_addr = format!("127.0.0.1:{}", ack_port);

    eprintln!("[INFO] Sending ACK to {}", ack_addr);

    match TcpStream::connect(&ack_addr).await {
        Ok(mut stream) => {
            // Send simple ACK message with our address
            let ack_msg = format!("{{:port {}}}\n", extract_port(our_addr));
            stream.write_all(ack_msg.as_bytes()).await.context("Failed to send ACK")?;
            stream.shutdown().await.context("Failed to close ACK connection")?;
            eprintln!("[INFO] ACK sent successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("[WARN] Failed to send ACK to {}: {}", ack_addr, e);
            // Don't fail startup just because ACK failed
            Ok(())
        }
    }
}

/// Extract port number from address string
fn extract_port(addr: &str) -> &str {
    addr.rsplit(':').next().unwrap_or("0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_port() {
        assert_eq!(extract_port("127.0.0.1:5099"), "5099");
        assert_eq!(extract_port("localhost:9000"), "9000");
        assert_eq!(extract_port("[::1]:8080"), "8080");
    }
}
