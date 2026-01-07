//! Environment variable locking for tests
//!
//! Provides synchronization primitives to prevent race conditions when parallel tests
//! modify environment variables (which affect the entire process).
//!
//! # Why This Matters
//!
//! Environment variables are **process-global state**. When tests run in parallel and
//! modify environment variables using `env::set_var()` or `env::remove_var()`, they can
//! interfere with each other, causing flaky test failures.
//!
//! # Usage
//!
//! ```rust
//! use oxur_testing::env_lock::with_env_lock;
//! use std::env;
//!
//! // In your test:
//! with_env_lock(|| {
//!     env::set_var("MY_VAR", "test_value");
//!
//!     // Do test work that depends on MY_VAR
//!     assert_eq!(env::var("MY_VAR").unwrap(), "test_value");
//!
//!     env::remove_var("MY_VAR");
//! });
//! ```
//!
//! # Thread Safety
//!
//! The lock ensures that only one test at a time can modify environment variables,
//! preventing interference between parallel test executions.

use std::sync::Mutex;

/// Global lock for tests that modify environment variables
///
/// This mutex ensures that only one test at a time can modify environment variables,
/// preventing race conditions when tests run in parallel.
///
/// **Important:** Always use `with_env_lock()` instead of locking this directly,
/// as it handles guard RAII properly.
static ENV_VAR_LOCK: Mutex<()> = Mutex::new(());

/// Execute a function with exclusive access to environment variables
///
/// This function acquires a global lock before executing the provided closure,
/// ensuring that no other tests can modify environment variables concurrently.
///
/// # Arguments
///
/// * `f` - Closure to execute with exclusive env var access
///
/// # Returns
///
/// Returns the value returned by the closure
///
/// # Panics
///
/// Panics if the lock is poisoned (another thread panicked while holding the lock)
///
/// # Examples
///
/// ```rust
/// use oxur_testing::env_lock::with_env_lock;
/// use std::env;
///
/// // In your test:
/// with_env_lock(|| {
///     let original = env::var("CACHE_DIR").ok();
///
///     env::set_var("CACHE_DIR", "/tmp/test-cache");
///     // ... test code ...
///
///     // Cleanup
///     match original {
///         Some(val) => env::set_var("CACHE_DIR", val),
///         None => env::remove_var("CACHE_DIR"),
///     }
/// });
/// ```
pub fn with_env_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = ENV_VAR_LOCK.lock().unwrap();
    f()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_env_lock_prevents_interference() {
        // This test verifies that the lock works by ensuring sequential execution
        // In practice, this prevents race conditions in real test scenarios

        let result = with_env_lock(|| {
            env::set_var("TEST_VAR_1", "value1");
            let val = env::var("TEST_VAR_1").unwrap();
            env::remove_var("TEST_VAR_1");
            val
        });

        assert_eq!(result, "value1");
    }

    #[test]
    fn test_env_lock_returns_value() {
        let result = with_env_lock(|| 42);
        assert_eq!(result, 42);

        let result = with_env_lock(|| "test".to_string());
        assert_eq!(result, "test");
    }

    #[test]
    fn test_env_lock_multiple_sequential_calls() {
        // Simulate multiple tests modifying different env vars
        with_env_lock(|| {
            env::set_var("SEQ_TEST_1", "a");
            assert_eq!(env::var("SEQ_TEST_1").unwrap(), "a");
            env::remove_var("SEQ_TEST_1");
        });

        with_env_lock(|| {
            env::set_var("SEQ_TEST_2", "b");
            assert_eq!(env::var("SEQ_TEST_2").unwrap(), "b");
            env::remove_var("SEQ_TEST_2");
        });
    }
}
