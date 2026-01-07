//! Oxur REPL Subprocess Binary
//!
//! This binary is spawned by the REPL server to execute user code in isolation.
//! It provides:
//! - Dynamic library loading via libloading
//! - Function execution with panic catching
//! - Output redirection
//! - IPC protocol over stdin/stdout
//!
//! Based on ODD-0026 Section 3.3 (Subprocess Runtime Binary)
//!
//! # Protocol
//!
//! Commands (stdin):
//! - `LOAD <path>` - Load a dynamic library
//! - `RUN <cache_key>` - Execute function from loaded library
//!
//! Responses (stdout):
//! - `LOADED` - Library loaded successfully
//! - `LOAD_ERROR <msg>` - Library loading failed
//! - `OXUR_EXECUTION_COMPLETE` - Execution finished successfully
//! - `OXUR_RUNTIME_ERROR <msg>` - Runtime error occurred
//! - `OXUR_PANIC_LOCATION <location>` - Panic occurred
//! - `OXUR_PANIC_MESSAGE <msg>` - Panic message
//!
//! # Why Subprocess? (ODD-0038 Decision 3)
//!
//! - Rust threads cannot be interrupted (no pthread_cancel)
//! - Ctrl-C support requires process isolation
//! - Crashes in user code don't corrupt REPL state
//! - Variable isolation and cleanup on crash

use libloading::{Library, Symbol};
use oxur_repl::subprocess::{init_global_store, set_result, take_result};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::panic;
use std::path::PathBuf;

/// Type signature for generated REPL functions
///
/// All generated functions follow this signature:
/// - No arguments (variables accessed via global store)
/// - No return value (results stored in global static)
type ReplFunction = unsafe extern "C" fn();

/// Type signatures for result retrieval functions exported by dynamic libraries
type GetResultFn = unsafe extern "C" fn() -> *const u8;
type GetResultLenFn = unsafe extern "C" fn() -> usize;

/// Library manager for tracking loaded dynamic libraries
struct LibraryManager {
    /// Map from cache key to loaded library
    libraries: HashMap<String, Library>,
    /// Map from cache key to library path
    paths: HashMap<String, PathBuf>,
}

impl LibraryManager {
    fn new() -> Self {
        Self { libraries: HashMap::new(), paths: HashMap::new() }
    }

    /// Load a library from path and associate with cache key
    fn load(&mut self, cache_key: String, path: PathBuf) -> Result<(), String> {
        // Check if already loaded
        if self.libraries.contains_key(&cache_key) {
            return Ok(());
        }

        // Load the library
        let library = unsafe {
            Library::new(&path)
                .map_err(|e| format!("Failed to load library {}: {}", path.display(), e))?
        };

        self.paths.insert(cache_key.clone(), path);
        self.libraries.insert(cache_key, library);

        Ok(())
    }

    /// Execute a function from a loaded library
    fn execute(&self, cache_key: &str) -> Result<(), String> {
        let library = self
            .libraries
            .get(cache_key)
            .ok_or_else(|| format!("Library not loaded: {}", cache_key))?;

        // Get the symbol (function entry point)
        // Convention: function name is "oxur_eval_<cache_key>"
        let symbol_name = format!("oxur_eval_{}", cache_key);

        unsafe {
            let func: Symbol<ReplFunction> = library
                .get(symbol_name.as_bytes())
                .map_err(|e| format!("Failed to find symbol {}: {}", symbol_name, e))?;

            // Execute with panic catching
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                func();
            }));

            match result {
                Ok(()) => {
                    // Try to retrieve result from global buffer
                    if let Ok(get_result_fn) = library.get::<GetResultFn>(b"oxur_get_result") {
                        if let Ok(get_len_fn) =
                            library.get::<GetResultLenFn>(b"oxur_get_result_len")
                        {
                            let ptr = get_result_fn();
                            let len = get_len_fn();

                            if !ptr.is_null() && len > 0 {
                                let slice = std::slice::from_raw_parts(ptr, len);
                                if let Ok(s) = std::str::from_utf8(slice) {
                                    set_result(s.to_string());
                                }
                            }
                        }
                    }
                    Ok(())
                }
                Err(panic_info) => {
                    // Extract panic message
                    let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic".to_string()
                    };

                    Err(format!("Panic: {}", msg))
                }
            }
        }
    }

    /// Get count of loaded libraries
    fn loaded_count(&self) -> usize {
        self.libraries.len()
    }
}

/// Process a LOAD command
fn handle_load(manager: &mut LibraryManager, args: &str) -> Result<(), String> {
    // Parse: LOAD <cache_key> <path>
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() != 2 {
        return Err("LOAD requires <cache_key> <path>".to_string());
    }

    let cache_key = parts[0].to_string();
    let path = PathBuf::from(parts[1]);

    manager.load(cache_key, path)?;
    Ok(())
}

/// Process a RUN command
fn handle_run(manager: &LibraryManager, args: &str) -> Result<(), String> {
    // Parse: RUN <cache_key>
    let cache_key = args.trim();
    if cache_key.is_empty() {
        return Err("RUN requires <cache_key>".to_string());
    }

    manager.execute(cache_key)?;
    Ok(())
}

/// Main IPC protocol loop
fn run_protocol() {
    let mut manager = LibraryManager::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = BufReader::new(stdin);

    for line in reader.lines() {
        match line {
            Ok(cmd) => {
                let cmd = cmd.trim();

                if cmd.is_empty() {
                    continue;
                }

                if let Some(args) = cmd.strip_prefix("LOAD ") {
                    match handle_load(&mut manager, args) {
                        Ok(()) => {
                            writeln!(stdout, "LOADED").ok();
                        }
                        Err(e) => {
                            writeln!(stdout, "LOAD_ERROR {}", e).ok();
                        }
                    }
                    stdout.flush().ok();
                } else if let Some(args) = cmd.strip_prefix("RUN ") {
                    match handle_run(&manager, args) {
                        Ok(()) => {
                            // Send result value if one was captured
                            if let Some(result_value) = take_result() {
                                writeln!(stdout, "OXUR_RESULT {}", result_value).ok();
                            }
                            writeln!(stdout, "OXUR_EXECUTION_COMPLETE").ok();
                        }
                        Err(e) => {
                            if e.starts_with("Panic:") {
                                // Split panic into location and message
                                writeln!(stdout, "OXUR_PANIC_LOCATION unknown").ok();
                                writeln!(stdout, "OXUR_PANIC_MESSAGE {}", e).ok();
                            } else {
                                writeln!(stdout, "OXUR_RUNTIME_ERROR {}", e).ok();
                            }
                        }
                    }
                    stdout.flush().ok();
                } else if cmd == "PING" {
                    // Health check command
                    writeln!(stdout, "PONG").ok();
                    stdout.flush().ok();
                } else if cmd == "STATS" {
                    // Debug command to get statistics
                    writeln!(stdout, "LIBRARIES_LOADED {}", manager.loaded_count()).ok();
                    stdout.flush().ok();
                } else {
                    eprintln!("Unknown command: {}", cmd);
                    writeln!(stdout, "ERROR Unknown command").ok();
                    stdout.flush().ok();
                }
            }
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                break;
            }
        }
    }
}

fn main() {
    // Initialize global variable store
    init_global_store();

    // Set up panic hook for better error reporting
    panic::set_hook(Box::new(|panic_info| {
        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown".to_string()
        };

        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<Any>".to_string()
        };

        eprintln!("PANIC at {}: {}", location, message);
    }));

    // Run the IPC protocol loop
    run_protocol();
}
