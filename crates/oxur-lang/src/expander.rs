//! Stage 2: Expand
//!
//! Converts Surface Forms into Core Forms through macro expansion and desugaring.
//! This is where syntactic sugar gets transformed into canonical forms.

use crate::parser::SurfaceForm;
use crate::core_forms::CoreForm;
use crate::Result;
use oxur_smap::SourceMap;

/// Expander handles macro expansion and desugaring
pub struct Expander {
    source_map: SourceMap,
}

impl Expander {
    pub fn new() -> Self {
        Self { source_map: SourceMap::new() }
    }

    /// Expand Surface Forms into Core Forms
    pub fn expand(&mut self, _forms: Vec<SurfaceForm>) -> Result<Vec<CoreForm>> {
        // Placeholder implementation
        Ok(vec![])
    }

    /// Get the source map after expansion
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Check if the source map is empty
    pub fn is_empty(&self) -> bool {
        let stats = self.source_map.stats();
        stats.surface_nodes == 0 && stats.expansions == 0 && stats.lowerings == 0
    }
}

impl Default for Expander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expander_creation() {
        let expander = Expander::new();
        assert!(expander.is_empty());
    }

    #[test]
    fn test_expander_default() {
        let expander = Expander::default();
        assert!(expander.is_empty());
    }

    #[test]
    fn test_expand_empty() {
        let mut expander = Expander::new();
        let result = expander.expand(vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_source_map_access() {
        let expander = Expander::new();
        let _map = expander.source_map();
        assert!(expander.is_empty());
    }
}
