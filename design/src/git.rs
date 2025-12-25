//! Git integration for design documents
//!
//! Provides functions for extracting metadata from git history
//! and performing git operations while preserving history.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Extract the original author from git history
/// Falls back to git config user.name, then "Unknown Author" if git fails
pub fn get_author(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();

    // Try to get author from git log (first commit)
    let output = Command::new("git")
        .args(["log", "--format=%an", "--reverse", "--"])
        .arg(path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(author) = stdout.lines().next() {
                let author = author.trim();
                if !author.is_empty() {
                    return author.to_string();
                }
            }
        }
    }

    // Fallback to git config user.name
    let output = Command::new("git").args(["config", "user.name"]).output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let name = stdout.trim();
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }

    "Unknown Author".to_string()
}

/// Extract creation date from git history (first commit)
/// Falls back to today's date if git fails
pub fn get_created_date(path: impl AsRef<Path>) -> NaiveDate {
    let path = path.as_ref();

    let output = Command::new("git")
        .args(["log", "--format=%ai", "--reverse", "--"])
        .arg(path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().next() {
                // Extract just the date portion (YYYY-MM-DD)
                if let Some(date_str) = line.split_whitespace().next() {
                    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        return date;
                    }
                }
            }
        }
    }

    chrono::Local::now().naive_local().date()
}

/// Extract last modified date from git history
/// Falls back to today's date if git fails
pub fn get_updated_date(path: impl AsRef<Path>) -> NaiveDate {
    let path = path.as_ref();

    let output = Command::new("git")
        .args(["log", "--format=%ai", "-1", "--"])
        .arg(path)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Extract just the date portion (YYYY-MM-DD)
            if let Some(date_str) = stdout.split_whitespace().next() {
                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    return date;
                }
            }
        }
    }

    chrono::Local::now().naive_local().date()
}

/// Move a file using git mv to preserve history
/// Creates destination directory if needed
pub fn git_mv(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    // Ensure destination directory exists
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).context("Failed to create destination directory")?;
    }

    // Execute git mv
    let output = Command::new("git")
        .arg("mv")
        .arg(src)
        .arg(dst)
        .output()
        .context("Failed to execute git mv")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git mv failed: {}", stderr.trim());
    }

    Ok(())
}

/// Stage a file with git add
pub fn git_add(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();

    let output = Command::new("git")
        .arg("add")
        .arg(path)
        .output()
        .context("Failed to execute git add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git add failed: {}", stderr.trim());
    }

    Ok(())
}

/// Check if a path is in a git repository
pub fn is_git_repo(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    let dir = if path.is_dir() { path } else { path.parent().unwrap_or(path) };

    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(dir)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if a file is tracked by git
pub fn is_tracked(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    Command::new("git")
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(path)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the git repository root directory
pub fn get_repo_root() -> Option<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Some(std::path::PathBuf::from(stdout.trim()))
    } else {
        None
    }
}
