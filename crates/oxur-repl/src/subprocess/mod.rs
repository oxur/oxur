//! Subprocess runtime support
//!
//! Type-erased variable storage and runtime utilities for
//! code executing in the subprocess.

mod variable_store;

pub use variable_store::{init_global_store, with_store, VariableStore};
