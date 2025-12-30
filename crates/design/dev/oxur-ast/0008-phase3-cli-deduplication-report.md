# CLI Code Reuse and Deduplication Report

**Date:** 2025-12-29
**Phase:** Phase 3 Complete
**Comparing:** `aster` (oxur-ast CLI) vs `oxd` (design docs CLI)

---

## Executive Summary

This report analyzes code patterns shared between two Oxur CLI tools to identify opportunities for deduplication and the creation of a shared `oxur-cli-common` crate.

**Key Findings:**

- Both CLIs share identical structural patterns for organization, error handling, and command dispatch
- Colored terminal output is handled identically in both tools
- Command-line parsing follows the same clap-based patterns
- File I/O patterns are similar but specialized to each domain
- **Recommendation:** Extract common patterns into `oxur-cli-common` crate

---

## CLI Tools Analyzed

### 1. `aster` - AST Manipulation CLI

**Location:** `crates/oxur-ast/`
**Purpose:** Rust AST ↔ S-expression conversion and verification
**Commands:** `to-ast`, `to-rust`, `verify`

### 2. `oxd` - Design Documentation Manager

**Location:** `crates/design/`
**Purpose:** Manage design documents with state tracking
**Commands:** `list`, `show`, `new`, `validate`, `transition`, `add`, `scan`, `debug`, etc.

---

## Shared Patterns Identified

### 1. **Main Entry Point Structure**

Both tools use identical organizational patterns:

**Pattern:**

```rust
// Identical structure in both
use anyhow::Result;
use clap::Parser;
use colored::*;

mod cli;
mod commands;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = execute_command(...) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}
```

**Locations:**

- `aster`: `crates/oxur-ast/src/main.rs` (lines 1-21)
- `oxd`: `crates/design/src/main.rs` (lines 1-60)

**Duplication Level:** HIGH
**Extraction Candidate:** ✅ Yes

---

### 2. **CLI Module Organization**

Both tools use identical patterns for separating CLI definitions:

**Pattern:**

```rust
//! CLI argument parsing

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "...")]
#[command(about = "...", long_about = None)]
#[command(after_help = "Use '... <command> --help' for more information.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // Command definitions...
}
```

**Locations:**

- `aster`: `crates/oxur-ast/src/cli.rs` (lines 1-95)
- `oxd`: `crates/design/src/cli.rs` (lines 1-200+)

**Duplication Level:** HIGH
**Extraction Candidate:** ✅ Yes (as a pattern/trait)

---

### 3. **Error Handling with Colored Output**

Both tools use identical error formatting:

**Pattern:**

```rust
use colored::*;

// Simple error display
eprintln!("{} {}", "Error:".red().bold(), e);

// Success indicators
println!("{} ...", "✓".green().bold());

// Info messages
println!("{} ...", "→".cyan());
```

**Locations:**

- `aster`: `src/main.rs` (line 16), `src/commands/verify.rs` (lines 534, 545, 555, 565, 577, 581)
- `oxd`: `src/main.rs` (lines 25-50), `src/errors.rs` (custom error module)

**Duplication Level:** HIGH
**Extraction Candidate:** ✅ Yes

---

### 4. **Command Dispatch Pattern**

Both tools use match-based command dispatch:

**Pattern:**

```rust
fn execute_command(command: Commands, ...) -> Result<()> {
    match command {
        Commands::Cmd1 { args... } => command_module::cmd1(args...),
        Commands::Cmd2 { args... } => command_module::cmd2(args...),
        // ...
    }
}
```

**Locations:**

- `aster`: `src/main.rs` (lines 23-29)
- `oxd`: `src/main.rs` (lines 117-170)

**Duplication Level:** MEDIUM
**Extraction Candidate:** ⚠️ Pattern only (too specialized)

---

### 5. **File I/O Patterns**

Both tools handle stdin/stdout and file paths similarly:

**Pattern:**

```rust
// Read from stdin or file
let content = if input.to_str() == Some("-") {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    buffer
} else {
    fs::read_to_string(&input)?
};

// Write to stdout or file
if let Some(output_path) = output {
    if output_path.to_str() == Some("-") {
        println!("{}", output_text);
    } else {
        fs::write(output_path, output_text)?;
    }
} else {
    println!("{}", output_text);
}
```

**Locations:**

- `aster`: `src/commands/to_ast.rs` (lines 422-453), `src/commands/to_rust.rs` (lines 474-502)
- `oxd`: Various command modules (file reading patterns)

**Duplication Level:** HIGH
**Extraction Candidate:** ✅ Yes

---

### 6. **Progress/Status Indicators**

Both tools use step-by-step progress output:

**Pattern:**

```rust
println!("1. Doing step one...");
println!("   {} Done", "✓".green());

println!("2. Doing step two...");
println!("   {} Done", "✓".green());

println!("\n{} All complete!", "✓".green().bold());
```

**Locations:**

- `aster`: `src/commands/verify.rs` (lines 528-584), `examples/convert_file.rs` (lines 26-42)
- `oxd`: `src/commands/scan.rs`, `src/commands/validate.rs`

**Duplication Level:** MEDIUM
**Extraction Candidate:** ✅ Yes

---

### 7. **Anyhow Error Type Usage**

Both CLIs use identical error handling strategy:

**Pattern:**

```rust
use anyhow::Result;

pub fn command(...) -> Result<()> {
    // Operations that may fail
    some_operation()?;

    Ok(())
}
```

**Locations:**

- Used throughout both codebases
- `aster`: All command modules
- `oxd`: All command modules

**Duplication Level:** HIGH (pattern consistency)
**Extraction Candidate:** ⚠️ Standard practice, but guidelines could be shared

---

## Patterns NOT Currently Shared

### 1. Table Rendering

**oxd only:** Uses custom colored ASCII table rendering
**Location:** `crates/design/src/commands/list.rs`, integration with `oxur-table` crate
**Potential:** `aster` may need table output in the future

### 2. State Management

**oxd only:** Complex state tracking with checksums and git integration
**Location:** `crates/design/src/state/`
**Potential:** Not applicable to `aster`

### 3. Interactive Prompts

**oxd only:** Interactive mode for document addition
**Location:** `crates/design/src/commands/add.rs`
**Potential:** Could be useful for future `aster` features

---

## Recommendations

### Immediate Actions

1. **Create `oxur-cli-common` crate** with the following modules:

```
oxur-cli-common/
├── src/
│   ├── lib.rs
│   ├── io.rs          // File I/O helpers (stdin/stdout/file handling)
│   ├── output.rs      // Colored output utilities
│   ├── progress.rs    // Step-by-step progress indicators
│   └── errors.rs      // Error formatting helpers
└── Cargo.toml
```

1. **Extract High-Priority Patterns:**
   - ✅ **io.rs**: `read_input()`, `write_output()` functions
   - ✅ **output.rs**: Colored helpers (success, error, info, etc.)
   - ✅ **progress.rs**: Step tracking with visual feedback
   - ✅ **errors.rs**: Standardized error formatting

2. **Document CLI Pattern Guidelines:**
   - Standard main.rs structure
   - CLI/command separation pattern
   - Error handling conventions
   - Progress indicator usage

---

## Detailed Extraction Proposals

### Proposal 1: I/O Utilities Module

**File:** `oxur-cli-common/src/io.rs`

```rust
use anyhow::Result;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

/// Read from stdin (if path is "-") or from file
pub fn read_input(path: &PathBuf) -> Result<String> {
    if path.to_str() == Some("-") {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

/// Write to stdout (if path is None or "-") or to file
pub fn write_output(content: &str, path: Option<&PathBuf>) -> Result<()> {
    match path {
        Some(p) if p.to_str() == Some("-") => {
            println!("{}", content);
            Ok(())
        }
        Some(p) => {
            fs::write(p, content)?;
            Ok(())
        }
        None => {
            println!("{}", content);
            Ok(())
        }
    }
}
```

**Impact:**

- `aster`: Replace 30+ lines across `to_ast.rs`, `to_rust.rs`
- `oxd`: Could replace similar patterns in multiple command modules
- **Est. LOC Reduction:** ~60-80 lines across both CLIs

---

### Proposal 2: Colored Output Module

**File:** `oxur-cli-common/src/output.rs`

```rust
use colored::*;

/// Print success message with checkmark
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print error message
pub fn error(msg: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg);
}

/// Print info message
pub fn info(msg: &str) {
    println!("{} {}", "→".cyan(), msg);
}

/// Print step with number
pub fn step(num: usize, msg: &str) {
    println!("{}. {}...", num, msg);
}

/// Print step completion
pub fn step_done() {
    println!("   {} Done", "✓".green());
}
```

**Impact:**

- `aster`: Replace ~15 lines in `verify.rs`, `convert_file.rs`
- `oxd`: Could standardize output across all commands
- **Est. LOC Reduction:** ~40-60 lines across both CLIs
- **Additional Benefit:** Consistent UX across all Oxur tools

---

### Proposal 3: Progress Tracker

**File:** `oxur-cli-common/src/progress.rs`

```rust
use colored::*;

pub struct ProgressTracker {
    current_step: usize,
    verbose: bool,
}

impl ProgressTracker {
    pub fn new(verbose: bool) -> Self {
        Self { current_step: 0, verbose }
    }

    pub fn step(&mut self, msg: &str) {
        if self.verbose {
            self.current_step += 1;
            println!("{}. {}...", self.current_step, msg);
        }
    }

    pub fn done(&self) {
        if self.verbose {
            println!("   {} Done", "✓".green());
        }
    }

    pub fn success(&self, msg: &str) {
        if self.verbose {
            println!("\n{} {}", "✓".green().bold(), msg);
        } else {
            println!("{} {}", "✓".green().bold(), msg);
        }
    }
}
```

**Impact:**

- `aster`: Simplify `verify.rs` verbose mode
- `oxd`: Could enhance progress feedback in long-running commands
- **Est. LOC Reduction:** ~25-35 lines

---

## Migration Strategy

### Phase 1: Create Foundation (Week 1)

1. Create `oxur-cli-common` crate skeleton
2. Add to workspace Cargo.toml
3. Implement `io.rs` module with comprehensive tests
4. Implement `output.rs` module

### Phase 2: Migrate `aster` (Week 2)

1. Add dependency on `oxur-cli-common`
2. Replace I/O patterns in `to_ast.rs`, `to_rust.rs`
3. Replace output patterns in `verify.rs`
4. Update examples to use common utilities
5. Run full test suite to verify

### Phase 3: Migrate `oxd` (Week 3)

1. Add dependency on `oxur-cli-common`
2. Identify command modules to update
3. Incremental migration (one command at a time)
4. Maintain backward compatibility
5. Full regression testing

### Phase 4: Documentation & Guidelines (Week 4)

1. Write `oxur-cli-common` usage guide
2. Document CLI development patterns
3. Create example "hello world" CLI using common utilities
4. Update contributor guidelines

---

## Metrics & Impact

### Current Duplication

| Pattern | LOC in `aster` | LOC in `oxd` | Total Duplicated |
|---------|---------------|-------------|------------------|
| Main structure | 21 | 60 | ~40 (pattern) |
| I/O utilities | 30 | 40 | 70 |
| Colored output | 15 | 25 | 40 |
| Progress indicators | 10 | 15 | 25 |
| **Total** | **76** | **140** | **~175 LOC** |

### Post-Extraction Benefits

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Total LOC (both CLIs) | 216 | ~100 | -54% |
| Shared common code | 0 | ~120 LOC | N/A |
| Maintenance burden | 2x (duplicate fixes) | 1x | -50% |
| UX consistency | Variable | Standardized | ✅ |
| Testing burden | 2x (duplicate tests) | 1x + common tests | -40% |

### Long-Term Value

- **Future CLIs:** Any new Oxur CLI can bootstrap with `oxur-cli-common`
- **Consistency:** All Oxur tools have unified UX (colors, progress, errors)
- **Maintenance:** Bug fixes and improvements benefit all tools simultaneously
- **Testing:** Shared utilities have comprehensive test coverage

---

## Future Considerations

### Potential Additions to `oxur-cli-common`

1. **Configuration File Handling**
   - Standard TOML/YAML config loading
   - Env variable integration
   - Config validation

2. **Interactive Prompts**
   - Y/N confirmations
   - Select from list
   - Text input with validation

3. **Logging Framework**
   - Structured logging
   - Verbosity levels
   - Log file output

4. **Update Checking**
   - Check for new versions
   - Self-update capabilities (optional)

5. **Shell Completion Generation**
   - Bash/Zsh/Fish completions
   - Standardized generation

---

## Conclusion

The analysis reveals significant opportunities for code reuse between `aster` and `oxd` CLIs. Creating an `oxur-cli-common` crate will:

✅ **Reduce code duplication** by ~175 LOC initially
✅ **Standardize UX** across all Oxur tools
✅ **Simplify maintenance** with centralized utilities
✅ **Accelerate development** of future CLIs
✅ **Improve testing** with shared test coverage

**Recommendation:** Proceed with `oxur-cli-common` extraction following the phased migration strategy outlined above.

---

**Next Steps:**

1. Create GitHub issue for `oxur-cli-common` crate
2. Design detailed API for initial modules (io, output, progress)
3. Write comprehensive tests before extraction
4. Begin Phase 1 implementation

---

*Report compiled: 2025-12-29*
*Codebase versions: `aster` (Phase 3 complete), `oxd` (current main branch)*
