//! Compilation cache for incremental builds

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Cache entry for a compiled file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// SHA-256 hash of the source file content
    content_hash: String,
    /// Last modification timestamp
    timestamp: u64,
}

/// Compilation cache for tracking which files need recompilation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CompilationCache {
    /// Map of file path to cache entry
    entries: HashMap<PathBuf, CacheEntry>,
}

impl CompilationCache {
    /// Load cache from file, or create new if doesn't exist
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Failed to read cache file: {}", path.display()))?;

            let cache: Self = serde_json::from_str(&contents)
                .with_context(|| format!("Failed to parse cache file: {}", path.display()))?;

            Ok(cache)
        } else {
            Ok(Self::default())
        }
    }

    /// Save cache to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self).context("Failed to serialize cache")?;

        fs::write(path, contents)
            .with_context(|| format!("Failed to write cache file: {}", path.display()))?;

        Ok(())
    }

    /// Check if a file needs recompilation
    ///
    /// Returns true if:
    /// - File is not in cache
    /// - File content hash changed
    /// - File timestamp changed
    pub fn needs_recompile(&self, file: &Path) -> Result<bool> {
        let current_hash = Self::compute_hash(file)?;
        let current_timestamp = Self::get_timestamp(file)?;

        if let Some(entry) = self.entries.get(file) {
            // Check if hash or timestamp changed
            Ok(entry.content_hash != current_hash || entry.timestamp != current_timestamp)
        } else {
            // Not in cache, needs compilation
            Ok(true)
        }
    }

    /// Update cache entry for a file
    pub fn update(&mut self, file: &Path) -> Result<()> {
        let content_hash = Self::compute_hash(file)?;
        let timestamp = Self::get_timestamp(file)?;

        self.entries.insert(file.to_path_buf(), CacheEntry { content_hash, timestamp });

        Ok(())
    }

    /// Compute SHA-256 hash of file contents
    fn compute_hash(file: &Path) -> Result<String> {
        let contents = fs::read(file)
            .with_context(|| format!("Failed to read file for hashing: {}", file.display()))?;

        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// Get file modification timestamp
    fn get_timestamp(file: &Path) -> Result<u64> {
        let metadata = fs::metadata(file)
            .with_context(|| format!("Failed to get metadata for: {}", file.display()))?;

        let modified = metadata.modified().context("Failed to get modification time")?;

        Ok(modified.duration_since(std::time::UNIX_EPOCH).context("Invalid timestamp")?.as_secs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_compute_hash() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file = temp_dir.path().join("test.oxr");

        fs::write(&file, "test content")?;
        let hash1 = CompilationCache::compute_hash(&file)?;

        // Same content, same hash
        let hash2 = CompilationCache::compute_hash(&file)?;
        assert_eq!(hash1, hash2);

        // Different content, different hash
        fs::write(&file, "different content")?;
        let hash3 = CompilationCache::compute_hash(&file)?;
        assert_ne!(hash1, hash3);

        Ok(())
    }

    #[test]
    fn test_needs_recompile_new_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file = temp_dir.path().join("test.oxr");
        fs::write(&file, "test")?;

        let cache = CompilationCache::default();
        assert!(cache.needs_recompile(&file)?);

        Ok(())
    }

    #[test]
    fn test_needs_recompile_unchanged() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file = temp_dir.path().join("test.oxr");
        fs::write(&file, "test")?;

        let mut cache = CompilationCache::default();
        cache.update(&file)?;

        // Should not need recompilation
        assert!(!cache.needs_recompile(&file)?);

        Ok(())
    }

    #[test]
    fn test_needs_recompile_content_changed() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file = temp_dir.path().join("test.oxr");
        fs::write(&file, "original")?;

        let mut cache = CompilationCache::default();
        cache.update(&file)?;

        // Change content
        thread::sleep(Duration::from_millis(10));
        fs::write(&file, "modified")?;

        // Should need recompilation
        assert!(cache.needs_recompile(&file)?);

        Ok(())
    }

    #[test]
    fn test_save_and_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_file = temp_dir.path().join("cache.json");
        let test_file = temp_dir.path().join("test.oxr");

        fs::write(&test_file, "test")?;

        // Create and save cache
        let mut cache1 = CompilationCache::default();
        cache1.update(&test_file)?;
        cache1.save(&cache_file)?;

        // Load cache
        let cache2 = CompilationCache::load(&cache_file)?;

        // Should have same entry
        assert!(!cache2.needs_recompile(&test_file)?);

        Ok(())
    }

    #[test]
    fn test_load_nonexistent() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_file = temp_dir.path().join("nonexistent.json");

        let cache = CompilationCache::load(&cache_file)?;
        assert_eq!(cache.entries.len(), 0);

        Ok(())
    }
}
