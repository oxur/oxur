//! System metadata captured at startup
//!
//! Provides information about the runtime environment including
//! versions, system info, and process details.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;
use sysinfo::System;

/// System metadata captured at startup
///
/// This struct captures various system and runtime information when
/// the REPL starts. It can be queried via the `(info)` command.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemMetadata {
    /// Oxur version (from Cargo.toml)
    pub oxur_version: String,
    /// Rust compiler version (detected at runtime)
    pub rust_version: String,
    /// Cargo version (detected at runtime)
    pub cargo_version: String,
    /// Operating system name (e.g., "macOS", "Ubuntu")
    pub os_name: String,
    /// OS version (e.g., "14.5")
    pub os_version: String,
    /// System architecture (e.g., "aarch64", "x86_64")
    pub arch: String,
    /// Machine hostname
    pub hostname: String,
    /// Process ID
    pub pid: u32,
    /// Current working directory at startup
    pub cwd: PathBuf,
    /// Startup timestamp
    pub started_at: SystemTime,
}

impl SystemMetadata {
    /// Capture system metadata at startup
    ///
    /// This should be called once when the REPL starts to capture
    /// a snapshot of the system state.
    pub fn capture() -> Self {
        Self {
            oxur_version: env!("CARGO_PKG_VERSION").to_string(),
            rust_version: detect_rust_version(),
            cargo_version: detect_cargo_version(),
            os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
            arch: System::cpu_arch().unwrap_or_else(|| "unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
            pid: std::process::id(),
            cwd: std::env::current_dir().unwrap_or_default(),
            started_at: SystemTime::now(),
        }
    }

    /// Get uptime since startup in seconds
    pub fn uptime_seconds(&self) -> f64 {
        self.started_at.elapsed().map(|d| d.as_secs_f64()).unwrap_or(0.0)
    }
}

/// Detect the installed Rust compiler version
fn detect_rust_version() -> String {
    std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Detect the installed Cargo version
fn detect_cargo_version() -> String {
    std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_metadata() {
        let metadata = SystemMetadata::capture();

        // Oxur version should be set
        assert!(!metadata.oxur_version.is_empty());

        // Rust version should be detected (or "unknown")
        assert!(!metadata.rust_version.is_empty());

        // Cargo version should be detected (or "unknown")
        assert!(!metadata.cargo_version.is_empty());

        // OS info should be present
        assert!(!metadata.os_name.is_empty());
        assert!(!metadata.arch.is_empty());

        // PID should be non-zero
        assert!(metadata.pid > 0);

        // CWD should be set (even if empty path on error)
        // Just check it exists
        let _ = metadata.cwd;
    }

    #[test]
    fn test_uptime_increases() {
        let metadata = SystemMetadata::capture();

        // Initial uptime should be very small
        let uptime1 = metadata.uptime_seconds();
        assert!(uptime1 < 1.0);

        // Wait a tiny bit
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Uptime should have increased
        let uptime2 = metadata.uptime_seconds();
        assert!(uptime2 > uptime1);
    }

    #[test]
    fn test_rust_version_detection() {
        let version = detect_rust_version();
        // Should either start with "rustc" or be "unknown"
        assert!(version.starts_with("rustc") || version == "unknown");
    }

    #[test]
    fn test_cargo_version_detection() {
        let version = detect_cargo_version();
        // Should either start with "cargo" or be "unknown"
        assert!(version.starts_with("cargo") || version == "unknown");
    }

    #[test]
    fn test_metadata_serialization() {
        let metadata = SystemMetadata::capture();

        // Should serialize to JSON without errors
        let json = serde_json::to_string(&metadata).expect("Failed to serialize");
        assert!(!json.is_empty());

        // Should deserialize back
        let deserialized: SystemMetadata =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(metadata.oxur_version, deserialized.oxur_version);
        assert_eq!(metadata.pid, deserialized.pid);
    }
}
