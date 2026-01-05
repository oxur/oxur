# oxur-smap

**Source mapping for the Oxur language** - Tracks code transformations through the compilation pipeline for accurate error reporting.

## Overview

`oxur-smap` is a foundation crate that provides source mapping capabilities for the Oxur compiler. It enables rustc compiler errors to be translated back to the original Oxur source code positions by tracking transformations through multiple compilation stages.

### Key Features

- **Multi-stage transformation tracking** - Surface Forms → Core Forms → Rust AST
- **Backward error translation** - Map Rust compiler errors to original Oxur source
- **Thread-safe NodeId generation** - Atomic counter with global singleton
- **Content-based caching** - Structure-only hashing for cache key generation
- **Zero dependencies** - Lightweight foundation crate
- **Comprehensive test coverage** - 98%+ coverage with unit and integration tests

## Architecture

The source map tracks three transformation stages:

```
┌─────────────────────────────────────────────────────────────┐
│                  OXUR COMPILATION PIPELINE                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Oxur Source (.ox)                                          │
│         ↓                                                   │
│  ┌──────────────────┐                                       │
│  │   Parser         │  Creates Surface Form AST             │
│  │                  │  Records: NodeId → SourcePos          │
│  └──────────────────┘                                       │
│         ↓                                                   │
│  Surface Forms (Lisp-like AST)                              │
│         ↓                                                   │
│  ┌──────────────────┐                                       │
│  │   Expander       │  Macro expansion, desugaring          │
│  │                  │  Records: Surface NodeId → Core NodeId│
│  └──────────────────┘                                       │
│         ↓                                                   │
│  Core Forms (Canonical IR)                                  │
│         ↓                                                   │
│  ┌──────────────────┐                                       │
│  │   Lowering       │  Generate Rust AST                    │
│  │                  │  Records: Core NodeId → Rust NodeId   │
│  └──────────────────┘                                       │
│         ↓                                                   │
│  Rust AST                                                   │
│         ↓                                                   │
│  ┌──────────────────┐                                       │
│  │   rustc          │  Compile to binary                    │
│  │                  │  Errors reference Rust NodeIds        │
│  └──────────────────┘                                       │
│         ↓                                                   │
│  ┌──────────────────┐                                       │
│  │  Error Mapper    │  Lookup: Rust NodeId → SourcePos     │
│  │                  │  Traverses: Rust → Core → Surface    │
│  └──────────────────┘                                       │
│         ↓                                                   │
│  User-friendly error message with original source position  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Core Types

#### `NodeId`

Unique identifier for AST nodes across all compilation stages.

```rust
pub struct NodeId(u32);

// Thread-safe global generator
pub fn new_node_id() -> NodeId;
```

#### `SourcePos`

Tracks source code positions with file path, line, column, and span length.

```rust
pub struct SourcePos {
    pub file: String,    // File path or "<repl>"
    pub line: u32,       // 1-indexed line number
    pub column: u32,     // 1-indexed column number
    pub length: u32,     // Span length for error highlighting
}
```

#### `SourceMap`

Maps NodeIds through transformation stages and provides backward lookup.

```rust
pub struct SourceMap {
    // Three-stage mapping:
    surface_positions: HashMap<NodeId, SourcePos>,
    surface_to_core: HashMap<NodeId, NodeId>,
    core_to_rust: HashMap<NodeId, NodeId>,
}
```

## Usage

### Basic Workflow

```rust
use oxur_smap::{SourceMap, new_node_id, SourcePos};

let mut map = SourceMap::new();

// Stage 1: Parser creates surface node
let surface = new_node_id();
let pos = SourcePos::new("example.ox".to_string(), 10, 5, 15);
map.record_surface_node(surface, pos);

// Stage 2: Expander creates core node
let core = new_node_id();
map.record_expansion(surface, core);

// Stage 3: Lowering creates rust node
let rust = new_node_id();
map.record_lowering(core, rust);

// Error translation: Map Rust error back to original source
if let Some(original_pos) = map.lookup(&rust) {
    println!("Error at {}:{}:{}",
        original_pos.file,
        original_pos.line,
        original_pos.column);
}
```

### REPL Usage

For REPL input, use the `SourcePos::repl()` constructor:

```rust
let pos = SourcePos::repl(1, 1, 20);  // Line 1, column 1, length 20
assert_eq!(pos.file, "<repl>");
```

### Cache Key Generation

For incremental compilation, generate cache keys based on transformation structure:

```rust
let hash = map.content_hash();
// Use hash as part of cache key for compiled artifacts
// Note: Hash is structure-based, excludes source positions
```

## Design Documentation

This implementation follows the specifications in:

- **ODD-0038**: Source Map Architecture (definitive specification)
- **ODD-0013**: Compilation Chain Architecture
- **ODD-0001**: Oxur Letter of Intent

See `../../design/docs/` for complete design documentation.

## Testing

The crate includes comprehensive test coverage:

- **Unit tests**: Each module (NodeId, SourcePos, SourceMap)
- **Integration tests**: End-to-end compilation simulation
- **Thread safety**: Concurrent NodeId generation verification
- **Coverage**: 98.41% (339/339 lines, 52/52 functions)

Run tests:

```bash
cargo test --package oxur-smap
```

Generate coverage report:

```bash
cargo llvm-cov --package oxur-smap --html
open target/llvm-cov/html/index.html
```

## API Documentation

Generate full API documentation:

```bash
cargo doc --package oxur-smap --no-deps --open
```

## Implementation Notes

### Design Decisions

**Single Global Counter**: We use a single global atomic counter for NodeId generation rather than per-stage ranges (e.g., 100-199 for surface). This simplifies implementation, avoids range exhaustion, and keeps NodeIds opaque.

**Structure-Only Hashing**: The `content_hash()` method hashes the transformation graph but NOT source positions. This allows cached artifacts to be reused when code structure is identical, even if moved to different lines. Trade-off: Very subtle semantic changes might be missed (acceptable for v1.0).

**Three Separate HashMaps**: We maintain three separate HashMaps for each transformation stage rather than a single unified structure. This provides clear separation of concerns and efficient lookup at each stage.

### Thread Safety

NodeId generation is thread-safe using `AtomicU32` with `SeqCst` ordering. The global generator can be safely accessed from multiple threads simultaneously.

## Dependencies

**None** - This is a zero-dependency foundation crate.

## Contributing

See the main Oxur project documentation:

- **CLAUDE.md** - Development guidelines
- **assets/ai/ai-rust/** - Rust best practices
- **Makefile** - Development workflow targets

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Status

**Phase 0: Complete** - Foundation implementation finished with comprehensive testing.

**Next Phase**: Integration with oxur-lang, oxur-comp, and oxur-repl for end-to-end error mapping.
