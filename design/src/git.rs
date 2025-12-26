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
    let output =
        Command::new("git").args(["log", "--format=%an", "--reverse", "--"]).arg(path).output();

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

    let output =
        Command::new("git").args(["log", "--format=%ai", "--reverse", "--"]).arg(path).output();

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

    let output = Command::new("git").args(["log", "--format=%ai", "-1", "--"]).arg(path).output();

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

    let output =
        Command::new("git").arg("add").arg(path).output().context("Failed to execute git add")?;

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
    let output = Command::new("git").args(["rev-parse", "--show-toplevel"]).output().ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Some(std::path::PathBuf::from(stdout.trim()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::process::Command;
    use tempfile::TempDir;

    /// Helper to run code in a directory and restore the original directory afterward
    fn in_dir<F, R>(dir: &std::path::Path, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(dir).unwrap();
        let result = f();

        // Try to restore original directory, but don't panic if it no longer exists
        if let Some(orig) = original_dir {
            let _ = std::env::set_current_dir(orig);
        }

        result
    }

    /// Helper to create a git repo for testing
    fn create_test_git_repo() -> TempDir {
        let temp = TempDir::new().unwrap();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(temp.path())
            .output()
            .expect("Failed to init git");

        // Configure user
        Command::new("git")
            .args(["config", "user.name", "Test Author"])
            .current_dir(temp.path())
            .output()
            .expect("Failed to config user.name");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(temp.path())
            .output()
            .expect("Failed to config user.email");

        temp
    }

    /// Helper to create and commit a file in a git repo
    fn create_and_commit_file(repo: &TempDir, filename: &str, content: &str, author: &str) {
        let file_path = repo.path().join(filename);
        fs::write(&file_path, content).unwrap();

        Command::new("git")
            .args(["add", filename])
            .current_dir(repo.path())
            .output()
            .expect("Failed to add file");

        Command::new("git")
            .args(["commit", "-m", "Test commit", &format!("--author={} <test@example.com>", author)])
            .current_dir(repo.path())
            .output()
            .expect("Failed to commit");
    }

    mod get_author {
        use super::*;

        #[test]
        #[serial]
        #[serial]
        fn test_gets_author_from_git_history() {
            let repo = create_test_git_repo();
            create_and_commit_file(&repo, "test.md", "content", "Original Author");

            let author = in_dir(repo.path(), || {
                get_author("test.md")
            });

            assert_eq!(author, "Original Author");
        }

        #[test]
        #[serial]
        fn test_gets_first_author_for_multiple_commits() {
            let repo = create_test_git_repo();

            // First commit by Original Author
            create_and_commit_file(&repo, "test.md", "v1", "Original Author");

            // Second commit by different author
            fs::write(repo.path().join("test.md"), "v2").unwrap();
            Command::new("git")
                .args(["add", "test.md"])
                .current_dir(repo.path())
                .output()
                .unwrap();
            Command::new("git")
                .args([
                    "commit",
                    "-m",
                    "Second commit",
                    "--author=Second Author <test@example.com>",
                ])
                .current_dir(repo.path())
                .output()
                .unwrap();

            let author = in_dir(repo.path(), || {
                get_author("test.md")
            });

            // Should return first author, not second
            assert_eq!(author, "Original Author");
        }

        #[test]
        #[serial]
        fn test_fallback_to_config_for_untracked_file() {
            let repo = create_test_git_repo();

            // Create file but don't commit
            fs::write(repo.path().join("untracked.md"), "content").unwrap();

            let author = in_dir(repo.path(), || {
                get_author("untracked.md")
            });

            // Should fallback to git config user.name
            assert_eq!(author, "Test Author");
        }

        #[test]
        #[serial]
        fn test_fallback_outside_repo() {
            let temp = TempDir::new().unwrap();
            fs::write(temp.path().join("file.md"), "content").unwrap();

            let author = in_dir(temp.path(), || {
                get_author("file.md")
            });

            // Should fallback to git config user.name when not in git repo
            // (will be global config value, or "Unknown Author" if not set)
            assert!(!author.is_empty());
        }
    }

    mod get_created_date {
        use super::*;

        #[test]
        #[serial]
        fn test_gets_date_from_first_commit() {
            let repo = create_test_git_repo();
            create_and_commit_file(&repo, "test.md", "v1", "Author");

            let created = in_dir(repo.path(), || get_created_date("test.md"));

            // Should be today (since we just created it)
            let today = chrono::Local::now().naive_local().date();
            assert_eq!(created, today);
        }

        #[test]
        #[serial]
        fn test_gets_first_commit_date_not_last() {
            let repo = create_test_git_repo();

            // First commit
            create_and_commit_file(&repo, "test.md", "v1", "Author");
            let first_date = in_dir(repo.path(), || get_created_date("test.md"));

            // Sleep briefly to ensure different timestamp
            std::thread::sleep(std::time::Duration::from_millis(1100));

            // Second commit
            fs::write(repo.path().join("test.md"), "v2").unwrap();
            Command::new("git")
                .args(["add", "test.md"])
                .current_dir(repo.path())
                .output()
                .unwrap();
            Command::new("git")
                .args(["commit", "-m", "Update"])
                .current_dir(repo.path())
                .output()
                .unwrap();

            // Should still return first commit date
            let created = in_dir(repo.path(), || get_created_date("test.md"));
            assert_eq!(created, first_date);
        }

        #[test]
        #[serial]
        fn test_fallback_to_today_for_untracked() {
            let repo = create_test_git_repo();
            fs::write(repo.path().join("untracked.md"), "content").unwrap();

            let created = in_dir(repo.path(), || get_created_date("untracked.md"));
            let today = chrono::Local::now().naive_local().date();

            assert_eq!(created, today);
        }
    }

    mod get_updated_date {
        use super::*;

        #[test]
        #[serial]
        fn test_gets_date_from_last_commit() {
            let repo = create_test_git_repo();
            create_and_commit_file(&repo, "test.md", "v1", "Author");

            let updated = in_dir(repo.path(), || get_updated_date("test.md"));

            // Should be today
            let today = chrono::Local::now().naive_local().date();
            assert_eq!(updated, today);
        }

        #[test]
        #[serial]
        fn test_gets_last_commit_date_not_first() {
            let repo = create_test_git_repo();

            // First commit
            create_and_commit_file(&repo, "test.md", "v1", "Author");

            // Sleep briefly
            std::thread::sleep(std::time::Duration::from_millis(1100));

            // Second commit
            fs::write(repo.path().join("test.md"), "v2").unwrap();
            Command::new("git")
                .args(["add", "test.md"])
                .current_dir(repo.path())
                .output()
                .unwrap();
            Command::new("git")
                .args(["commit", "-m", "Update"])
                .current_dir(repo.path())
                .output()
                .unwrap();

            // Should return today (last commit)
            let updated = in_dir(repo.path(), || get_updated_date("test.md"));
            let today = chrono::Local::now().naive_local().date();
            assert_eq!(updated, today);
        }

        #[test]
        #[serial]
        fn test_fallback_to_today_for_untracked() {
            let repo = create_test_git_repo();
            fs::write(repo.path().join("untracked.md"), "content").unwrap();

            let updated = in_dir(repo.path(), || get_updated_date("untracked.md"));
            let today = chrono::Local::now().naive_local().date();

            assert_eq!(updated, today);
        }
    }

    mod git_mv {
        use super::*;

        #[test]
        #[serial]
        fn test_moves_tracked_file() {
            let repo = create_test_git_repo();
            create_and_commit_file(&repo, "src.md", "content", "Author");

            let result = in_dir(repo.path(), || git_mv("src.md", "dest.md"));
            assert!(result.is_ok());
            assert!(!repo.path().join("src.md").exists());
            assert!(repo.path().join("dest.md").exists());
        }

        #[test]
        #[serial]
        fn test_creates_destination_directory() {
            let repo = create_test_git_repo();
            create_and_commit_file(&repo, "src.md", "content", "Author");

            let result = in_dir(repo.path(), || git_mv("src.md", "subdir/nested/dest.md"));
            assert!(result.is_ok());
            assert!(!repo.path().join("src.md").exists());
            assert!(repo.path().join("subdir/nested/dest.md").exists());
        }

        #[test]
        #[serial]
        fn test_fails_for_untracked_file() {
            let repo = create_test_git_repo();
            fs::write(repo.path().join("untracked.md"), "content").unwrap();

            let result = in_dir(repo.path(), || git_mv("untracked.md", "dest.md"));
            assert!(result.is_err());
        }

        #[test]
        #[serial]
        fn test_fails_for_nonexistent_file() {
            let repo = create_test_git_repo();

            let result = in_dir(repo.path(), || git_mv("nonexistent.md", "dest.md"));
            assert!(result.is_err());
        }
    }

    mod git_add {
        use super::*;

        #[test]
        #[serial]
        fn test_stages_untracked_file() {
            let repo = create_test_git_repo();
            fs::write(repo.path().join("new.md"), "content").unwrap();

            let result = in_dir(repo.path(), || git_add("new.md"));
            assert!(result.is_ok());

            // Verify it's staged
            let status = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(repo.path())
                .output()
                .unwrap();
            let output = String::from_utf8_lossy(&status.stdout);
            assert!(output.contains("A  new.md"));
        }

        #[test]
        #[serial]
        fn test_stages_modified_file() {
            let repo = create_test_git_repo();
            create_and_commit_file(&repo, "test.md", "v1", "Author");

            // Modify the file
            fs::write(repo.path().join("test.md"), "v2").unwrap();

            let result = in_dir(repo.path(), || git_add("test.md"));
            assert!(result.is_ok());

            // Verify it's staged
            let status = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(repo.path())
                .output()
                .unwrap();
            let output = String::from_utf8_lossy(&status.stdout);
            assert!(output.contains("M  test.md"));
        }

        #[test]
        #[serial]
        fn test_fails_for_nonexistent_file() {
            let repo = create_test_git_repo();

            let result = in_dir(repo.path(), || git_add("nonexistent.md"));
            assert!(result.is_err());
        }
    }

    mod is_git_repo {
        use super::*;

        #[test]
        #[serial]
        fn test_returns_true_in_git_repo() {
            let repo = create_test_git_repo();
            assert!(is_git_repo(repo.path()));
        }

        #[test]
        #[serial]
        fn test_returns_true_for_file_in_repo() {
            let repo = create_test_git_repo();
            let file_path = repo.path().join("test.md");
            fs::write(&file_path, "content").unwrap();

            assert!(is_git_repo(&file_path));
        }

        #[test]
        #[serial]
        fn test_returns_true_in_subdirectory() {
            let repo = create_test_git_repo();
            let subdir = repo.path().join("subdir");
            fs::create_dir(&subdir).unwrap();

            assert!(is_git_repo(&subdir));
        }

        #[test]
        #[serial]
        fn test_returns_false_outside_repo() {
            let temp = TempDir::new().unwrap();
            assert!(!is_git_repo(temp.path()));
        }
    }

    mod is_tracked {
        use super::*;

        #[test]
        #[serial]
        fn test_returns_true_for_tracked_file() {
            let repo = create_test_git_repo();
            create_and_commit_file(&repo, "tracked.md", "content", "Author");

            let tracked = in_dir(repo.path(), || is_tracked("tracked.md"));
            assert!(tracked);
        }

        #[test]
        #[serial]
        fn test_returns_false_for_untracked_file() {
            let repo = create_test_git_repo();
            fs::write(repo.path().join("untracked.md"), "content").unwrap();

            let tracked = in_dir(repo.path(), || is_tracked("untracked.md"));
            assert!(!tracked);
        }

        #[test]
        #[serial]
        fn test_returns_false_for_nonexistent_file() {
            let repo = create_test_git_repo();

            let tracked = in_dir(repo.path(), || is_tracked("nonexistent.md"));
            assert!(!tracked);
        }

        #[test]
        #[serial]
        fn test_returns_false_outside_repo() {
            let temp = TempDir::new().unwrap();
            fs::write(temp.path().join("file.md"), "content").unwrap();

            let tracked = in_dir(temp.path(), || is_tracked("file.md"));
            assert!(!tracked);
        }
    }

    mod get_repo_root {
        use super::*;

        #[test]
        #[serial]
        fn test_returns_root_in_repo() {
            let repo = create_test_git_repo();

            // Change to repo directory
            std::env::set_current_dir(repo.path()).unwrap();

            let root = get_repo_root();
            assert!(root.is_some());

            let root = root.unwrap();
            assert_eq!(root, repo.path().canonicalize().unwrap());
        }

        #[test]
        #[serial]
        fn test_returns_root_from_subdirectory() {
            let repo = create_test_git_repo();
            let subdir = repo.path().join("subdir");
            fs::create_dir(&subdir).unwrap();

            // Change to subdirectory
            std::env::set_current_dir(&subdir).unwrap();

            let root = get_repo_root();
            assert!(root.is_some());

            let root = root.unwrap();
            assert_eq!(root, repo.path().canonicalize().unwrap());
        }

        #[test]
        #[serial]
        fn test_returns_none_outside_repo() {
            let temp = TempDir::new().unwrap();
            std::env::set_current_dir(temp.path()).unwrap();

            let root = get_repo_root();
            assert!(root.is_none());
        }
    }
}
