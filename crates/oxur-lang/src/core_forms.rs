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
        let form = CoreForm::Number { id: NodeId::new(1), value: 42 };
        assert_eq!(form.node_id().0, 1);
    }

    #[test]
    fn test_node_id_display() {
        let id = NodeId::new(123);
        assert_eq!(format!("{}", id), "#123");
    }

    #[test]
    fn test_node_id_equality() {
        let id1 = NodeId::new(42);
        let id2 = NodeId::new(42);
        let id3 = NodeId::new(43);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_symbol_node_id() {
        let form = CoreForm::Symbol { id: NodeId::new(10), name: "foo".to_string() };
        assert_eq!(form.node_id().0, 10);
    }

    #[test]
    fn test_string_node_id() {
        let form = CoreForm::String { id: NodeId::new(20), value: "hello".to_string() };
        assert_eq!(form.node_id().0, 20);
    }

    #[test]
    fn test_list_node_id() {
        let form = CoreForm::List { id: NodeId::new(30), elements: vec![] };
        assert_eq!(form.node_id().0, 30);
    }

    #[test]
    fn test_define_func_node_id() {
        let form = CoreForm::DefineFunc {
            id: NodeId::new(40),
            name: "test".to_string(),
            params: vec![],
            body: Box::new(CoreForm::Number { id: NodeId::new(41), value: 0 }),
        };
        assert_eq!(form.node_id().0, 40);
    }

    #[test]
    fn test_if_expr_node_id() {
        let form = CoreForm::IfExpr {
            id: NodeId::new(50),
            condition: Box::new(CoreForm::Symbol { id: NodeId::new(51), name: "true".to_string() }),
            then_branch: Box::new(CoreForm::Number { id: NodeId::new(52), value: 1 }),
            else_branch: None,
        };
        assert_eq!(form.node_id().0, 50);
    }

    #[test]
    fn test_match_expr_node_id() {
        let form = CoreForm::MatchExpr {
            id: NodeId::new(60),
            scrutinee: Box::new(CoreForm::Symbol { id: NodeId::new(61), name: "x".to_string() }),
            arms: vec![],
        };
        assert_eq!(form.node_id().0, 60);
    }

    #[test]
    fn test_match_arm_creation() {
        let arm = MatchArm {
            pattern: CoreForm::Symbol { id: NodeId::new(70), name: "pattern".to_string() },
            body: CoreForm::Number { id: NodeId::new(71), value: 42 },
        };
        assert_eq!(arm.pattern.node_id().0, 70);
        assert_eq!(arm.body.node_id().0, 71);
    }
}
