//! Type-erased variable storage for subprocess runtime
//!
//! Provides type-erased storage for REPL variables that persists across
//! evaluations within a subprocess session. Based on the evcxr pattern.
//!
//! Based on ODD-0026 Section 3.1 (VariableStore Pattern)

use std::any::Any;
use std::collections::HashMap;
use std::sync::Mutex;

/// Type-erased variable store
///
/// Stores variables with type erasure using `Box<dyn Any + 'static>`.
/// Variables are owned by the store and cannot reference each other
/// (which aligns with Lisp's immutable data structure semantics).
///
/// # Thread Safety
///
/// The store is protected by a Mutex and can be accessed from multiple
/// threads safely. In practice, it's accessed via a global static in
/// the subprocess binary.
///
/// # Constraints (ODD-0038 Decision 7)
///
/// - `Box<dyn Any + 'static>` requires owned data
/// - No inter-variable references possible
/// - Aligns with Lisp semantics (immutable data structures)
///
/// # Examples
///
/// ```
/// use oxur_repl::subprocess::VariableStore;
///
/// let mut store = VariableStore::new();
///
/// // Store a value
/// store.set("x".to_string(), 42i32);
///
/// // Retrieve with correct type
/// let x: Option<&i32> = store.get("x");
/// assert_eq!(x, Some(&42));
///
/// // Wrong type returns None
/// let x: Option<&String> = store.get("x");
/// assert_eq!(x, None);
/// ```
#[derive(Debug, Default)]
pub struct VariableStore {
    /// Map from variable name to type-erased value
    variables: HashMap<String, Box<dyn Any + Send>>,
}

impl VariableStore {
    /// Create a new empty variable store
    pub fn new() -> Self {
        Self { variables: HashMap::new() }
    }

    /// Store a variable with type erasure
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    /// * `value` - Value to store (must be `'static` and `Send`)
    ///
    /// # Type Parameters
    ///
    /// * `T` - Type of value (must be `'static` and `Send`)
    pub fn set<T: 'static + Send>(&mut self, name: String, value: T) {
        self.variables.insert(name, Box::new(value));
    }

    /// Get a variable with type checking
    ///
    /// Returns `Some(&T)` if the variable exists and has the correct type,
    /// `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    ///
    /// # Type Parameters
    ///
    /// * `T` - Expected type of variable
    pub fn get<T: 'static>(&self, name: &str) -> Option<&T> {
        self.variables.get(name).and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable reference to a variable
    ///
    /// Returns `Some(&mut T)` if the variable exists and has the correct type,
    /// `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    ///
    /// # Type Parameters
    ///
    /// * `T` - Expected type of variable
    pub fn get_mut<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.variables.get_mut(name).and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Check if a variable exists
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Remove a variable
    ///
    /// Returns `true` if the variable existed and was removed
    pub fn remove(&mut self, name: &str) -> bool {
        self.variables.remove(name).is_some()
    }

    /// Get number of stored variables
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// List all variable names
    pub fn names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }
}

/// Thread-safe global variable store
///
/// Used in the subprocess binary to provide a global storage location
/// for REPL variables. Protected by a Mutex for thread safety.
#[allow(dead_code)] // Used in Phase 2 subprocess implementation
static GLOBAL_STORE: Mutex<Option<VariableStore>> = Mutex::new(None);

/// Global result value storage
///
/// Stores the result of the last expression evaluation for retrieval
/// by the subprocess IPC protocol.
static GLOBAL_RESULT: Mutex<Option<String>> = Mutex::new(None);

/// Initialize the global variable store
///
/// Should be called once when the subprocess starts.
#[allow(dead_code)] // Used in Phase 2 subprocess implementation
pub fn init_global_store() {
    let mut store = GLOBAL_STORE.lock().unwrap();
    *store = Some(VariableStore::new());
}

/// Access the global variable store
///
/// Provides access to the global store via a closure. The store is
/// locked while the closure executes.
///
/// # Panics
///
/// Panics if the global store has not been initialized.
///
/// # Examples
///
/// ```
/// use oxur_repl::subprocess::{init_global_store, with_store};
///
/// init_global_store();
///
/// // Store a value
/// with_store(|store| {
///     store.set("x".to_string(), 42i32);
/// });
///
/// // Retrieve a value
/// let result = with_store(|store| {
///     store.get::<i32>("x").copied()
/// });
///
/// assert_eq!(result, Some(42));
/// ```
#[allow(dead_code)] // Used in Phase 2 subprocess implementation
pub fn with_store<F, R>(f: F) -> R
where
    F: FnOnce(&mut VariableStore) -> R,
{
    let mut guard = GLOBAL_STORE.lock().unwrap();
    let store =
        guard.as_mut().expect("Global store not initialized - call init_global_store() first");
    f(store)
}

/// Store the result of the last expression evaluation
///
/// Called by generated wrapper code to store the result value.
/// The result is formatted as a Debug string for transmission.
///
/// # Examples
///
/// ```
/// use oxur_repl::subprocess::set_result;
///
/// // Typically called by generated code:
/// let value = 42;
/// set_result(format!("{:?}", value));
/// ```
pub fn set_result(value: String) {
    let mut result = GLOBAL_RESULT.lock().unwrap();
    *result = Some(value);
}

/// Retrieve and clear the result of the last expression evaluation
///
/// Called by the subprocess IPC protocol after executing user code.
/// Returns the result value and clears the global storage.
///
/// # Returns
///
/// The stored result string, or None if no result was stored.
pub fn take_result() -> Option<String> {
    let mut result = GLOBAL_RESULT.lock().unwrap();
    result.take()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_variable_store_basic() {
        let mut store = VariableStore::new();

        store.set("x".to_string(), 42i32);
        assert_eq!(store.get::<i32>("x"), Some(&42));
    }

    #[test]
    fn test_variable_store_type_mismatch() {
        let mut store = VariableStore::new();

        store.set("x".to_string(), 42i32);

        // Wrong type returns None
        assert_eq!(store.get::<String>("x"), None);
        assert_eq!(store.get::<f64>("x"), None);
    }

    #[test]
    fn test_variable_store_multiple_types() {
        let mut store = VariableStore::new();

        store.set("int_val".to_string(), 42i32);
        store.set("str_val".to_string(), "hello".to_string());
        store.set("bool_val".to_string(), true);

        assert_eq!(store.get::<i32>("int_val"), Some(&42));
        assert_eq!(store.get::<String>("str_val"), Some(&"hello".to_string()));
        assert_eq!(store.get::<bool>("bool_val"), Some(&true));
    }

    #[test]
    fn test_variable_store_overwrite() {
        let mut store = VariableStore::new();

        store.set("x".to_string(), 42i32);
        assert_eq!(store.get::<i32>("x"), Some(&42));

        store.set("x".to_string(), 100i32);
        assert_eq!(store.get::<i32>("x"), Some(&100));
    }

    #[test]
    fn test_variable_store_overwrite_different_type() {
        let mut store = VariableStore::new();

        store.set("x".to_string(), 42i32);
        assert_eq!(store.get::<i32>("x"), Some(&42));

        // Overwrite with different type
        store.set("x".to_string(), "hello".to_string());
        assert_eq!(store.get::<String>("x"), Some(&"hello".to_string()));
        assert_eq!(store.get::<i32>("x"), None); // Old type no longer accessible
    }

    #[test]
    fn test_variable_store_get_mut() {
        let mut store = VariableStore::new();

        store.set("x".to_string(), 42i32);

        if let Some(x) = store.get_mut::<i32>("x") {
            *x += 10;
        }

        assert_eq!(store.get::<i32>("x"), Some(&52));
    }

    #[test]
    fn test_variable_store_contains() {
        let mut store = VariableStore::new();

        store.set("x".to_string(), 42i32);

        assert!(store.contains("x"));
        assert!(!store.contains("y"));
    }

    #[test]
    fn test_variable_store_remove() {
        let mut store = VariableStore::new();

        store.set("x".to_string(), 42i32);
        assert!(store.contains("x"));

        assert!(store.remove("x"));
        assert!(!store.contains("x"));
        assert!(!store.remove("x")); // Second remove returns false
    }

    #[test]
    fn test_variable_store_len() {
        let mut store = VariableStore::new();

        assert_eq!(store.len(), 0);
        assert!(store.is_empty());

        store.set("x".to_string(), 42i32);
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());

        store.set("y".to_string(), "hello".to_string());
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_variable_store_clear() {
        let mut store = VariableStore::new();

        store.set("x".to_string(), 42i32);
        store.set("y".to_string(), "hello".to_string());
        assert_eq!(store.len(), 2);

        store.clear();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_variable_store_names() {
        let mut store = VariableStore::new();

        store.set("x".to_string(), 42i32);
        store.set("y".to_string(), "hello".to_string());
        store.set("z".to_string(), true);

        let names = store.names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"x".to_string()));
        assert!(names.contains(&"y".to_string()));
        assert!(names.contains(&"z".to_string()));
    }

    #[test]
    fn test_variable_store_complex_types() {
        let mut store = VariableStore::new();

        // Store Vec
        store.set("vec".to_string(), vec![1, 2, 3]);
        assert_eq!(store.get::<Vec<i32>>("vec"), Some(&vec![1, 2, 3]));

        // Store custom struct
        #[derive(Debug, PartialEq)]
        struct Point {
            x: i32,
            y: i32,
        }

        store.set("point".to_string(), Point { x: 10, y: 20 });
        assert_eq!(store.get::<Point>("point"), Some(&Point { x: 10, y: 20 }));
    }

    #[test]
    #[serial]
    fn test_global_store() {
        init_global_store();

        // Store a value
        with_store(|store| {
            store.set("global_x".to_string(), 42i32);
        });

        // Retrieve the value
        let result = with_store(|store| store.get::<i32>("global_x").copied());

        assert_eq!(result, Some(42));

        // Clear for next test
        with_store(|store| store.clear());
    }

    #[test]
    #[serial]
    fn test_global_store_multiple_accesses() {
        init_global_store();

        with_store(|store| store.clear());

        // Multiple accesses work correctly
        with_store(|store| {
            store.set("a".to_string(), 1i32);
        });

        with_store(|store| {
            store.set("b".to_string(), 2i32);
        });

        let sum = with_store(|store| {
            let a = store.get::<i32>("a").copied().unwrap_or(0);
            let b = store.get::<i32>("b").copied().unwrap_or(0);
            a + b
        });

        assert_eq!(sum, 3);
    }
}
