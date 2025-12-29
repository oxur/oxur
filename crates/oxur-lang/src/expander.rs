//! Stage 2: Expand
//!
//! Converts Surface Forms into Core Forms through macro expansion and desugaring.
//! This is where syntactic sugar gets transformed into canonical forms.

use crate::{core_forms::CoreForm, source_map::SourceMap, Result};
use crate::parser::SurfaceForm;

/// Expander handles macro expansion and desugaring
pub struct Expander {
    source_map: SourceMap,
}

impl Expander {
    pub fn new() -> Self {
        Self {
            source_map: SourceMap::new(),
        }
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
        assert!(expander.source_map().is_empty());
    }
}
