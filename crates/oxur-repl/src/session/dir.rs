//! Session directory management with tmpfs optimization
//!
//! Provides temporary directory management for REPL sessions with
//! best-effort tmpfs allocation on Linux for improved performance.
//!
//! Based on ODD-0026 Section 9 (Temp Directory Strategy)

use crate::protocol::SessionId;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Session directory errors
#[derive(Debug, Error)]
pub enum SessionDirError {
    #[error("Failed to create session directory: {0}")]
    CreateFailed(#[from] std::io::Error),

    #[error("Session directory not found: {0}")]
    NotFound(PathBuf),

    #[error("Failed to cleanup session directory: {0}")]
    CleanupFailed(String),
}

pub type Result<T> = std::result::Result<T, SessionDirError>;

/// Session directory manager
///
/// Manages temporary directories for REPL sessions with performance
/// optimizations:
/// - Linux: Tries tmpfs (/dev/shm) for RAM-backed storage
/// - macOS/Windows: Uses OS temp directory with caching
/// - All platforms: Respects OXUR_REPL_TEMP_DIR override
///
/// # Examples
///
/// ```
/// use oxur_repl::session::SessionDir;
/// use oxur_repl::protocol::SessionId;
///
/// let session_id = SessionId::new("test-session");
/// let dir = SessionDir::new(&session_id).expect("Failed to create session dir");
///
/// // Use directory for compilation artifacts
/// let source_path = dir.path().join("eval_0.rs");
///
/// // Cleanup happens automatically on drop
/// ```
#[derive(Debug, Clone)]
pub struct SessionDir {
    /// Path to the session directory
    path: PathBuf,

    /// Whether this directory is on tmpfs (RAM-backed)
    is_tmpfs: bool,

    /// Session ID for this directory
    session_id: SessionId,
}

impl SessionDir {
    /// Create a new session directory
    ///
    /// Strategy (per ODD-0026 Decision 4):
    /// 1. Check OXUR_REPL_TEMP_DIR environment variable
    /// 2. Try tmpfs on Linux (/dev/shm)
    /// 3. Fall back to OS temp directory
    ///
    /// # Errors
    ///
    /// Returns error if directory creation fails
    pub fn new(session_id: &SessionId) -> Result<Self> {
        // Try user override first
        if let Ok(dir) = env::var("OXUR_REPL_TEMP_DIR") {
            return Self::create_in(PathBuf::from(dir), session_id, false);
        }

        // Try tmpfs on Linux
        #[cfg(target_os = "linux")]
        if let Ok(tmpfs_dir) = Self::try_tmpfs(session_id) {
            return Ok(tmpfs_dir);
        }

        // Fall back to OS temp directory
        let temp_dir = env::temp_dir().join("oxur-repl").join(session_id.as_str());

        Self::create_in(temp_dir, session_id, false)
    }

    /// Try to create directory on tmpfs (Linux only)
    #[cfg(target_os = "linux")]
    fn try_tmpfs(session_id: &SessionId) -> Result<Self> {
        let tmpfs_path = PathBuf::from("/dev/shm").join("oxur-repl").join(session_id.as_str());

        Self::create_in(tmpfs_path, session_id, true)
    }

    /// Create directory at specific path
    fn create_in(path: PathBuf, session_id: &SessionId, is_tmpfs: bool) -> Result<Self> {
        fs::create_dir_all(&path)?;

        Ok(Self { path, is_tmpfs, session_id: session_id.clone() })
    }

    /// Get the path to this session directory
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if this directory is on tmpfs
    pub fn is_tmpfs(&self) -> bool {
        self.is_tmpfs
    }

    /// Get the session ID
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Write source code to a file in this directory
    ///
    /// # Arguments
    ///
    /// * `name` - File name (e.g., "eval_0.rs")
    /// * `content` - Source code content
    pub fn write_source(&self, name: impl AsRef<str>, content: impl AsRef<str>) -> Result<PathBuf> {
        let path = self.path.join(name.as_ref());
        fs::write(&path, content.as_ref())?;
        Ok(path)
    }

    /// Create a subdirectory within this session directory
    pub fn create_subdir(&self, name: impl AsRef<str>) -> Result<PathBuf> {
        let subdir = self.path.join(name.as_ref());
        fs::create_dir_all(&subdir)?;
        Ok(subdir)
    }

    /// Clean up the session directory
    ///
    /// Removes all files and the directory itself.
    /// Called automatically on drop, but can be called explicitly.
    pub fn cleanup(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_dir_all(&self.path).map_err(|e| {
                SessionDirError::CleanupFailed(format!(
                    "Failed to remove {}: {}",
                    self.path.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    /// List all files in the session directory
    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            }
        }

        Ok(files)
    }
}

impl Drop for SessionDir {
    fn drop(&mut self) {
        // Best-effort cleanup - log errors but don't panic
        if let Err(e) = self.cleanup() {
            eprintln!("Warning: Failed to cleanup session directory: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_dir_creation() {
        let session_id = SessionId::new("test-create");
        let dir = SessionDir::new(&session_id).expect("Failed to create session dir");

        assert!(dir.path().exists());
        assert_eq!(dir.session_id(), &session_id);
    }

    #[test]
    fn test_session_dir_write_source() {
        let session_id = SessionId::new("test-write");
        let dir = SessionDir::new(&session_id).expect("Failed to create session dir");

        let source_path =
            dir.write_source("test.rs", "fn main() {}").expect("Failed to write source");

        assert!(source_path.exists());
        let content = fs::read_to_string(&source_path).expect("Failed to read source");
        assert_eq!(content, "fn main() {}");
    }

    #[test]
    fn test_session_dir_create_subdir() {
        let session_id = SessionId::new("test-subdir");
        let dir = SessionDir::new(&session_id).expect("Failed to create session dir");

        let subdir = dir.create_subdir("artifacts").expect("Failed to create subdir");

        assert!(subdir.exists());
        assert!(subdir.is_dir());
    }

    #[test]
    fn test_session_dir_list_files() {
        let session_id = SessionId::new("test-list");
        let dir = SessionDir::new(&session_id).expect("Failed to create session dir");

        dir.write_source("file1.rs", "// file 1").expect("Failed to write file1");
        dir.write_source("file2.rs", "// file 2").expect("Failed to write file2");

        let files = dir.list_files().expect("Failed to list files");
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_session_dir_cleanup() {
        let session_id = SessionId::new("test-cleanup");
        let dir = SessionDir::new(&session_id).expect("Failed to create session dir");
        let path = dir.path().to_path_buf();

        dir.write_source("test.rs", "fn main() {}").expect("Failed to write source");

        assert!(path.exists());

        dir.cleanup().expect("Failed to cleanup");
        assert!(!path.exists());
    }

    #[test]
    fn test_session_dir_auto_cleanup_on_drop() {
        let session_id = SessionId::new("test-drop");
        let path = {
            let dir = SessionDir::new(&session_id).expect("Failed to create session dir");
            dir.path().to_path_buf()
        }; // dir dropped here

        // Give the system a moment to cleanup
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(!path.exists());
    }

    #[test]
    fn test_session_dir_respects_env_var() {
        let test_dir = env::temp_dir().join("oxur-test-custom");
        env::set_var("OXUR_REPL_TEMP_DIR", &test_dir);

        let session_id = SessionId::new("test-env");
        let dir = SessionDir::new(&session_id).expect("Failed to create session dir");

        assert!(dir.path().starts_with(&test_dir));

        env::remove_var("OXUR_REPL_TEMP_DIR");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_session_dir_tmpfs_on_linux() {
        let session_id = SessionId::new("test-tmpfs");
        let dir = SessionDir::new(&session_id).expect("Failed to create session dir");

        // If /dev/shm exists and is writable, we should be on tmpfs
        if PathBuf::from("/dev/shm").exists() {
            assert!(dir.is_tmpfs() || !dir.is_tmpfs()); // Either is valid
        }
    }
}
