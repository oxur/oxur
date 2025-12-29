# Oxur Crates Setup Plan for Claude Code

## Overview

This plan sets up four Rust crates for the Oxur compilation toolchain, following the architecture defined in the compilation chain architecture document. Each crate will be a working placeholder that compiles successfully and demonstrates the intended structure.

## Project Structure

```
oxur/
├── Cargo.toml                 # Workspace manifest
├── crates/
│   ├── oxur-comp/            # Compiler library and binary
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs        # Library root
│   │   │   └── main.rs       # Binary (oxurc)
│   │   └── README.md
│   ├── oxur-lang/            # Language processing (parser, expander)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs        # Library root
│   │   │   ├── parser.rs     # Stage 1: Parse
│   │   │   ├── expander.rs   # Stage 2: Expand
│   │   │   ├── core_forms.rs # Core Forms IR definitions
│   │   │   └── source_map.rs # Source mapping
│   │   └── README.md
│   ├── oxur-repl/            # REPL protocol, client, and server
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs        # Library root
│   │   │   ├── protocol.rs   # REPL protocol definitions
│   │   │   ├── client.rs     # REPL client
│   │   │   └── server.rs     # REPL server with tiered execution
│   │   └── README.md
│   └── oxur-cli/             # CLI binary
│       ├── Cargo.toml
│       ├── src/
│       │   └── main.rs       # Binary (oxur)
│       └── README.md
└── README.md                  # Workspace overview
```

## Implementation Tasks

### Task 1: Create Workspace Root

**File**: `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "crates/oxur-comp",
    "crates/oxur-lang",
    "crates/oxur-repl",
    "crates/oxur-cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
authors = ["Duncan McGreggor <duncan@oxur.io>"]
repository = "https://github.com/oxur/oxur"

[workspace.dependencies]
# Common dependencies across workspace
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.5", features = ["derive", "cargo"] }
syn = { version = "2.0", features = ["full", "parsing", "printing", "visit"] }
quote = "1.0"
proc-macro2 = "1.0"
prettyplease = "0.2"
```

**File**: `README.md`

```markdown
# Oxur

A Lisp dialect that compiles to Rust with 100% interoperability.

## Architecture

Oxur follows a multi-stage compilation pipeline:

1. **Parse** - Oxur syntax → Surface Forms
2. **Expand** - Surface Forms → Core Forms (IR)
3. **Lower** - Core Forms → Rust AST
4. **Generate** - Rust AST → Rust source
5. **Compile** - Rust source → Binary (via rustc)

## Crates

- **oxur-comp** - Compiler library and binary (`oxurc`)
- **oxur-lang** - Language processing (parsing, expansion, Core Forms)
- **oxur-repl** - REPL protocol, client, and server
- **oxur-cli** - Main CLI tool (`oxur`)

## Quick Start

```bash
# Build all crates
cargo build --workspace

# Run the compiler
cargo run --bin oxurc -- --help

# Run the CLI tool
cargo run --bin oxur -- --help

# Run tests
cargo test --workspace
```

## Status

v0.1.0 - Initial placeholder implementation with complete compilation architecture.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
```

---

### Task 2: Create oxur-lang Crate

**File**: `crates/oxur-lang/Cargo.toml`

```toml
[package]
name = "oxur-lang"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Oxur language processing: parser, expander, and Core Forms IR"

[dependencies]
thiserror.workspace = true
serde.workspace = true
syn.workspace = true

[dev-dependencies]
anyhow.workspace = true
```

**File**: `crates/oxur-lang/src/lib.rs`

```rust
//! Oxur Language Processing
//!
//! This crate handles the frontend of the Oxur compilation pipeline:
//! - Stage 1: Parse (Oxur syntax → Surface Forms)
//! - Stage 2: Expand (Surface Forms → Core Forms)
//!
//! Core Forms are the stable intermediate representation (IR) that serves
//! as the contract between the frontend (oxur-lang) and backend (oxur-comp).

pub mod parser;
pub mod expander;
pub mod core_forms;
pub mod source_map;

pub use core_forms::{CoreForm, NodeId};
pub use parser::Parser;
pub use expander::Expander;
pub use source_map::SourceMap;

/// Result type for language operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for language processing
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse error at {location}: {message}")]
    Parse {
        message: String,
        location: Location,
    },
    
    #[error("Expansion error at node {node_id}: {message}")]
    Expand {
        message: String,
        node_id: NodeId,
    },
    
    #[error("Invalid syntax: {0}")]
    Syntax(String),
}

/// Source location for error reporting
#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
```

**File**: `crates/oxur-lang/src/parser.rs`

```rust
//! Stage 1: Parse
//!
//! Converts raw Oxur source text into Surface Forms (S-expression AST).
//! Handles tokenization, reader, and reader macros.

use crate::{Error, Location, Result};

/// Parser converts Oxur source text into Surface Forms
pub struct Parser {
    source: String,
    position: usize,
}

impl Parser {
    pub fn new(source: String) -> Self {
        Self {
            source,
            position: 0,
        }
    }
    
    /// Parse the source into Surface Forms
    pub fn parse(&mut self) -> Result<Vec<SurfaceForm>> {
        // Placeholder implementation
        Ok(vec![])
    }
}

/// Surface Forms - parsed S-expressions before macro expansion
#[derive(Debug, Clone)]
pub enum SurfaceForm {
    Symbol(String),
    Number(i64),
    String(String),
    List(Vec<SurfaceForm>),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_creation() {
        let parser = Parser::new("(+ 1 2)".to_string());
        assert_eq!(parser.position, 0);
    }
}
```

**File**: `crates/oxur-lang/src/expander.rs`

```rust
//! Stage 2: Expand
//!
//! Converts Surface Forms into Core Forms through macro expansion and desugaring.
//! This is where syntactic sugar gets transformed into canonical forms.

use crate::{core_forms::CoreForm, source_map::SourceMap, Error, Result};
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
    pub fn expand(&mut self, forms: Vec<SurfaceForm>) -> Result<Vec<CoreForm>> {
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
```

**File**: `crates/oxur-lang/src/core_forms.rs`

```rust
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
```

**File**: `crates/oxur-lang/src/source_map.rs`

```rust
//! Source Map
//!
//! Tracks the transformation of code through all compilation stages.
//! Essential for accurate error reporting.

use crate::core_forms::NodeId;
use crate::Location;
use std::collections::HashMap;

/// Source map tracks transformations through compilation
#[derive(Debug, Clone)]
pub struct SourceMap {
    mappings: HashMap<NodeId, SourceInfo>,
}

/// Information about a node's origin
#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub location: Location,
    pub original_text: String,
    pub parent: Option<NodeId>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }
    
    pub fn add(&mut self, node_id: NodeId, info: SourceInfo) {
        self.mappings.insert(node_id, info);
    }
    
    pub fn get(&self, node_id: NodeId) -> Option<&SourceInfo> {
        self.mappings.get(&node_id)
    }
    
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_source_map() {
        let mut map = SourceMap::new();
        assert!(map.is_empty());
        
        let node_id = NodeId::new(1);
        let info = SourceInfo {
            location: Location { line: 1, column: 5 },
            original_text: "(+ 1 2)".to_string(),
            parent: None,
        };
        
        map.add(node_id, info);
        assert!(!map.is_empty());
        assert!(map.get(node_id).is_some());
    }
}
```

**File**: `crates/oxur-lang/README.md`

```markdown
# oxur-lang

Language processing for Oxur: parsing, macro expansion, and Core Forms IR.

## Stages

### Stage 1: Parse
Converts raw Oxur source into Surface Forms (S-expression AST).

### Stage 2: Expand
Transforms Surface Forms into Core Forms through macro expansion and desugaring.

## Core Forms

Core Forms are the canonical intermediate representation that serves as the stable
contract between the Oxur frontend and the Rust backend. All syntactic sugar and
macros are expanded into these forms.

## Source Maps

The source map tracks every transformation from original source through to Rust AST,
enabling accurate error reporting that points back to the original Oxur code.
```

---

### Task 3: Create oxur-comp Crate

**File**: `crates/oxur-comp/Cargo.toml`

```toml
[package]
name = "oxur-comp"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Oxur compiler: lowers Core Forms to Rust and generates binaries"

[[bin]]
name = "oxurc"
path = "src/main.rs"

[dependencies]
oxur-lang = { path = "../oxur-lang" }
anyhow.workspace = true
thiserror.workspace = true
clap.workspace = true
syn.workspace = true
quote.workspace = true
prettyplease.workspace = true
serde_json.workspace = true

[dev-dependencies]
tempfile = "3.8"
```

**File**: `crates/oxur-comp/src/lib.rs`

```rust
//! Oxur Compiler
//!
//! Handles the backend of the Oxur compilation pipeline:
//! - Stage 3: Lower (Core Forms → Rust AST)
//! - Stage 4: Generate (Rust AST → Rust source)
//! - Stage 5: Compile (Rust source → Binary via rustc)

pub mod lowering;
pub mod codegen;
pub mod compiler;

pub use compiler::Compiler;
pub use lowering::Lowerer;
pub use codegen::CodeGenerator;

/// Result type for compilation operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for compilation
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lowering error: {0}")]
    Lowering(String),
    
    #[error("Code generation error: {0}")]
    CodeGen(String),
    
    #[error("Compilation error: {0}")]
    Compile(String),
    
    #[error("Language error: {0}")]
    Language(#[from] oxur_lang::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

**File**: `crates/oxur-comp/src/lowering.rs`

```rust
//! Stage 3: Lower
//!
//! Converts Core Forms into Rust AST using the syn crate.

use crate::{Error, Result};
use oxur_lang::{CoreForm, NodeId};
use std::collections::HashMap;

/// Lowerer converts Core Forms to Rust AST
pub struct Lowerer {
    node_map: HashMap<NodeId, syn::Expr>,
}

impl Lowerer {
    pub fn new() -> Self {
        Self {
            node_map: HashMap::new(),
        }
    }
    
    /// Lower Core Forms to Rust AST
    pub fn lower(&mut self, forms: Vec<CoreForm>) -> Result<syn::File> {
        // Placeholder implementation
        Ok(syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![],
        })
    }
}

impl Default for Lowerer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lowerer_creation() {
        let lowerer = Lowerer::new();
        assert_eq!(lowerer.node_map.len(), 0);
    }
}
```

**File**: `crates/oxur-comp/src/codegen.rs`

```rust
//! Stage 4: Generate
//!
//! Converts Rust AST into formatted Rust source code.

use crate::{Error, Result};

/// Code generator produces formatted Rust source
pub struct CodeGenerator;

impl CodeGenerator {
    pub fn new() -> Self {
        Self
    }
    
    /// Generate formatted Rust source from AST
    pub fn generate(&self, file: &syn::File) -> Result<String> {
        // Use prettyplease for formatting
        Ok(prettyplease::unparse(file))
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_codegen_empty_file() {
        let gen = CodeGenerator::new();
        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![],
        };
        let result = gen.generate(&file);
        assert!(result.is_ok());
    }
}
```

**File**: `crates/oxur-comp/src/compiler.rs`

```rust
//! Compiler
//!
//! Orchestrates the complete compilation pipeline from Core Forms to binary.

use crate::{CodeGenerator, Error, Lowerer, Result};
use oxur_lang::CoreForm;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Compiler orchestrates the full compilation pipeline
pub struct Compiler {
    lowerer: Lowerer,
    codegen: CodeGenerator,
    output_dir: PathBuf,
}

impl Compiler {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            lowerer: Lowerer::new(),
            codegen: CodeGenerator::new(),
            output_dir,
        }
    }
    
    /// Compile Core Forms to a binary
    pub fn compile(&mut self, forms: Vec<CoreForm>, output: &Path) -> Result<()> {
        // Stage 3: Lower to Rust AST
        let ast = self.lowerer.lower(forms)?;
        
        // Stage 4: Generate Rust source
        let source = self.codegen.generate(&ast)?;
        
        // Write to temporary .rs file
        let rs_file = self.output_dir.join("generated.rs");
        std::fs::write(&rs_file, source)?;
        
        // Stage 5: Compile with rustc
        self.compile_with_rustc(&rs_file, output)?;
        
        Ok(())
    }
    
    fn compile_with_rustc(&self, source: &Path, output: &Path) -> Result<()> {
        let status = Command::new("rustc")
            .arg(source)
            .arg("-o")
            .arg(output)
            .status()?;
        
        if !status.success() {
            return Err(Error::Compile(format!(
                "rustc failed with exit code: {:?}",
                status.code()
            )));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new(PathBuf::from("/tmp"));
        assert_eq!(compiler.output_dir, PathBuf::from("/tmp"));
    }
}
```

**File**: `crates/oxur-comp/src/main.rs`

```rust
//! oxurc - Oxur Compiler Binary
//!
//! Main entry point for the Oxur compiler.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oxurc")]
#[command(about = "Oxur compiler - compiles Oxur code to native binaries", long_about = None)]
struct Cli {
    /// Input Oxur source file
    #[arg(value_name = "FILE")]
    input: PathBuf,
    
    /// Output binary path
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    
    /// Output directory for intermediate files
    #[arg(long, default_value = ".oxur-build")]
    build_dir: PathBuf,
    
    /// Emit generated Rust source (don't delete)
    #[arg(long)]
    emit_rust: bool,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.verbose {
        println!("Compiling: {}", cli.input.display());
    }
    
    // Placeholder: Read input file
    let source = std::fs::read_to_string(&cli.input)?;
    
    if cli.verbose {
        println!("Source length: {} bytes", source.len());
    }
    
    // Parse
    if cli.verbose {
        println!("Stage 1: Parsing...");
    }
    let mut parser = oxur_lang::Parser::new(source);
    let surface_forms = parser.parse()?;
    
    // Expand
    if cli.verbose {
        println!("Stage 2: Expanding macros...");
    }
    let mut expander = oxur_lang::Expander::new();
    let core_forms = expander.expand(surface_forms)?;
    
    // Compile
    if cli.verbose {
        println!("Stage 3-5: Lowering, generating, and compiling...");
    }
    let output = cli.output.unwrap_or_else(|| {
        cli.input.with_extension("")
    });
    
    let mut compiler = oxur_comp::Compiler::new(cli.build_dir.clone());
    compiler.compile(core_forms, &output)?;
    
    if cli.verbose {
        println!("Successfully compiled to: {}", output.display());
    }
    
    // Clean up build directory unless --emit-rust
    if !cli.emit_rust && cli.build_dir.exists() {
        std::fs::remove_dir_all(&cli.build_dir)?;
    }
    
    Ok(())
}
```

**File**: `crates/oxur-comp/README.md`

```markdown
# oxur-comp

Oxur compiler library and binary.

## Compiler Binary (`oxurc`)

```bash
# Compile an Oxur file
oxurc input.ox -o output

# Keep generated Rust source
oxurc input.ox --emit-rust

# Verbose output
oxurc input.ox -v
```

## Library

The `oxur-comp` library provides:

- **Lowering**: Core Forms → Rust AST (Stage 3)
- **Code Generation**: Rust AST → Rust source (Stage 4)
- **Compilation**: Rust source → Binary via rustc (Stage 5)

## Architecture

The compiler takes Core Forms (from `oxur-lang`) and:

1. Lowers them to Rust AST using the `syn` crate
2. Generates formatted Rust source using `prettyplease`
3. Invokes `rustc` to produce the final binary
```

---

### Task 4: Create oxur-repl Crate

**File**: `crates/oxur-repl/Cargo.toml`

```toml
[package]
name = "oxur-repl"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Oxur REPL: protocol, client, and server with tiered execution"

[dependencies]
oxur-lang = { path = "../oxur-lang" }
oxur-comp = { path = "../oxur-comp" }
thiserror.workspace = true
serde.workspace = true
serde_json.workspace = true

[dev-dependencies]
anyhow.workspace = true
```

**File**: `crates/oxur-repl/src/lib.rs`

```rust
//! Oxur REPL
//!
//! Provides a Read-Eval-Print-Loop with three-tier execution:
//! - Tier 1: Direct interpretation for simple expressions
//! - Tier 2: Cached compiled functions
//! - Tier 3: JIT compilation for complex code

pub mod protocol;
pub mod client;
pub mod server;

pub use protocol::{ReplRequest, ReplResponse};
pub use client::ReplClient;
pub use server::ReplServer;

/// Result type for REPL operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for REPL
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Evaluation error: {0}")]
    Eval(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Language error: {0}")]
    Language(#[from] oxur_lang::Error),
    
    #[error("Compilation error: {0}")]
    Compile(#[from] oxur_comp::Error),
}
```

**File**: `crates/oxur-repl/src/protocol.rs`

```rust
//! REPL Protocol
//!
//! Defines the communication protocol between REPL client and server.

use serde::{Deserialize, Serialize};

/// Request from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplRequest {
    /// Evaluate an expression
    Eval { source: String },
    
    /// Load a file
    Load { path: String },
    
    /// Reset the REPL state
    Reset,
    
    /// Get REPL status
    Status,
    
    /// Shutdown the server
    Shutdown,
}

/// Response from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplResponse {
    /// Successful evaluation result
    Value { value: String },
    
    /// Evaluation error
    Error { message: String },
    
    /// Status information
    Status {
        tier1_count: usize,
        tier2_count: usize,
        tier3_count: usize,
    },
    
    /// Acknowledgment
    Ok,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_request_serialization() {
        let req = ReplRequest::Eval {
            source: "(+ 1 2)".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: ReplRequest = serde_json::from_str(&json).unwrap();
        
        match parsed {
            ReplRequest::Eval { source } => assert_eq!(source, "(+ 1 2)"),
            _ => panic!("Wrong variant"),
        }
    }
}
```

**File**: `crates/oxur-repl/src/client.rs`

```rust
//! REPL Client
//!
//! Handles user interaction and communication with the REPL server.

use crate::{protocol::*, Error, Result};

/// REPL client for user interaction
pub struct ReplClient {
    // In a full implementation, this would handle communication
    // with the server process
}

impl ReplClient {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Send a request to the server
    pub fn send(&mut self, request: ReplRequest) -> Result<ReplResponse> {
        // Placeholder implementation
        match request {
            ReplRequest::Eval { .. } => {
                Ok(ReplResponse::Value {
                    value: "result".to_string(),
                })
            }
            ReplRequest::Status => {
                Ok(ReplResponse::Status {
                    tier1_count: 0,
                    tier2_count: 0,
                    tier3_count: 0,
                })
            }
            _ => Ok(ReplResponse::Ok),
        }
    }
    
    /// Run the interactive REPL loop
    pub fn run(&mut self) -> Result<()> {
        println!("Oxur REPL v0.1.0");
        println!("Type (exit) to quit");
        
        // Placeholder - would read from stdin and evaluate
        Ok(())
    }
}

impl Default for ReplClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_creation() {
        let _client = ReplClient::new();
    }
}
```

**File**: `crates/oxur-repl/src/server.rs`

```rust
//! REPL Server
//!
//! Implements three-tier execution strategy:
//! - Tier 1: Interpreter for simple expressions (<1ms)
//! - Tier 2: Cache of compiled functions (~0ms)
//! - Tier 3: JIT compilation for complex code (50-200ms first time)

use crate::{protocol::*, Error, Result};
use oxur_lang::{CoreForm, Expander, Parser};
use std::collections::HashMap;

/// Execution tier for performance tracking
#[derive(Debug, Clone, Copy)]
enum ExecutionTier {
    Interpreter,
    Cached,
    Jit,
}

/// REPL server with tiered execution
pub struct ReplServer {
    parser: Parser,
    expander: Expander,
    cache: HashMap<String, String>,
    stats: TierStats,
}

#[derive(Debug, Default)]
struct TierStats {
    tier1_count: usize,
    tier2_count: usize,
    tier3_count: usize,
}

impl ReplServer {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(String::new()),
            expander: Expander::new(),
            cache: HashMap::new(),
            stats: TierStats::default(),
        }
    }
    
    /// Handle a REPL request
    pub fn handle(&mut self, request: ReplRequest) -> Result<ReplResponse> {
        match request {
            ReplRequest::Eval { source } => {
                self.eval(&source)
            }
            ReplRequest::Load { path } => {
                self.load(&path)
            }
            ReplRequest::Reset => {
                self.reset();
                Ok(ReplResponse::Ok)
            }
            ReplRequest::Status => {
                Ok(ReplResponse::Status {
                    tier1_count: self.stats.tier1_count,
                    tier2_count: self.stats.tier2_count,
                    tier3_count: self.stats.tier3_count,
                })
            }
            ReplRequest::Shutdown => {
                Ok(ReplResponse::Ok)
            }
        }
    }
    
    fn eval(&mut self, source: &str) -> Result<ReplResponse> {
        // Determine execution tier
        let tier = self.choose_tier(source);
        
        match tier {
            ExecutionTier::Interpreter => {
                self.stats.tier1_count += 1;
                self.eval_interpret(source)
            }
            ExecutionTier::Cached => {
                self.stats.tier2_count += 1;
                self.eval_cached(source)
            }
            ExecutionTier::Jit => {
                self.stats.tier3_count += 1;
                self.eval_jit(source)
            }
        }
    }
    
    fn choose_tier(&self, source: &str) -> ExecutionTier {
        // Simple heuristic - would be more sophisticated in practice
        if source.len() < 50 {
            ExecutionTier::Interpreter
        } else if self.cache.contains_key(source) {
            ExecutionTier::Cached
        } else {
            ExecutionTier::Jit
        }
    }
    
    fn eval_interpret(&mut self, source: &str) -> Result<ReplResponse> {
        // Placeholder: direct interpretation
        Ok(ReplResponse::Value {
            value: format!("interpreted: {}", source),
        })
    }
    
    fn eval_cached(&self, source: &str) -> Result<ReplResponse> {
        // Placeholder: cached function call
        if let Some(result) = self.cache.get(source) {
            Ok(ReplResponse::Value {
                value: result.clone(),
            })
        } else {
            Ok(ReplResponse::Error {
                message: "Cache miss".to_string(),
            })
        }
    }
    
    fn eval_jit(&mut self, source: &str) -> Result<ReplResponse> {
        // Placeholder: compile and execute
        let result = format!("jit-compiled: {}", source);
        self.cache.insert(source.to_string(), result.clone());
        Ok(ReplResponse::Value { value: result })
    }
    
    fn load(&mut self, _path: &str) -> Result<ReplResponse> {
        // Placeholder implementation
        Ok(ReplResponse::Ok)
    }
    
    fn reset(&mut self) {
        self.cache.clear();
        self.stats = TierStats::default();
    }
}

impl Default for ReplServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_creation() {
        let server = ReplServer::new();
        assert_eq!(server.stats.tier1_count, 0);
    }
    
    #[test]
    fn test_tier_selection() {
        let server = ReplServer::new();
        
        // Short expression → Tier 1
        let tier = server.choose_tier("(+ 1 2)");
        assert!(matches!(tier, ExecutionTier::Interpreter));
        
        // Long expression → Tier 3
        let tier = server.choose_tier(&"x".repeat(100));
        assert!(matches!(tier, ExecutionTier::Jit));
    }
}
```

**File**: `crates/oxur-repl/README.md`

```markdown
# oxur-repl

REPL implementation with three-tier execution for optimal performance.

## Architecture

### Three-Tier Execution

1. **Tier 1 - Interpreter** (<1ms)
   - Direct interpretation for simple expressions
   - Fast startup, good for interactive exploration
   
2. **Tier 2 - Cached** (~0ms)
   - Previously compiled functions
   - Just function call overhead
   
3. **Tier 3 - JIT** (50-200ms first time, cached after)
   - Full compilation for complex code
   - Native performance after first run

## Protocol

The REPL uses a simple request/response protocol:

- `Eval` - Evaluate an expression
- `Load` - Load a file
- `Reset` - Clear REPL state
- `Status` - Get execution statistics
- `Shutdown` - Stop the server

## Usage

```rust
use oxur_repl::ReplServer;

let mut server = ReplServer::new();
let response = server.handle(ReplRequest::Eval {
    source: "(+ 1 2)".to_string()
});
```
```

---

### Task 5: Create oxur-cli Crate

**File**: `crates/oxur-cli/Cargo.toml`

```toml
[package]
name = "oxur-cli"
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Oxur CLI tool for running, compiling, and managing Oxur projects"

[[bin]]
name = "oxur"
path = "src/main.rs"

[dependencies]
oxur-lang = { path = "../oxur-lang" }
oxur-comp = { path = "../oxur-comp" }
oxur-repl = { path = "../oxur-repl" }
anyhow.workspace = true
clap.workspace = true

[dev-dependencies]
```

**File**: `crates/oxur-cli/src/main.rs`

```rust
//! oxur - Oxur CLI Tool
//!
//! Main command-line interface for Oxur projects.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oxur")]
#[command(about = "Oxur - A Lisp that compiles to Rust", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile an Oxur file to binary
    Compile {
        /// Input Oxur source file
        input: PathBuf,
        
        /// Output binary path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Run an Oxur file (compile and execute)
    Run {
        /// Input Oxur source file
        input: PathBuf,
        
        /// Arguments to pass to the program
        args: Vec<String>,
    },
    
    /// Start the interactive REPL
    Repl,
    
    /// Create a new Oxur project
    New {
        /// Project name
        name: String,
    },
    
    /// Build the current project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    
    /// Run tests
    Test,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Compile { input, output } => {
            println!("Compiling: {}", input.display());
            
            // Read source
            let source = std::fs::read_to_string(&input)?;
            
            // Parse and expand
            let mut parser = oxur_lang::Parser::new(source);
            let surface_forms = parser.parse()?;
            
            let mut expander = oxur_lang::Expander::new();
            let core_forms = expander.expand(surface_forms)?;
            
            // Compile
            let output = output.unwrap_or_else(|| input.with_extension(""));
            let build_dir = PathBuf::from(".oxur-build");
            
            let mut compiler = oxur_comp::Compiler::new(build_dir);
            compiler.compile(core_forms, &output)?;
            
            println!("Compiled successfully: {}", output.display());
        }
        
        Commands::Run { input, args } => {
            println!("Running: {}", input.display());
            
            // Would compile and execute
            if !args.is_empty() {
                println!("With args: {:?}", args);
            }
            
            println!("(Not yet implemented)");
        }
        
        Commands::Repl => {
            println!("Starting REPL...");
            let mut client = oxur_repl::ReplClient::new();
            client.run()?;
        }
        
        Commands::New { name } => {
            println!("Creating new project: {}", name);
            
            // Would create project directory structure
            let project_dir = PathBuf::from(&name);
            std::fs::create_dir_all(&project_dir)?;
            
            println!("Created project directory: {}", project_dir.display());
            println!("(Not yet fully implemented)");
        }
        
        Commands::Build { release } => {
            println!("Building project...");
            if release {
                println!("Release mode enabled");
            }
            println!("(Not yet implemented)");
        }
        
        Commands::Test => {
            println!("Running tests...");
            println!("(Not yet implemented)");
        }
    }
    
    Ok(())
}
```

**File**: `crates/oxur-cli/README.md`

```markdown
# oxur-cli

Main CLI tool for working with Oxur.

## Commands

### Compile
```bash
oxur compile input.ox -o output
```

Compile an Oxur file to a native binary.

### Run
```bash
oxur run input.ox -- arg1 arg2
```

Compile and run an Oxur file with arguments.

### REPL
```bash
oxur repl
```

Start the interactive REPL.

### New
```bash
oxur new my-project
```

Create a new Oxur project with standard structure.

### Build
```bash
oxur build
oxur build --release
```

Build the current project.

### Test
```bash
oxur test
```

Run tests in the current project.

## Future Features

- Package management integration
- Dependency resolution
- Project templates
- IDE tool support
```

---

## Verification Steps

After creating all files, run these commands to verify everything compiles:

```bash
# From the workspace root

# Check that all crates compile
cargo check --workspace

# Build all binaries
cargo build --workspace

# Run tests
cargo test --workspace

# Try the binaries
cargo run --bin oxurc -- --help
cargo run --bin oxur -- --help

# Verify binary names
ls -la target/debug/oxurc
ls -la target/debug/oxur
```

## Success Criteria

✅ All crates compile without errors
✅ All tests pass
✅ Both binaries (`oxurc` and `oxur`) are created
✅ Help text displays correctly for both binaries
✅ Workspace structure matches the architecture document
✅ All placeholder implementations have basic tests
✅ Dependencies are properly configured

## Notes

- Each crate has placeholder implementations that compile successfully
- Core architecture from the document is reflected in the structure
- Source maps, node IDs, and compilation stages are properly modeled
- REPL has three-tier execution framework in place
- Error types are properly structured with `thiserror`
- Tests demonstrate the intended API usage

This provides a solid foundation for implementing the full Oxur compilation pipeline incrementally.
