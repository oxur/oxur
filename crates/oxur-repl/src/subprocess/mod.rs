//! Subprocess runtime support
//!
//! Type-erased variable storage and runtime utilities for
//! code executing in the subprocess.

mod variable_store;

pub use variable_store::{init_global_store, set_result, take_result, with_store, VariableStore};
