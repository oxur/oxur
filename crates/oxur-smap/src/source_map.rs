use crate::{NodeId, SourcePos};
use std::collections::HashMap;

/// Tracks AST transformations for error reporting
///
/// This is the core of the source mapping system. It maintains three
/// separate mappings:
///
/// 1. surface_positions: NodeId → SourcePos (from parser)
/// 2. surface_to_core: NodeId → NodeId (from macro expansion)
/// 3. core_to_rust: NodeId → NodeId (from lowering)
///
/// These three maps enable backward traversal from a Rust compiler
/// error position back to the original Oxur source code.
#[derive(Debug, Default)]
pub struct SourceMap {
    /// Surface Form positions (recorded during parsing)
    surface_positions: HashMap<NodeId, SourcePos>,

    /// Transformation chain: Surface → Core (recorded during expansion)
    surface_to_core: HashMap<NodeId, NodeId>,

    /// Transformation chain: Core → Rust (recorded during lowering)
    core_to_rust: HashMap<NodeId, NodeId>,
}

impl SourceMap {
    /// Create a new empty source map
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a surface form node position
    ///
    /// Called by the parser when creating surface form AST nodes.
    ///
    /// # Example
    /// ```
    /// use oxur_smap::{SourceMap, NodeId, SourcePos};
    ///
    /// let mut map = SourceMap::new();
    /// let node = NodeId::from_raw(100);
    /// let pos = SourcePos::repl(1, 1, 10);
    ///
    /// map.record_surface_node(node, pos);
    /// assert_eq!(map.get_surface_position(&node).unwrap().line, 1);
    /// ```
    pub fn record_surface_node(&mut self, node: NodeId, pos: SourcePos) {
        self.surface_positions.insert(node, pos);
    }

    /// Record an expansion transformation
    ///
    /// Called by the macro expander when transforming a surface form
    /// node into a core form node.
    ///
    /// # Example
    /// ```
    /// use oxur_smap::{SourceMap, NodeId};
    ///
    /// let mut map = SourceMap::new();
    /// let surface = NodeId::from_raw(100);
    /// let core = NodeId::from_raw(200);
    ///
    /// map.record_expansion(surface, core);
    /// assert_eq!(map.get_core_from_surface(&surface), Some(&core));
    /// ```
    pub fn record_expansion(&mut self, surface: NodeId, core: NodeId) {
        self.surface_to_core.insert(surface, core);
    }

    /// Record a lowering transformation
    ///
    /// Called by the Rust AST generator when transforming a core form
    /// node into a Rust AST node.
    ///
    /// # Example
    /// ```
    /// use oxur_smap::{SourceMap, NodeId};
    ///
    /// let mut map = SourceMap::new();
    /// let core = NodeId::from_raw(200);
    /// let rust = NodeId::from_raw(300);
    ///
    /// map.record_lowering(core, rust);
    /// assert_eq!(map.get_rust_from_core(&core), Some(&rust));
    /// ```
    pub fn record_lowering(&mut self, core: NodeId, rust: NodeId) {
        self.core_to_rust.insert(core, rust);
    }

    /// Look up the original source position for a Rust AST node
    ///
    /// This performs backward traversal through the transformation chain:
    /// Rust → Core → Surface → SourcePos
    ///
    /// Returns None if any link in the chain is missing.
    ///
    /// # Example
    /// ```
    /// use oxur_smap::{SourceMap, NodeId, SourcePos};
    ///
    /// let mut map = SourceMap::new();
    /// let surface = NodeId::from_raw(100);
    /// let core = NodeId::from_raw(200);
    /// let rust = NodeId::from_raw(300);
    /// let pos = SourcePos::repl(1, 5, 10);
    ///
    /// map.record_surface_node(surface, pos.clone());
    /// map.record_expansion(surface, core);
    /// map.record_lowering(core, rust);
    ///
    /// let result = map.lookup(&rust).unwrap();
    /// assert_eq!(result.line, 1);
    /// assert_eq!(result.column, 5);
    /// ```
    pub fn lookup(&self, rust_node: &NodeId) -> Option<SourcePos> {
        // Step 1: Rust → Core
        // Find the core node that maps to this rust node
        let core_node = self.core_to_rust.iter().find(|(_, &r)| r == *rust_node).map(|(c, _)| c)?;

        // Step 2: Core → Surface
        // Find the surface node that maps to this core node
        let surface_node =
            self.surface_to_core.iter().find(|(_, &c)| c == *core_node).map(|(s, _)| s)?;

        // Step 3: Surface → SourcePos
        // Look up the original position
        self.surface_positions.get(surface_node).cloned()
    }

    /// Get surface position directly (for testing/debugging)
    pub fn get_surface_position(&self, node: &NodeId) -> Option<&SourcePos> {
        self.surface_positions.get(node)
    }

    /// Get core node from surface node (for testing/debugging)
    pub fn get_core_from_surface(&self, surface: &NodeId) -> Option<&NodeId> {
        self.surface_to_core.get(surface)
    }

    /// Get rust node from core node (for testing/debugging)
    pub fn get_rust_from_core(&self, core: &NodeId) -> Option<&NodeId> {
        self.core_to_rust.get(core)
    }

    /// Get statistics about the source map (for debugging)
    pub fn stats(&self) -> SourceMapStats {
        SourceMapStats {
            surface_nodes: self.surface_positions.len(),
            expansions: self.surface_to_core.len(),
            lowerings: self.core_to_rust.len(),
        }
    }

    /// Generate a content hash for cache key generation
    ///
    /// This hash includes the structure of all transformations but NOT
    /// the actual source positions. This allows cached artifacts to be
    /// reused when the transformation structure is identical, even if
    /// line numbers have changed.
    ///
    /// # Design Decision: Structure-Only Hashing
    ///
    /// We hash the transformation graph (NodeId → NodeId mappings) but
    /// NOT the surface positions. This means:
    ///
    /// - Same code structure → Same hash (even if moved to different line)
    /// - Cache hits more frequent (position changes don't invalidate)
    /// - Trade-off: Very subtle semantic changes might be missed
    ///
    /// For v1.0, this is acceptable. If needed, we can add a flag for
    /// "strict hashing" that includes positions in v1.1+.
    pub fn content_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash transformation structure (not positions)
        // Sort for deterministic hashing
        let mut expansions: Vec<_> = self.surface_to_core.iter().collect();
        expansions.sort_by_key(|(k, _)| *k);
        for (surface, core) in expansions {
            surface.hash(&mut hasher);
            core.hash(&mut hasher);
        }

        let mut lowerings: Vec<_> = self.core_to_rust.iter().collect();
        lowerings.sort_by_key(|(k, _)| *k);
        for (core, rust) in lowerings {
            core.hash(&mut hasher);
            rust.hash(&mut hasher);
        }

        hasher.finish()
    }
}

/// Statistics about a SourceMap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceMapStats {
    pub surface_nodes: usize,
    pub expansions: usize,
    pub lowerings: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_map_empty() {
        let map = SourceMap::new();
        let node = NodeId::from_raw(100);
        assert!(map.lookup(&node).is_none());
    }

    #[test]
    fn test_source_map_default() {
        let map = SourceMap::default();
        let node = NodeId::from_raw(100);
        assert!(map.lookup(&node).is_none());
    }

    #[test]
    fn test_source_map_record_surface() {
        let mut map = SourceMap::new();
        let node = NodeId::from_raw(100);
        let pos = SourcePos::repl(1, 1, 10);

        map.record_surface_node(node, pos.clone());
        assert_eq!(map.get_surface_position(&node).unwrap().line, 1);
    }

    #[test]
    fn test_source_map_full_chain() {
        let mut map = SourceMap::new();

        // Create a full transformation chain
        let surface = NodeId::from_raw(100);
        let core = NodeId::from_raw(200);
        let rust = NodeId::from_raw(300);
        let pos = SourcePos::repl(1, 5, 10);

        // Record transformations
        map.record_surface_node(surface, pos.clone());
        map.record_expansion(surface, core);
        map.record_lowering(core, rust);

        // Lookup should traverse the full chain
        let result = map.lookup(&rust).unwrap();
        assert_eq!(result.line, 1);
        assert_eq!(result.column, 5);
        assert_eq!(result.length, 10);
    }

    #[test]
    fn test_source_map_broken_chain_no_lowering() {
        let mut map = SourceMap::new();

        let surface = NodeId::from_raw(100);
        let core = NodeId::from_raw(200);
        let rust = NodeId::from_raw(300);
        let pos = SourcePos::repl(1, 1, 10);

        // Only record surface and expansion (missing lowering)
        map.record_surface_node(surface, pos);
        map.record_expansion(surface, core);

        // Lookup should fail (no lowering recorded)
        assert!(map.lookup(&rust).is_none());
    }

    #[test]
    fn test_source_map_broken_chain_no_expansion() {
        let mut map = SourceMap::new();

        let surface = NodeId::from_raw(100);
        let rust = NodeId::from_raw(300);
        let pos = SourcePos::repl(1, 1, 10);

        // Only record surface (missing expansion and lowering)
        map.record_surface_node(surface, pos);

        // Lookup should fail (no expansion recorded)
        assert!(map.lookup(&rust).is_none());
    }

    #[test]
    fn test_source_map_stats() {
        let mut map = SourceMap::new();

        let surface1 = NodeId::from_raw(100);
        let surface2 = NodeId::from_raw(101);
        let core1 = NodeId::from_raw(200);
        let rust1 = NodeId::from_raw(300);
        let pos1 = SourcePos::repl(1, 1, 5);
        let pos2 = SourcePos::repl(2, 1, 8);

        map.record_surface_node(surface1, pos1);
        map.record_surface_node(surface2, pos2);
        map.record_expansion(surface1, core1);
        map.record_lowering(core1, rust1);

        let stats = map.stats();
        assert_eq!(stats.surface_nodes, 2);
        assert_eq!(stats.expansions, 1);
        assert_eq!(stats.lowerings, 1);
    }

    #[test]
    fn test_source_map_multiple_nodes() {
        let mut map = SourceMap::new();

        // Create two separate transformation chains
        let surface1 = NodeId::from_raw(100);
        let core1 = NodeId::from_raw(200);
        let rust1 = NodeId::from_raw(300);
        let pos1 = SourcePos::new("file1.ox".to_string(), 1, 1, 5);

        let surface2 = NodeId::from_raw(101);
        let core2 = NodeId::from_raw(201);
        let rust2 = NodeId::from_raw(301);
        let pos2 = SourcePos::new("file2.ox".to_string(), 10, 20, 15);

        // Record both chains
        map.record_surface_node(surface1, pos1.clone());
        map.record_expansion(surface1, core1);
        map.record_lowering(core1, rust1);

        map.record_surface_node(surface2, pos2.clone());
        map.record_expansion(surface2, core2);
        map.record_lowering(core2, rust2);

        // Lookup both should work independently
        let result1 = map.lookup(&rust1).unwrap();
        assert_eq!(result1.file, "file1.ox");
        assert_eq!(result1.line, 1);

        let result2 = map.lookup(&rust2).unwrap();
        assert_eq!(result2.file, "file2.ox");
        assert_eq!(result2.line, 10);
    }

    #[test]
    fn test_content_hash_deterministic() {
        let mut map1 = SourceMap::new();
        let mut map2 = SourceMap::new();

        let surface = NodeId::from_raw(100);
        let core = NodeId::from_raw(200);
        let rust = NodeId::from_raw(300);

        // Build identical transformation structures
        map1.record_expansion(surface, core);
        map1.record_lowering(core, rust);

        map2.record_expansion(surface, core);
        map2.record_lowering(core, rust);

        // Hashes should be identical
        assert_eq!(map1.content_hash(), map2.content_hash());
    }

    #[test]
    fn test_content_hash_position_independent() {
        let mut map1 = SourceMap::new();
        let mut map2 = SourceMap::new();

        let surface = NodeId::from_raw(100);
        let core = NodeId::from_raw(200);
        let pos1 = SourcePos::repl(1, 1, 10);
        let pos2 = SourcePos::repl(99, 1, 10); // Different line

        // Same structure, different positions
        map1.record_surface_node(surface, pos1);
        map1.record_expansion(surface, core);

        map2.record_surface_node(surface, pos2);
        map2.record_expansion(surface, core);

        // Hashes should still be identical (positions not included)
        assert_eq!(map1.content_hash(), map2.content_hash());
    }

    #[test]
    fn test_content_hash_structure_sensitive() {
        let mut map1 = SourceMap::new();
        let mut map2 = SourceMap::new();

        let surface = NodeId::from_raw(100);
        let core1 = NodeId::from_raw(200);
        let core2 = NodeId::from_raw(201); // Different core node

        map1.record_expansion(surface, core1);
        map2.record_expansion(surface, core2);

        // Different structure → different hash
        assert_ne!(map1.content_hash(), map2.content_hash());
    }

    #[test]
    fn test_content_hash_empty_map() {
        let map1 = SourceMap::new();
        let map2 = SourceMap::new();

        // Empty maps should have the same hash
        assert_eq!(map1.content_hash(), map2.content_hash());
    }

    #[test]
    fn test_content_hash_order_independence() {
        let mut map1 = SourceMap::new();
        let mut map2 = SourceMap::new();

        let surface1 = NodeId::from_raw(100);
        let surface2 = NodeId::from_raw(101);
        let core1 = NodeId::from_raw(200);
        let core2 = NodeId::from_raw(201);

        // Add mappings in different orders
        map1.record_expansion(surface1, core1);
        map1.record_expansion(surface2, core2);

        map2.record_expansion(surface2, core2);
        map2.record_expansion(surface1, core1);

        // Hashes should be identical (sorting ensures order independence)
        assert_eq!(map1.content_hash(), map2.content_hash());
    }
}
