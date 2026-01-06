//! Persistent artifact cache with SHA256 content addressing
//!
//! Provides global disk-based caching of compiled dynamic libraries
//! with content-addressed storage for deduplication and fast lookups.
//!
//! Based on ODD-0026 Section 5.1 (Global Artifact Cache)

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Cache errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to initialize cache directory: {0}")]
    InitFailed(#[from] std::io::Error),

    #[error("Cache directory not found")]
    NotFound,

    #[error("Failed to load cache index: {0}")]
    IndexLoadFailed(String),

    #[error("Failed to save cache index: {0}")]
    IndexSaveFailed(String),

    #[error("Artifact not found for key: {0}")]
    ArtifactNotFound(String),

    #[error("Failed to copy artifact: {0}")]
    CopyFailed(String),
}

pub type Result<T> = std::result::Result<T, CacheError>;

/// Cache index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// SHA256 hash key
    key: String,

    /// Path to cached artifact
    path: PathBuf,

    /// Timestamp when cached
    cached_at: u64,

    /// Size in bytes
    size_bytes: u64,
}

/// Cache index stored on disk
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CacheIndex {
    /// Map from cache key to entry
    entries: HashMap<String, CacheEntry>,

    /// Index format version
    version: u32,
}

/// Global artifact cache
///
/// Provides persistent caching of compiled dynamic libraries using
/// SHA256 content-addressed storage. The cache is shared across all
/// REPL sessions and persists across restarts.
///
/// # Cache Key Algorithm (ODD-0026)
///
/// ```text
/// SHA256(
///     source_code +
///     dependencies +
///     optimization_level +
///     source_map_config
/// )
/// ```
///
/// # Directory Structure
///
/// ```text
/// ~/.cache/oxur/artifacts/
/// ├── index.json                 # Cache index
/// ├── <sha256-1>/
/// │   ├── lib.so                 # Compiled artifact (Linux)
/// │   └── metadata.json          # Entry metadata
/// ├── <sha256-2>/
/// │   ├── lib.dylib              # Compiled artifact (macOS)
/// │   └── metadata.json
/// └── ...
/// ```
///
/// # Examples
///
/// ```
/// use oxur_repl::cache::ArtifactCache;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut cache = ArtifactCache::new()?;
///
/// // Generate cache key from source
/// let key = cache.generate_key("(+ 1 2)", &[], 0, "default");
///
/// // Check if artifact exists
/// if let Some(path) = cache.get(&key)? {
///     println!("Cache hit: {:?}", path);
/// } else {
///     println!("Cache miss, need to compile");
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ArtifactCache {
    /// Cache root directory
    cache_dir: PathBuf,

    /// In-memory index
    index: CacheIndex,
}

impl ArtifactCache {
    /// Create or open the global artifact cache
    ///
    /// Uses platform-specific cache directory:
    /// - Linux: `~/.cache/oxur/artifacts/`
    /// - macOS: `~/Library/Caches/oxur/artifacts/`
    /// - Windows: `%LOCALAPPDATA%\oxur\artifacts\`
    ///
    /// Can be overridden with `OXUR_CACHE_DIR` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created or index cannot be loaded
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;

        let index = Self::load_index(&cache_dir)?;

        Ok(Self { cache_dir, index })
    }

    /// Create a new artifact cache with a specific directory
    ///
    /// This is primarily useful for testing with isolated cache directories.
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Path to use as the cache directory
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be created or index cannot be loaded
    pub fn with_directory(cache_dir: impl AsRef<Path>) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        fs::create_dir_all(&cache_dir)?;

        let index = Self::load_index(&cache_dir)?;

        Ok(Self { cache_dir, index })
    }

    /// Get the cache directory path
    fn get_cache_dir() -> Result<PathBuf> {
        // Check environment variable override
        if let Ok(dir) = std::env::var("OXUR_CACHE_DIR") {
            return Ok(PathBuf::from(dir).join("artifacts"));
        }

        // Use platform-specific cache directory
        dirs::cache_dir().ok_or(CacheError::NotFound).map(|dir| dir.join("oxur").join("artifacts"))
    }

    /// Load cache index from disk
    fn load_index(cache_dir: &Path) -> Result<CacheIndex> {
        let index_path = cache_dir.join("index.json");

        if !index_path.exists() {
            // No index yet, create empty
            return Ok(CacheIndex { version: 1, ..Default::default() });
        }

        let content = fs::read_to_string(&index_path)
            .map_err(|e| CacheError::IndexLoadFailed(format!("Failed to read index: {}", e)))?;

        serde_json::from_str(&content)
            .map_err(|e| CacheError::IndexLoadFailed(format!("Failed to parse index JSON: {}", e)))
    }

    /// Save cache index to disk
    fn save_index(&self) -> Result<()> {
        let index_path = self.cache_dir.join("index.json");

        let content = serde_json::to_string_pretty(&self.index).map_err(|e| {
            CacheError::IndexSaveFailed(format!("Failed to serialize index: {}", e))
        })?;

        fs::write(&index_path, content)
            .map_err(|e| CacheError::IndexSaveFailed(format!("Failed to write index: {}", e)))?;

        Ok(())
    }

    /// Generate cache key from compilation inputs
    ///
    /// # Arguments
    ///
    /// * `source` - Source code
    /// * `dependencies` - List of dependency names and versions
    /// * `opt_level` - Optimization level (0-3)
    /// * `source_map_config` - Source map configuration identifier
    ///
    /// # Returns
    ///
    /// SHA256 hash as hex string
    pub fn generate_key(
        &self,
        source: impl AsRef<str>,
        dependencies: &[(&str, &str)],
        opt_level: u8,
        source_map_config: impl AsRef<str>,
    ) -> String {
        let mut hasher = Sha256::new();

        // Hash source code
        hasher.update(source.as_ref().as_bytes());

        // Hash dependencies (sorted for consistency)
        let mut deps = dependencies.to_vec();
        deps.sort();
        for (name, version) in deps {
            hasher.update(name.as_bytes());
            hasher.update(version.as_bytes());
        }

        // Hash optimization level
        hasher.update([opt_level]);

        // Hash source map config
        hasher.update(source_map_config.as_ref().as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Get cached artifact by key
    ///
    /// Returns the path to the cached dynamic library if it exists.
    ///
    /// # Arguments
    ///
    /// * `key` - SHA256 cache key
    ///
    /// # Returns
    ///
    /// `Some(path)` if artifact exists, `None` otherwise
    pub fn get(&self, key: impl AsRef<str>) -> Result<Option<PathBuf>> {
        let key = key.as_ref();

        if let Some(entry) = self.index.entries.get(key) {
            if entry.path.exists() {
                Ok(Some(entry.path.clone()))
            } else {
                // Entry exists but file is missing - index is stale
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Insert artifact into cache
    ///
    /// Copies the artifact to cache directory and updates the index.
    ///
    /// # Arguments
    ///
    /// * `key` - SHA256 cache key
    /// * `artifact_path` - Path to compiled artifact
    ///
    /// # Errors
    ///
    /// Returns error if artifact cannot be copied or index cannot be saved
    pub fn insert(&mut self, key: impl AsRef<str>, artifact_path: &Path) -> Result<PathBuf> {
        let key = key.as_ref();

        // Create cache directory for this artifact
        let cache_path = self.cache_dir.join(key);
        fs::create_dir_all(&cache_path)?;

        // Determine artifact filename based on platform
        let artifact_filename = Self::artifact_filename();
        let cached_artifact = cache_path.join(artifact_filename);

        // Copy artifact to cache
        fs::copy(artifact_path, &cached_artifact).map_err(|e| {
            CacheError::CopyFailed(format!(
                "Failed to copy {} to {}: {}",
                artifact_path.display(),
                cached_artifact.display(),
                e
            ))
        })?;

        // Get artifact metadata
        let metadata = fs::metadata(&cached_artifact)?;
        let size_bytes = metadata.len();
        let cached_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time is before UNIX epoch")
            .as_secs();

        // Update index
        let entry = CacheEntry {
            key: key.to_string(),
            path: cached_artifact.clone(),
            cached_at,
            size_bytes,
        };

        self.index.entries.insert(key.to_string(), entry);
        self.save_index()?;

        // Check if we need to evict old entries to stay under size limit
        let _ = self.check_and_evict(); // Best effort, don't fail insert on eviction issues

        Ok(cached_artifact)
    }

    /// Get platform-specific artifact filename
    #[cfg(target_os = "linux")]
    fn artifact_filename() -> &'static str {
        "lib.so"
    }

    #[cfg(target_os = "macos")]
    fn artifact_filename() -> &'static str {
        "lib.dylib"
    }

    #[cfg(target_os = "windows")]
    fn artifact_filename() -> &'static str {
        "lib.dll"
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    fn artifact_filename() -> &'static str {
        "lib.so" // Default fallback
    }

    /// Clear all cached artifacts
    ///
    /// Removes all artifacts from cache directory and resets index.
    pub fn clear(&mut self) -> Result<()> {
        // Remove all artifact directories
        for entry in self.index.entries.values() {
            if let Some(parent) = entry.path.parent() {
                let _ = fs::remove_dir_all(parent);
            }
        }

        // Reset index
        self.index.entries.clear();
        self.save_index()?;

        Ok(())
    }

    /// Get cache statistics
    ///
    /// Returns (entry_count, total_size_bytes)
    pub fn stats(&self) -> (usize, u64) {
        let count = self.index.entries.len();
        let total_size = self.index.entries.values().map(|e| e.size_bytes).sum();
        (count, total_size)
    }

    /// List all cache keys
    pub fn keys(&self) -> Vec<String> {
        self.index.entries.keys().cloned().collect()
    }

    /// Evict least recently used cache entries
    ///
    /// Removes oldest entries until total cache size is below the limit.
    /// Default limit is 1GB. Can be configured via `OXUR_CACHE_MAX_SIZE_MB` environment variable.
    ///
    /// # Arguments
    ///
    /// * `max_size_bytes` - Maximum allowed cache size in bytes. If None, uses default (1GB)
    ///
    /// # Returns
    ///
    /// Number of entries evicted
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use oxur_repl::cache::ArtifactCache;
    ///
    /// let mut cache = ArtifactCache::new()?;
    /// let evicted = cache.evict_lru(Some(100 * 1024 * 1024))?; // Limit to 100MB
    /// println!("Evicted {} cache entries", evicted);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn evict_lru(&mut self, max_size_bytes: Option<u64>) -> Result<usize> {
        // Get max size from parameter or environment, default to 1GB
        let max_size = max_size_bytes.unwrap_or_else(|| {
            std::env::var("OXUR_CACHE_MAX_SIZE_MB")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .map(|mb| mb * 1024 * 1024)
                .unwrap_or(1024 * 1024 * 1024) // 1GB default
        });

        let (_count, total_size) = self.stats();
        if total_size <= max_size {
            return Ok(0); // No eviction needed
        }

        // Sort entries by cached_at (oldest first) and collect keys to evict
        let mut entries: Vec<_> =
            self.index.entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        entries.sort_by_key(|(_, entry)| entry.cached_at);

        let mut evicted = 0;
        let mut current_size = total_size;
        let mut keys_to_remove = Vec::new();

        // Collect keys to evict until under limit
        for (key, entry) in entries {
            if current_size <= max_size {
                break;
            }

            keys_to_remove.push(key.clone());
            current_size -= entry.size_bytes;
        }

        // Remove artifacts and entries
        for key in keys_to_remove {
            if let Some(entry) = self.index.entries.get(&key) {
                // Remove artifact directory (best effort, ignore errors)
                let artifact_dir = self.cache_dir.join(&entry.key);
                if artifact_dir.exists() {
                    let _ = fs::remove_dir_all(&artifact_dir); // Ignore errors during eviction
                }

                // Remove from index
                self.index.entries.remove(&key);
                evicted += 1;
            }
        }

        // Save updated index
        if evicted > 0 {
            self.save_index()?;
        }

        Ok(evicted)
    }

    /// Get total cache size in bytes
    pub fn total_size_bytes(&self) -> u64 {
        self.stats().1
    }

    /// Check if cache is over size limit and evict if needed
    ///
    /// This is automatically called after `insert()` operations.
    /// Can also be called manually for periodic cleanup.
    ///
    /// # Returns
    ///
    /// Number of entries evicted
    pub fn check_and_evict(&mut self) -> Result<usize> {
        self.evict_lru(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_cache() -> (ArtifactCache, PathBuf) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_dir =
            env::temp_dir().join(format!("oxur-test-cache-{}-{}", std::process::id(), id));
        env::set_var("OXUR_CACHE_DIR", &test_dir);

        let mut cache = ArtifactCache::new().expect("Failed to create cache");
        cache.clear().expect("Failed to clear cache");
        (cache, test_dir)
    }

    fn cleanup_test_cache(test_dir: PathBuf) {
        env::remove_var("OXUR_CACHE_DIR");
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_cache_creation() {
        let (cache, test_dir) = setup_test_cache();
        assert!(cache.cache_dir.exists());
        cleanup_test_cache(test_dir);
    }

    #[test]
    fn test_generate_key_deterministic() {
        let (cache, test_dir) = setup_test_cache();

        let key1 = cache.generate_key("(+ 1 2)", &[], 0, "default");
        let key2 = cache.generate_key("(+ 1 2)", &[], 0, "default");

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 64); // SHA256 produces 64 hex chars

        cleanup_test_cache(test_dir);
    }

    #[test]
    fn test_generate_key_different_sources() {
        let (cache, test_dir) = setup_test_cache();

        let key1 = cache.generate_key("(+ 1 2)", &[], 0, "default");
        let key2 = cache.generate_key("(+ 2 3)", &[], 0, "default");

        assert_ne!(key1, key2);

        cleanup_test_cache(test_dir);
    }

    #[test]
    fn test_generate_key_different_dependencies() {
        let (cache, test_dir) = setup_test_cache();

        let key1 = cache.generate_key("(+ 1 2)", &[("foo", "1.0")], 0, "default");
        let key2 = cache.generate_key("(+ 1 2)", &[("foo", "2.0")], 0, "default");

        assert_ne!(key1, key2);

        cleanup_test_cache(test_dir);
    }

    #[test]
    fn test_insert_and_get() {
        let (mut cache, test_dir) = setup_test_cache();

        // Create a temporary artifact file
        let temp_artifact = env::temp_dir().join("test_artifact.so");
        fs::write(&temp_artifact, b"fake dylib content").expect("Failed to create temp artifact");

        // Generate key and insert
        let key = cache.generate_key("(+ 1 2)", &[], 0, "default");
        let cached_path = cache.insert(&key, &temp_artifact).expect("Failed to insert artifact");

        assert!(cached_path.exists());

        // Retrieve from cache
        let retrieved = cache.get(&key).expect("Failed to get artifact");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), cached_path);

        // Cleanup
        fs::remove_file(&temp_artifact).ok();
        cleanup_test_cache(test_dir);
    }

    #[test]
    fn test_get_nonexistent() {
        let (cache, test_dir) = setup_test_cache();

        let result = cache.get("nonexistent-key").expect("Failed to query cache");
        assert!(result.is_none());

        cleanup_test_cache(test_dir);
    }

    #[test]
    fn test_cache_stats() {
        let (mut cache, test_dir) = setup_test_cache();

        let temp_artifact = env::temp_dir().join("test_stats.so");
        fs::write(&temp_artifact, b"test content").expect("Failed to create temp artifact");

        let key = cache.generate_key("test", &[], 0, "default");
        cache.insert(&key, &temp_artifact).expect("Failed to insert");

        let (count, size) = cache.stats();
        assert_eq!(count, 1);
        assert!(size > 0);

        fs::remove_file(&temp_artifact).ok();
        cleanup_test_cache(test_dir);
    }

    #[test]
    fn test_cache_clear() {
        let (mut cache, test_dir) = setup_test_cache();

        let temp_artifact = env::temp_dir().join("test_clear.so");
        fs::write(&temp_artifact, b"test").expect("Failed to create temp artifact");

        let key = cache.generate_key("test", &[], 0, "default");
        cache.insert(&key, &temp_artifact).expect("Failed to insert");

        let (count_before, _) = cache.stats();
        assert_eq!(count_before, 1);

        cache.clear().expect("Failed to clear cache");

        let (count_after, _) = cache.stats();
        assert_eq!(count_after, 0);

        fs::remove_file(&temp_artifact).ok();
        cleanup_test_cache(test_dir);
    }

    #[test]
    fn test_cache_persistence() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1000);

        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_dir =
            env::temp_dir().join(format!("oxur-test-persist-{}-{}", std::process::id(), id));

        let temp_artifact = env::temp_dir().join(format!("test_persist_{}.so", id));
        fs::write(&temp_artifact, b"persist test").expect("Failed to create temp artifact");

        let (key, cached_path) = {
            // Use with_directory for isolated test cache (no environment variable needed)
            let mut cache =
                ArtifactCache::with_directory(&test_dir).expect("Failed to create cache");

            let key = cache.generate_key("persist", &[], 0, "default");
            let cached_path = cache.insert(&key, &temp_artifact).expect("Failed to insert");

            (key, cached_path)
        }; // cache dropped here - index should be persisted to disk

        // Verify the cached file actually exists
        assert!(cached_path.exists(), "Cached file should exist at {:?}", cached_path);

        // Create new cache instance with same directory - should load index from disk
        let cache2 = ArtifactCache::with_directory(&test_dir).expect("Failed to create cache2");
        let retrieved = cache2.get(&key).expect("Failed to get from cache2");

        assert!(retrieved.is_some(), "Should retrieve cached artifact");
        if let Some(path) = retrieved {
            assert_eq!(path, cached_path, "Retrieved path should match cached path");
        }

        // Cleanup
        fs::remove_file(&temp_artifact).ok();
        cleanup_test_cache(test_dir);
    }
}
