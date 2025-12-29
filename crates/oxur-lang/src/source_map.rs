//! Source Map
//!
//! Tracks the transformation of code through all compilation stages.
//! Essential for accurate error reporting.

use crate::core_forms::NodeId;
use crate::Location;
use std::collections::HashMap;

/// Source map tracks transformations through compilation
#[derive(Debug, Clone)]
pub struct SourceMap {
    mappings: HashMap<NodeId, SourceInfo>,
}

/// Information about a node's origin
#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub location: Location,
    pub original_text: String,
    pub parent: Option<NodeId>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    pub fn add(&mut self, node_id: NodeId, info: SourceInfo) {
        self.mappings.insert(node_id, info);
    }

    pub fn get(&self, node_id: NodeId) -> Option<&SourceInfo> {
        self.mappings.get(&node_id)
    }

    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_map() {
        let mut map = SourceMap::new();
        assert!(map.is_empty());

        let node_id = NodeId::new(1);
        let info = SourceInfo {
            location: Location { line: 1, column: 5 },
            original_text: "(+ 1 2)".to_string(),
            parent: None,
        };

        map.add(node_id, info);
        assert!(!map.is_empty());
        assert!(map.get(node_id).is_some());
    }
}
