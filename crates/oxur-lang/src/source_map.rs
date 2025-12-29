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
        Self { mappings: HashMap::new() }
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

    #[test]
    fn test_source_map_default() {
        let map = SourceMap::default();
        assert!(map.is_empty());
    }

    #[test]
    fn test_source_map_get_missing() {
        let map = SourceMap::new();
        assert!(map.get(NodeId::new(999)).is_none());
    }

    #[test]
    fn test_source_info_with_parent() {
        let mut map = SourceMap::new();
        let parent_id = NodeId::new(1);
        let child_id = NodeId::new(2);

        let parent_info = SourceInfo {
            location: Location { line: 1, column: 1 },
            original_text: "(parent)".to_string(),
            parent: None,
        };

        let child_info = SourceInfo {
            location: Location { line: 1, column: 5 },
            original_text: "child".to_string(),
            parent: Some(parent_id),
        };

        map.add(parent_id, parent_info);
        map.add(child_id, child_info);

        let retrieved = map.get(child_id).unwrap();
        assert_eq!(retrieved.parent, Some(parent_id));
    }

    #[test]
    fn test_source_info_clone() {
        let info = SourceInfo {
            location: Location { line: 10, column: 20 },
            original_text: "test".to_string(),
            parent: Some(NodeId::new(5)),
        };

        let cloned = info.clone();
        assert_eq!(cloned.location.line, 10);
        assert_eq!(cloned.location.column, 20);
        assert_eq!(cloned.original_text, "test");
        assert_eq!(cloned.parent, Some(NodeId::new(5)));
    }
}
