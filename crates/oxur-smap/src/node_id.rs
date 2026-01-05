use std::sync::atomic::{AtomicU32, Ordering};

/// Unique identifier for AST nodes across all compilation stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    /// Create a new NodeId from a raw u32 value
    ///
    /// # Safety
    /// The caller must ensure uniqueness across all nodes.
    /// Typically only used for deserialization or testing.
    pub const fn from_raw(id: u32) -> Self {
        NodeId(id)
    }

    /// Get the raw u32 value
    pub const fn as_raw(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

/// Global NodeId generator using atomic counter
///
/// Thread-safe generation of unique NodeIds.
///
/// # Design Decision: Single Global Counter vs Stage Ranges
///
/// We use a single global atomic counter rather than per-stage ranges
/// (e.g., 100-199 for surface, 200-299 for core) because:
///
/// 1. Simpler implementation (single AtomicU32)
/// 2. No risk of range exhaustion
/// 3. NodeIds are opaque - internal structure doesn't matter
/// 4. Debuggability via SourceMap lookup, not NodeId values
///
/// If debugging requires it, we can add stage tagging later without
/// breaking the API.
pub struct NodeIdGenerator {
    next_id: AtomicU32,
}

impl NodeIdGenerator {
    /// Create a new generator starting from 1 (0 reserved for invalid)
    pub const fn new() -> Self {
        Self { next_id: AtomicU32::new(1) }
    }

    /// Generate the next unique NodeId
    ///
    /// # Panics
    /// Panics if we exceed u32::MAX nodes (2^32 - 1).
    /// This is unlikely in practice (~4 billion nodes).
    pub fn next(&self) -> NodeId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        assert!(id < u32::MAX, "NodeId counter overflow");
        NodeId(id)
    }

    /// Reset the generator (for testing only)
    #[cfg(test)]
    pub fn reset(&self) {
        self.next_id.store(1, Ordering::SeqCst);
    }
}

impl Default for NodeIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Global singleton generator
///
/// Thread-safe access to NodeId generation.
static GLOBAL_GENERATOR: NodeIdGenerator = NodeIdGenerator::new();

/// Generate a new unique NodeId
///
/// This is the primary API for obtaining NodeIds during compilation.
pub fn new_node_id() -> NodeId {
    GLOBAL_GENERATOR.next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_basic() {
        let id1 = NodeId::from_raw(42);
        let id2 = NodeId::from_raw(42);
        assert_eq!(id1, id2);
        assert_eq!(id1.as_raw(), 42);
    }

    #[test]
    fn test_node_id_display() {
        let id = NodeId::from_raw(123);
        assert_eq!(format!("{}", id), "NodeId(123)");
    }

    #[test]
    fn test_node_id_ordering() {
        let id1 = NodeId::from_raw(10);
        let id2 = NodeId::from_raw(20);
        assert!(id1 < id2);
        assert!(id2 > id1);
    }

    #[test]
    fn test_node_id_generator() {
        let gen = NodeIdGenerator::new();
        let id1 = gen.next();
        let id2 = gen.next();
        assert_ne!(id1, id2);
        assert_eq!(id1.as_raw() + 1, id2.as_raw());
    }

    #[test]
    fn test_global_generator_thread_safe() {
        use std::sync::Arc;
        use std::thread;

        let ids = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut handles = vec![];

        // Spawn 10 threads, each generating 100 NodeIds
        for _ in 0..10 {
            let ids_clone = Arc::clone(&ids);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let id = new_node_id();
                    ids_clone.lock().unwrap().push(id);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All 1000 IDs should be unique
        let mut id_set = std::collections::HashSet::new();
        for id in ids.lock().unwrap().iter() {
            assert!(id_set.insert(*id), "Duplicate NodeId generated");
        }
        assert_eq!(id_set.len(), 1000);
    }
}
