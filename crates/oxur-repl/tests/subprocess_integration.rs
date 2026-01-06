//! Integration tests for subprocess IPC protocol
//!
//! These tests verify the subprocess binary's IPC protocol implementation
//! by spawning actual subprocess instances and communicating over stdin/stdout.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Find the subprocess binary
fn find_subprocess_binary() -> PathBuf {
    // Try workspace target directory (from crate)
    let workspace_debug = PathBuf::from("../../target/debug/oxur-repl-subprocess");
    if workspace_debug.exists() {
        return workspace_debug;
    }

    // Try workspace target directory (from workspace root)
    let root_debug = PathBuf::from("target/debug/oxur-repl-subprocess");
    if root_debug.exists() {
        return root_debug;
    }

    // Try current directory target
    let current_debug = PathBuf::from("../../../target/debug/oxur-repl-subprocess");
    if current_debug.exists() {
        return current_debug;
    }

    // Fallback: assume it's in PATH or relative path
    PathBuf::from("target/debug/oxur-repl-subprocess")
}

/// Helper to spawn subprocess and get handles
fn spawn_subprocess(
) -> (std::process::Child, std::process::ChildStdin, BufReader<std::process::ChildStdout>) {
    let binary_path = find_subprocess_binary();

    let mut child = Command::new(&binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to spawn subprocess at {:?}: {}", binary_path, e));

    let stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");
    let reader = BufReader::new(stdout);

    (child, stdin, reader)
}

/// Send a command and read the response
fn send_and_receive(
    stdin: &mut std::process::ChildStdin,
    reader: &mut BufReader<std::process::ChildStdout>,
    command: &str,
) -> String {
    writeln!(stdin, "{}", command).expect("Failed to write command");
    stdin.flush().expect("Failed to flush");

    let mut response = String::new();
    reader.read_line(&mut response).expect("Failed to read response");

    response.trim().to_string()
}

#[test]
#[ignore] // Requires subprocess binary to be built
fn test_ping_pong() {
    let (mut child, mut stdin, mut reader) = spawn_subprocess();

    let response = send_and_receive(&mut stdin, &mut reader, "PING");
    assert_eq!(response, "PONG");

    drop(stdin);
    child.wait().ok();
}

#[test]
#[ignore] // Requires subprocess binary to be built
fn test_stats_command() {
    let (mut child, mut stdin, mut reader) = spawn_subprocess();

    let response = send_and_receive(&mut stdin, &mut reader, "STATS");
    assert!(response.starts_with("LIBRARIES_LOADED "));

    drop(stdin);
    child.wait().ok();
}

#[test]
#[ignore] // Requires subprocess binary to be built
fn test_load_nonexistent_library() {
    let (mut child, mut stdin, mut reader) = spawn_subprocess();

    let response = send_and_receive(&mut stdin, &mut reader, "LOAD test_key /nonexistent/lib.so");

    assert!(response.starts_with("LOAD_ERROR"), "Expected LOAD_ERROR, got: {}", response);

    drop(stdin);
    child.wait().ok();
}

#[test]
#[ignore] // Requires subprocess binary to be built
fn test_run_without_load() {
    let (mut child, mut stdin, mut reader) = spawn_subprocess();

    let response = send_and_receive(&mut stdin, &mut reader, "RUN test_key");

    assert!(
        response.starts_with("OXUR_RUNTIME_ERROR"),
        "Expected OXUR_RUNTIME_ERROR, got: {}",
        response
    );
    assert!(response.contains("not loaded"));

    drop(stdin);
    child.wait().ok();
}

#[test]
#[ignore] // Requires subprocess binary to be built
fn test_unknown_command() {
    let (mut child, mut stdin, mut reader) = spawn_subprocess();

    let response = send_and_receive(&mut stdin, &mut reader, "INVALID_COMMAND");

    assert!(response.starts_with("ERROR"), "Expected ERROR, got: {}", response);

    drop(stdin);
    child.wait().ok();
}

#[test]
#[ignore] // Requires subprocess binary to be built
fn test_empty_lines_ignored() {
    let (mut child, mut stdin, mut reader) = spawn_subprocess();

    // Send empty line
    writeln!(stdin).expect("Failed to write");
    stdin.flush().expect("Failed to flush");

    // Send PING to verify subprocess still responsive
    let response = send_and_receive(&mut stdin, &mut reader, "PING");
    assert_eq!(response, "PONG");

    drop(stdin);
    child.wait().ok();
}

#[test]
#[ignore] // Requires subprocess binary to be built
fn test_multiple_commands_sequence() {
    let (mut child, mut stdin, mut reader) = spawn_subprocess();

    // Test multiple commands in sequence
    let response1 = send_and_receive(&mut stdin, &mut reader, "PING");
    assert_eq!(response1, "PONG");

    let response2 = send_and_receive(&mut stdin, &mut reader, "STATS");
    assert!(response2.starts_with("LIBRARIES_LOADED "));

    let response3 = send_and_receive(&mut stdin, &mut reader, "PING");
    assert_eq!(response3, "PONG");

    drop(stdin);
    child.wait().ok();
}

#[test]
#[ignore] // Requires subprocess binary to be built
fn test_subprocess_exits_on_stdin_close() {
    let (mut child, stdin, _reader) = spawn_subprocess();

    // Close stdin
    drop(stdin);

    // Wait for subprocess to exit (with timeout)
    let wait_result = std::thread::spawn(move || child.wait()).join().expect("Thread panicked");

    assert!(wait_result.is_ok(), "Subprocess should exit cleanly");
}
