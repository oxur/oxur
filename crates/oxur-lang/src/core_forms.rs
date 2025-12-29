//! Core Forms - The Intermediate Representation (IR)
//!
//! Core Forms are canonical S-expressions that serve as the stable contract
//! between compilation stages. After macro expansion and desugaring, all Oxur
//! code is represented in these forms.

use serde::{Deserialize, Serialize};

/// Unique identifier for AST nodes, used for source mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Core Forms - canonical S-expressions after expansion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreForm {
    // Literals
    Symbol {
        id: NodeId,
        name: String,
    },
    Number {
        id: NodeId,
        value: i64,
    },
    String {
        id: NodeId,
        value: String,
    },

    // Compound forms
    List {
        id: NodeId,
        elements: Vec<CoreForm>,
    },

    // Core language constructs (to be expanded)
    DefineFunc {
        id: NodeId,
        name: String,
        params: Vec<String>,
        body: Box<CoreForm>,
    },

    IfExpr {
        id: NodeId,
        condition: Box<CoreForm>,
        then_branch: Box<CoreForm>,
        else_branch: Option<Box<CoreForm>>,
    },

    MatchExpr {
        id: NodeId,
        scrutinee: Box<CoreForm>,
        arms: Vec<MatchArm>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: CoreForm,
    pub body: CoreForm,
}

impl CoreForm {
    pub fn node_id(&self) -> NodeId {
        match self {
            CoreForm::Symbol { id, .. } => *id,
            CoreForm::Number { id, .. } => *id,
            CoreForm::String { id, .. } => *id,
            CoreForm::List { id, .. } => *id,
            CoreForm::DefineFunc { id, .. } => *id,
            CoreForm::IfExpr { id, .. } => *id,
            CoreForm::MatchExpr { id, .. } => *id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_core_form_node_id() {
        let form = CoreForm::Number {
            id: NodeId::new(1),
            value: 42,
        };
        assert_eq!(form.node_id().0, 1);
    }
}
