---
number: 39
title: "oxur-smap Implementation Plan"
author: "macro expander"
component: All
tags: [change-me]
created: 2026-01-05
updated: 2026-01-05
state: Active
supersedes: null
superseded-by: null
version: 1.0
---

# oxur-smap Implementation Plan

## Executive Summary

This document provides a complete, actionable implementation plan for the `oxur-smap` crate - Oxur's source map system for tracking code transformations and enabling high-quality error messages. The design is **production-ready** based on comprehensive specifications in documents 0038, 0030, and supporting materials.

**Key Facts:**

- **Status**: Design complete, implementation not started
- **Priority**: P0 - Critical path blocker for all other work
- **Unique Feature**: Multi-stage transformation tracking (Surface → Core → Rust)
- **Competitive Advantage**: No other Lisp has this capability
- **Estimated Effort**: 2-3 days for Phase 0 core implementation

---

## Table of Contents

1. [Overview & Context](#1-overview--context)
2. [Architecture](#2-architecture)
3. [Phase 0: Core Implementation](#3-phase-0-core-implementation)
4. [Phase 1: Integration](#4-phase-1-integration)
5. [Phase 2: Polish & Optimization](#5-phase-2-polish--optimization)
6. [Testing Strategy](#6-testing-strategy)
7. [Implementation Checklist](#7-implementation-checklist)
8. [Open Design Decisions](#8-open-design-decisions)
9. [Success Criteria](#9-success-criteria)

---

## 1. Overview & Context

### 1.1 What is oxur-smap?

`oxur-smap` is a **foundation crate** (zero dependencies) that provides source mapping capabilities for tracking code transformations through the Oxur compilation pipeline. It enables rustc compiler errors to be translated back to the original Oxur source code positions.

### 1.2 Why It Matters

**Problem:**
When Oxur code is transformed through multiple stages (Surface Forms → Core Forms → Rust AST → Rust Code), compiler errors from `rustc` reference positions in the generated Rust code, not the original Oxur source.

**Solution:**
`oxur-smap` tracks transformations at each stage, allowing backward traversal from Rust error positions to the original Oxur source location that caused the error.

**Impact:**

- **Developer Experience**: Rustc-quality error messages in Oxur REPL
- **Competitive Advantage**: Unique feature no other Lisp provides
- **Foundation**: Enables other advanced tooling (debugger, profiler, IDE support)

### 1.3 Position in Architecture

```
┌──────────────────────────────────────────────────┐
│           Oxur Compilation Pipeline              │
├──────────────────────────────────────────────────┤
│  Surface Forms  →  Core Forms  →  Rust AST       │
│       ↓                ↓              ↓          │
│    oxur-lang      oxur-lang      oxur-comp       │
│       ↓                ↓              ↓          │
│ record_surface record_expansion record_lowering  |
│     node()            ()             ()          │
│       ↓                ↓              ↓          │
│  ┌────────────────────────────────────────────┐  │
│  │           oxur-smap (THIS CRATE)           │  │
│  │  ┌──────────────────────────────────────┐  │  │
│  │  │  SourceMap                           │  │  │
│  │  │  - surface_positions: NodeId → Pos   │  │  │
│  │  │  - surface_to_core: NodeId → NodeId  │  │  │
│  │  │  - core_to_rust: NodeId → NodeId     │  │  │
│  │  └──────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────┘  │
│                       ↓                          │
│                 lookup(rust_node)                │
│                       ↓                          │
│             Original Source Position             │
└──────────────────────────────────────────────────┘
```

**Dependencies:**

- **Upstream**: None (foundation crate)
- **Downstream**: oxur-lang, oxur-comp, oxur-repl

**Critical Path:**
This crate **blocks all other implementation work**. No other crate can be completed without `oxur-smap` types and APIs.

---

## 2. Architecture

### 2.1 Core Types (from 0038 Section 2.1)

#### NodeId

```rust
/// Unique identifier for AST nodes across all compilation stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);
```

**Design Notes:**

- Uses `u32` (not `u64`) as specified in doc 0038
- Must be globally unique across all stages
- Used as HashMap key (requires Hash, Eq)
- Lightweight (4 bytes) for efficient copying

#### SourcePos

```rust
/// Source position in original Oxur code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePos {
    pub file: String,      // Source file path
    pub line: u32,         // 1-indexed line number
    pub column: u32,       // 1-indexed column number
    pub length: u32,       // Span length for highlighting
}
```

**Design Notes:**

- 1-indexed (matches editor conventions)
- `length` enables highlighting full expressions
- Clone-able for error reporting
- String for file path (could be PathBuf, but String is more flexible for REPL)

#### SourceMap

```rust
/// Tracks AST transformations for error reporting
pub struct SourceMap {
    // Surface Form positions (from parsing)
    surface_positions: HashMap<NodeId, SourcePos>,

    // Transformation chains
    surface_to_core: HashMap<NodeId, NodeId>,  // Expansion
    core_to_rust: HashMap<NodeId, NodeId>,     // Lowering
}
```

**Design Notes:**

- Three separate HashMaps for the three transformation stages
- Intentionally not a graph (simpler, faster lookup)
- Memory overhead: ~72 bytes per node (3 HashMap entries)
- Expected size: ~1000 nodes per typical compilation

### 2.2 Core Operations

The API follows the **record-lookup pattern**:

**Recording Phase** (during compilation):

1. `record_surface_node()` - Called by parser
2. `record_expansion()` - Called by macro expander
3. `record_lowering()` - Called by Rust AST generator

**Lookup Phase** (during error translation):
4. `lookup()` - Traverses backward through transformation chain

### 2.3 Transformation Flow

```
┌──────────────┐
│  User Types  │  def add(x, y): x + y
└──────┬───────┘
       │ oxur-lang::parse()
       ↓
┌──────────────┐
│ Surface Form │  NodeId=100
└──────┬───────┘  SourcePos{line=1, col=0, len=21}
       │
       │ record_surface_node(100, pos)
       ↓
┌──────────────────────────────────┐
│ surface_positions[100] = pos     │
└──────────────────────────────────┘
       │
       │ oxur-lang::expand()
       ↓
┌──────────────┐
│  Core Form   │  NodeId=200
└──────┬───────┘
       │ record_expansion(100, 200)
       ↓
┌──────────────────────────────────┐
│ surface_to_core[100] = 200       │
└──────────────────────────────────┘
       │
       │ oxur-comp::lower()
       ↓
┌──────────────┐
│  Rust AST    │  NodeId=300
└──────┬───────┘
       │ record_lowering(200, 300)
       ↓
┌──────────────────────────────────┐
│ core_to_rust[200] = 300          │
└──────────────────────────────────┘
       │
       │ rustc error at rust_node=300
       ↓
┌──────────────┐
│   lookup()   │  Backward traversal:
└──────┬───────┘  300 → 200 → 100 → SourcePos
       │
       ↓
  Original Position
  line=1, col=0, len=21
```

---

## 3. Phase 0: Core Implementation

**Goal:** Create `oxur-smap` crate with all core types and operations.
**Duration:** 2-3 days
**Status:** Unblocked, ready to start immediately

### 3.1 Crate Setup

#### Step 1: Create Crate Structure

```bash
cargo new --lib oxur-smap
cd oxur-smap
```

#### Step 2: Configure Cargo.toml

```toml
[package]
name = "oxur-smap"
version = "0.1.0"
edition = "2021"
authors = ["Duncan McGreggor"]
license = "MIT OR Apache-2.0"
description = "Source mapping for Oxur language - tracks code transformations for error reporting"
repository = "https://github.com/oxur-lang/oxur"
keywords = ["compiler", "source-map", "error-reporting"]
categories = ["development-tools"]

[dependencies]
# Intentionally empty - this is a foundation crate with zero dependencies

[dev-dependencies]
# For testing only
```

**Design Decision:** Zero dependencies ensures `oxur-smap` can be used anywhere without version conflicts.

#### Step 3: Module Structure

```
oxur-smap/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API and re-exports
│   ├── node_id.rs       # NodeId type and generator
│   ├── source_pos.rs    # SourcePos type
│   ├── source_map.rs    # SourceMap implementation
│   └── hash.rs          # content_hash() implementation
└── tests/
    ├── basic.rs         # Basic functionality tests
    ├── lookup.rs        # Lookup algorithm tests
    └── integration.rs   # End-to-end tests
```

### 3.2 Implementation: NodeId (src/node_id.rs)

```rust
use std::sync::atomic::{AtomicU32, Ordering};

/// Unique identifier for AST nodes across all compilation stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    /// Create a new NodeId from a raw u32 value
    ///
    /// # Safety
    /// The caller must ensure uniqueness across all nodes.
    /// Typically only used for deserialization or testing.
    pub const fn from_raw(id: u32) -> Self {
        NodeId(id)
    }

    /// Get the raw u32 value
    pub const fn as_raw(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

/// Global NodeId generator using atomic counter
///
/// Thread-safe generation of unique NodeIds.
///
/// # Design Decision: Single Global Counter vs Stage Ranges
///
/// We use a single global atomic counter rather than per-stage ranges
/// (e.g., 100-199 for surface, 200-299 for core) because:
///
/// 1. Simpler implementation (single AtomicU32)
/// 2. No risk of range exhaustion
/// 3. NodeIds are opaque - internal structure doesn't matter
/// 4. Debuggability via SourceMap lookup, not NodeId values
///
/// If debugging requires it, we can add stage tagging later without
/// breaking the API.
pub struct NodeIdGenerator {
    next_id: AtomicU32,
}

impl NodeIdGenerator {
    /// Create a new generator starting from 1 (0 reserved for invalid)
    pub const fn new() -> Self {
        Self {
            next_id: AtomicU32::new(1),
        }
    }

    /// Generate the next unique NodeId
    ///
    /// # Panics
    /// Panics if we exceed u32::MAX nodes (2^32 - 1).
    /// This is unlikely in practice (~4 billion nodes).
    pub fn next(&self) -> NodeId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        assert!(id < u32::MAX, "NodeId counter overflow");
        NodeId(id)
    }

    /// Reset the generator (for testing only)
    #[cfg(test)]
    pub fn reset(&self) {
        self.next_id.store(1, Ordering::SeqCst);
    }
}

impl Default for NodeIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Global singleton generator
///
/// Thread-safe access to NodeId generation.
static GLOBAL_GENERATOR: NodeIdGenerator = NodeIdGenerator::new();

/// Generate a new unique NodeId
///
/// This is the primary API for obtaining NodeIds during compilation.
pub fn new_node_id() -> NodeId {
    GLOBAL_GENERATOR.next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_basic() {
        let id1 = NodeId::from_raw(42);
        let id2 = NodeId::from_raw(42);
        assert_eq!(id1, id2);
        assert_eq!(id1.as_raw(), 42);
    }

    #[test]
    fn node_id_generator() {
        let gen = NodeIdGenerator::new();
        let id1 = gen.next();
        let id2 = gen.next();
        assert_ne!(id1, id2);
        assert_eq!(id1.as_raw() + 1, id2.as_raw());
    }

    #[test]
    fn global_generator_thread_safe() {
        use std::sync::Arc;
        use std::thread;

        let ids = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut handles = vec![];

        // Spawn 10 threads, each generating 100 NodeIds
        for _ in 0..10 {
            let ids_clone = Arc::clone(&ids);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let id = new_node_id();
                    ids_clone.lock().unwrap().push(id);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // All 1000 IDs should be unique
        let mut id_set = std::collections::HashSet::new();
        for id in ids.lock().unwrap().iter() {
            assert!(id_set.insert(*id), "Duplicate NodeId generated");
        }
    }
}
```

**Design Notes:**

- **Atomic Counter**: Thread-safe without locks (using `SeqCst` ordering for correctness)
- **Overflow Protection**: Assert on u32::MAX (4 billion nodes is practically infinite)
- **Testing API**: `reset()` only available in test builds
- **Global Singleton**: Simplest API - just call `new_node_id()`

### 3.3 Implementation: SourcePos (src/source_pos.rs)

```rust
/// Source position in original Oxur code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePos {
    /// Source file path (or "<repl>" for REPL input)
    pub file: String,

    /// 1-indexed line number
    pub line: u32,

    /// 1-indexed column number
    pub column: u32,

    /// Length of the span (for error highlighting)
    pub length: u32,
}

impl SourcePos {
    /// Create a new source position
    pub fn new(file: String, line: u32, column: u32, length: u32) -> Self {
        assert!(line > 0, "Line numbers are 1-indexed");
        assert!(column > 0, "Column numbers are 1-indexed");
        Self { file, line, column, length }
    }

    /// Create a position for REPL input
    pub fn repl(line: u32, column: u32, length: u32) -> Self {
        Self::new("<repl>".to_string(), line, column, length)
    }

    /// Get end column (column + length)
    pub fn end_column(&self) -> u32 {
        self.column + self.length
    }

    /// Check if this position contains another position
    pub fn contains(&self, other: &SourcePos) -> bool {
        self.file == other.file
            && self.line == other.line
            && other.column >= self.column
            && other.column < self.end_column()
    }
}

impl std::fmt::Display for SourcePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_pos_basic() {
        let pos = SourcePos::new("test.oxur".to_string(), 1, 5, 10);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.length, 10);
        assert_eq!(pos.end_column(), 15);
    }

    #[test]
    fn source_pos_repl() {
        let pos = SourcePos::repl(1, 1, 20);
        assert_eq!(pos.file, "<repl>");
    }

    #[test]
    fn source_pos_contains() {
        let span = SourcePos::new("test.oxur".to_string(), 1, 5, 10);
        let inner = SourcePos::new("test.oxur".to_string(), 1, 7, 3);
        let outer = SourcePos::new("test.oxur".to_string(), 1, 3, 2);

        assert!(span.contains(&inner));
        assert!(!span.contains(&outer));
    }

    #[test]
    #[should_panic(expected = "Line numbers are 1-indexed")]
    fn source_pos_zero_line() {
        SourcePos::new("test.oxur".to_string(), 0, 1, 1);
    }

    #[test]
    #[should_panic(expected = "Column numbers are 1-indexed")]
    fn source_pos_zero_column() {
        SourcePos::new("test.oxur".to_string(), 1, 0, 1);
    }
}
```

**Design Notes:**

- **1-Indexed**: Matches editor conventions (most editors start at line 1, column 1)
- **Length Field**: Enables highlighting entire expressions in error messages
- **REPL Support**: Special constructor for REPL input (common case)
- **Contains Check**: Useful for multi-line expression handling (future)

### 3.4 Implementation: SourceMap (src/source_map.rs)

```rust
use std::collections::HashMap;
use crate::{NodeId, SourcePos};

/// Tracks AST transformations for error reporting
///
/// This is the core of the source mapping system. It maintains three
/// separate mappings:
///
/// 1. surface_positions: NodeId → SourcePos (from parser)
/// 2. surface_to_core: NodeId → NodeId (from macro expansion)
/// 3. core_to_rust: NodeId → NodeId (from lowering)
///
/// These three maps enable backward traversal from a Rust compiler
/// error position back to the original Oxur source code.
#[derive(Debug, Default)]
pub struct SourceMap {
    /// Surface Form positions (recorded during parsing)
    surface_positions: HashMap<NodeId, SourcePos>,

    /// Transformation chain: Surface → Core (recorded during expansion)
    surface_to_core: HashMap<NodeId, NodeId>,

    /// Transformation chain: Core → Rust (recorded during lowering)
    core_to_rust: HashMap<NodeId, NodeId>,
}

impl SourceMap {
    /// Create a new empty source map
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a surface form node position
    ///
    /// Called by the parser when creating surface form AST nodes.
    ///
    /// # Example
    /// ```
    /// use oxur_smap::{SourceMap, NodeId, SourcePos};
    ///
    /// let mut map = SourceMap::new();
    /// let node = NodeId::from_raw(100);
    /// let pos = SourcePos::repl(1, 1, 10);
    ///
    /// map.record_surface_node(node, pos);
    /// assert_eq!(map.get_surface_position(&node).unwrap().line, 1);
    /// ```
    pub fn record_surface_node(&mut self, node: NodeId, pos: SourcePos) {
        self.surface_positions.insert(node, pos);
    }

    /// Record an expansion transformation
    ///
    /// Called by the macro expander when transforming a surface form
    /// node into a core form node.
    ///
    /// # Example
    /// ```
    /// use oxur_smap::{SourceMap, NodeId};
    ///
    /// let mut map = SourceMap::new();
    /// let surface = NodeId::from_raw(100);
    /// let core = NodeId::from_raw(200);
    ///
    /// map.record_expansion(surface, core);
    /// assert_eq!(map.get_core_from_surface(&surface), Some(&core));
    /// ```
    pub fn record_expansion(&mut self, surface: NodeId, core: NodeId) {
        self.surface_to_core.insert(surface, core);
    }

    /// Record a lowering transformation
    ///
    /// Called by the Rust AST generator when transforming a core form
    /// node into a Rust AST node.
    ///
    /// # Example
    /// ```
    /// use oxur_smap::{SourceMap, NodeId};
    ///
    /// let mut map = SourceMap::new();
    /// let core = NodeId::from_raw(200);
    /// let rust = NodeId::from_raw(300);
    ///
    /// map.record_lowering(core, rust);
    /// assert_eq!(map.get_rust_from_core(&core), Some(&rust));
    /// ```
    pub fn record_lowering(&mut self, core: NodeId, rust: NodeId) {
        self.core_to_rust.insert(core, rust);
    }

    /// Look up the original source position for a Rust AST node
    ///
    /// This performs backward traversal through the transformation chain:
    /// Rust → Core → Surface → SourcePos
    ///
    /// Returns None if any link in the chain is missing.
    ///
    /// # Example
    /// ```
    /// use oxur_smap::{SourceMap, NodeId, SourcePos};
    ///
    /// let mut map = SourceMap::new();
    /// let surface = NodeId::from_raw(100);
    /// let core = NodeId::from_raw(200);
    /// let rust = NodeId::from_raw(300);
    /// let pos = SourcePos::repl(1, 5, 10);
    ///
    /// map.record_surface_node(surface, pos.clone());
    /// map.record_expansion(surface, core);
    /// map.record_lowering(core, rust);
    ///
    /// let result = map.lookup(&rust).unwrap();
    /// assert_eq!(result.line, 1);
    /// assert_eq!(result.column, 5);
    /// ```
    pub fn lookup(&self, rust_node: &NodeId) -> Option<SourcePos> {
        // Step 1: Rust → Core
        // Find the core node that maps to this rust node
        let core_node = self.core_to_rust
            .iter()
            .find(|(_, &r)| r == *rust_node)
            .map(|(c, _)| c)?;

        // Step 2: Core → Surface
        // Find the surface node that maps to this core node
        let surface_node = self.surface_to_core
            .iter()
            .find(|(_, &c)| c == *core_node)
            .map(|(s, _)| s)?;

        // Step 3: Surface → SourcePos
        // Look up the original position
        self.surface_positions.get(surface_node).cloned()
    }

    /// Get surface position directly (for testing/debugging)
    pub fn get_surface_position(&self, node: &NodeId) -> Option<&SourcePos> {
        self.surface_positions.get(node)
    }

    /// Get core node from surface node (for testing/debugging)
    pub fn get_core_from_surface(&self, surface: &NodeId) -> Option<&NodeId> {
        self.surface_to_core.get(surface)
    }

    /// Get rust node from core node (for testing/debugging)
    pub fn get_rust_from_core(&self, core: &NodeId) -> Option<&NodeId> {
        self.core_to_rust.get(core)
    }

    /// Get statistics about the source map (for debugging)
    pub fn stats(&self) -> SourceMapStats {
        SourceMapStats {
            surface_nodes: self.surface_positions.len(),
            expansions: self.surface_to_core.len(),
            lowerings: self.core_to_rust.len(),
        }
    }
}

/// Statistics about a SourceMap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceMapStats {
    pub surface_nodes: usize,
    pub expansions: usize,
    pub lowerings: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_map_empty() {
        let map = SourceMap::new();
        let node = NodeId::from_raw(100);
        assert!(map.lookup(&node).is_none());
    }

    #[test]
    fn source_map_record_surface() {
        let mut map = SourceMap::new();
        let node = NodeId::from_raw(100);
        let pos = SourcePos::repl(1, 1, 10);

        map.record_surface_node(node, pos.clone());
        assert_eq!(map.get_surface_position(&node).unwrap().line, 1);
    }

    #[test]
    fn source_map_full_chain() {
        let mut map = SourceMap::new();

        // Create a full transformation chain
        let surface = NodeId::from_raw(100);
        let core = NodeId::from_raw(200);
        let rust = NodeId::from_raw(300);
        let pos = SourcePos::repl(1, 5, 10);

        // Record transformations
        map.record_surface_node(surface, pos.clone());
        map.record_expansion(surface, core);
        map.record_lowering(core, rust);

        // Lookup should traverse the full chain
        let result = map.lookup(&rust).unwrap();
        assert_eq!(result.line, 1);
        assert_eq!(result.column, 5);
        assert_eq!(result.length, 10);
    }

    #[test]
    fn source_map_broken_chain() {
        let mut map = SourceMap::new();

        let surface = NodeId::from_raw(100);
        let core = NodeId::from_raw(200);
        let rust = NodeId::from_raw(300);
        let pos = SourcePos::repl(1, 1, 10);

        // Only record surface and expansion (missing lowering)
        map.record_surface_node(surface, pos);
        map.record_expansion(surface, core);

        // Lookup should fail (no lowering recorded)
        assert!(map.lookup(&rust).is_none());
    }

    #[test]
    fn source_map_stats() {
        let mut map = SourceMap::new();

        let surface1 = NodeId::from_raw(100);
        let surface2 = NodeId::from_raw(101);
        let core1 = NodeId::from_raw(200);
        let pos1 = SourcePos::repl(1, 1, 5);
        let pos2 = SourcePos::repl(2, 1, 8);

        map.record_surface_node(surface1, pos1);
        map.record_surface_node(surface2, pos2);
        map.record_expansion(surface1, core1);

        let stats = map.stats();
        assert_eq!(stats.surface_nodes, 2);
        assert_eq!(stats.expansions, 1);
        assert_eq!(stats.lowerings, 0);
    }
}
```

**Design Notes:**

- **Backward Lookup**: The `lookup()` method is O(n) where n is the number of transformations (typically <1000). This is acceptable because:
  - Lookups only happen on errors (rare)
  - HashMaps are already fast
  - Could optimize with reverse indices if needed (future)
- **Optional Links**: Returns `None` if any link in the chain is missing (graceful degradation)
- **Testing Helpers**: Additional getters for testing and debugging
- **Statistics**: Useful for debugging and monitoring

### 3.5 Implementation: Content Hashing (src/hash.rs)

```rust
use crate::SourceMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl SourceMap {
    /// Generate a content hash for cache key generation
    ///
    /// This hash includes the structure of all transformations but NOT
    /// the actual source positions. This allows cached artifacts to be
    /// reused when the transformation structure is identical, even if
    /// line numbers have changed.
    ///
    /// # Design Decision: Structure-Only Hashing
    ///
    /// We hash the transformation graph (NodeId → NodeId mappings) but
    /// NOT the surface positions. This means:
    ///
    /// - Same code structure → Same hash (even if moved to different line)
    /// - Cache hits more frequent (position changes don't invalidate)
    /// - Trade-off: Very subtle semantic changes might be missed
    ///
    /// For v1.0, this is acceptable. If needed, we can add a flag for
    /// "strict hashing" that includes positions in v1.1+.
    pub fn content_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash transformation structure (not positions)
        // Sort for deterministic hashing
        let mut expansions: Vec<_> = self.surface_to_core.iter().collect();
        expansions.sort_by_key(|(k, _)| *k);
        for (surface, core) in expansions {
            surface.hash(&mut hasher);
            core.hash(&mut hasher);
        }

        let mut lowerings: Vec<_> = self.core_to_rust.iter().collect();
        lowerings.sort_by_key(|(k, _)| *k);
        for (core, rust) in lowerings {
            core.hash(&mut hasher);
            rust.hash(&mut hasher);
        }

        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeId, SourcePos};

    #[test]
    fn content_hash_deterministic() {
        let mut map1 = SourceMap::new();
        let mut map2 = SourceMap::new();

        let surface = NodeId::from_raw(100);
        let core = NodeId::from_raw(200);
        let rust = NodeId::from_raw(300);

        // Build identical transformation structures
        map1.record_expansion(surface, core);
        map1.record_lowering(core, rust);

        map2.record_expansion(surface, core);
        map2.record_lowering(core, rust);

        // Hashes should be identical
        assert_eq!(map1.content_hash(), map2.content_hash());
    }

    #[test]
    fn content_hash_position_independent() {
        let mut map1 = SourceMap::new();
        let mut map2 = SourceMap::new();

        let surface = NodeId::from_raw(100);
        let core = NodeId::from_raw(200);
        let pos1 = SourcePos::repl(1, 1, 10);
        let pos2 = SourcePos::repl(99, 1, 10); // Different line

        // Same structure, different positions
        map1.record_surface_node(surface, pos1);
        map1.record_expansion(surface, core);

        map2.record_surface_node(surface, pos2);
        map2.record_expansion(surface, core);

        // Hashes should still be identical (positions not included)
        assert_eq!(map1.content_hash(), map2.content_hash());
    }

    #[test]
    fn content_hash_structure_sensitive() {
        let mut map1 = SourceMap::new();
        let mut map2 = SourceMap::new();

        let surface = NodeId::from_raw(100);
        let core1 = NodeId::from_raw(200);
        let core2 = NodeId::from_raw(201); // Different core node

        map1.record_expansion(surface, core1);
        map2.record_expansion(surface, core2);

        // Different structure → different hash
        assert_ne!(map1.content_hash(), map2.content_hash());
    }
}
```

**Design Notes:**

- **Structure-Only Hashing**: Intentionally excludes source positions for better cache hit rate
- **Deterministic**: Sorts keys before hashing to ensure consistency
- **Fast**: Uses `DefaultHasher` (optimized for speed, not cryptographic security)
- **Trade-off Documented**: Explains why positions are excluded

### 3.6 Implementation: lib.rs (Public API)

```rust
//! Source mapping for Oxur language
//!
//! This crate provides source mapping capabilities for tracking code
//! transformations through the Oxur compilation pipeline. It enables
//! rustc compiler errors to be translated back to the original Oxur
//! source code positions.
//!
//! # Architecture
//!
//! The source map tracks three transformation stages:
//!
//! 1. **Surface Forms** → **Core Forms** (macro expansion)
//! 2. **Core Forms** → **Rust AST** (lowering)
//! 3. **Rust errors** → **Original Source** (backward lookup)
//!
//! # Example
//!
//! ```
//! use oxur_smap::{SourceMap, new_node_id, SourcePos};
//!
//! let mut map = SourceMap::new();
//!
//! // Parser creates surface node
//! let surface = new_node_id();
//! let pos = SourcePos::repl(1, 5, 10);
//! map.record_surface_node(surface, pos);
//!
//! // Expander creates core node
//! let core = new_node_id();
//! map.record_expansion(surface, core);
//!
//! // Lowering creates rust node
//! let rust = new_node_id();
//! map.record_lowering(core, rust);
//!
//! // Error translator looks up original position
//! let original = map.lookup(&rust).unwrap();
//! assert_eq!(original.line, 1);
//! assert_eq!(original.column, 5);
//! ```

mod node_id;
mod source_pos;
mod source_map;
mod hash;

// Re-export public API
pub use node_id::{NodeId, NodeIdGenerator, new_node_id};
pub use source_pos::SourcePos;
pub use source_map::{SourceMap, SourceMapStats};
```

**Design Notes:**

- **Clean API**: Only exports what's needed
- **Example-Driven**: Shows common usage pattern
- **Documentation**: Explains architecture and purpose

---

## 4. Phase 1: Integration

**Goal:** Update dependent crates to use `oxur-smap` types.
**Duration:** 2-3 days
**Status:** Blocked by Phase 0 completion

### 4.1 Update oxur-lang

#### Step 1: Add Dependency

```toml
# oxur-lang/Cargo.toml
[dependencies]
oxur-smap = { path = "../oxur-smap" }
```

#### Step 2: Remove Old Stub

```bash
# Delete the outdated stub
rm oxur-lang/src/source_map.rs

# Update mod declarations in lib.rs
```

#### Step 3: Update Parser API

```rust
// oxur-lang/src/parser.rs

use oxur_smap::{SourceMap, new_node_id, SourcePos};

pub fn parse_lisp(
    code: &str,
    source_map: &mut SourceMap  // <-- NEW PARAMETER
) -> Result<Vec<SurfaceForm>, ParseError> {
    // For each parsed form, record its position
    let node = new_node_id();
    let pos = SourcePos::new(
        file.to_string(),
        line,
        column,
        length
    );
    source_map.record_surface_node(node, pos);

    // Build SurfaceForm with node attached
    SurfaceForm { node, /* ... */ }
}
```

#### Step 4: Update Expander API

```rust
// oxur-lang/src/expander.rs

use oxur_smap::{SourceMap, new_node_id};

pub fn expand(
    surface: &SurfaceForm,
    source_map: &mut SourceMap  // <-- NEW PARAMETER
) -> Result<CoreForm, ExpandError> {
    // Record expansion transformation
    let core_node = new_node_id();
    source_map.record_expansion(surface.node, core_node);

    // Build CoreForm with node attached
    CoreForm { node: core_node, /* ... */ }
}
```

### 4.2 Update oxur-comp

#### Step 1: Add Dependency

```toml
# oxur-comp/Cargo.toml
[dependencies]
oxur-smap = { path = "../oxur-smap" }
```

#### Step 2: Update Lowering API

```rust
// oxur-comp/src/lower.rs

use oxur_smap::{SourceMap, new_node_id};

pub fn lower(
    core: &CoreForm,
    source_map: &mut SourceMap  // <-- NEW PARAMETER
) -> Result<RustAst, LowerError> {
    // Record lowering transformation
    let rust_node = new_node_id();
    source_map.record_lowering(core.node, rust_node);

    // Build Rust AST with node embedded in comments
    // Example: /* oxur_node=300 */ fn add(x: i32, y: i32)
}
```

### 4.3 Update oxur-repl

#### Step 1: Add Dependency

```toml
# oxur-repl/Cargo.toml
[dependencies]
oxur-smap = { path = "../oxur-smap" }
```

#### Step 2: Integrate with CachedCompiler

```rust
// oxur-repl/src/compiler.rs

use std::sync::Arc;
use oxur_smap::SourceMap;

pub struct CachedCompiler {
    source_map: Arc<SourceMap>,  // Shared across pipeline
    // ... other fields
}

impl CachedCompiler {
    pub fn new() -> Self {
        Self {
            source_map: Arc::new(SourceMap::new()),
            // ...
        }
    }

    pub fn eval(&mut self, code: &str) -> Result<Value, EvalError> {
        // Thread source_map through entire pipeline
        let surface = oxur_lang::parse_lisp(code, &mut *self.source_map)?;
        let core = oxur_lang::expand(&surface, &mut *self.source_map)?;
        let rust_ast = oxur_comp::lower(&core, &mut *self.source_map)?;

        // ... compilation and execution

        // On error, use source_map to translate positions
    }
}
```

#### Step 3: Implement Error Translator

```rust
// oxur-repl/src/error_translator.rs

use oxur_smap::{SourceMap, NodeId};
use std::sync::Arc;

pub struct ErrorTranslator {
    source_map: Arc<SourceMap>,
}

impl ErrorTranslator {
    pub fn translate_rustc_error(
        &self,
        rustc_output: &str
    ) -> Result<OxurError, TranslateError> {
        // 1. Parse rustc JSON output
        // 2. Extract NodeId from comments (/* oxur_node=300 */)
        // 3. Lookup original position
        let rust_node = self.extract_node_id(rustc_output)?;
        let original_pos = self.source_map.lookup(&rust_node)
            .ok_or(TranslateError::MissingMapping)?;

        // 4. Format Oxur error message
        OxurError::new(original_pos, /* ... */)
    }

    fn extract_node_id(&self, rustc_output: &str) -> Result<NodeId, TranslateError> {
        // Regex: /\* oxur_node=(\d+) \*/
        // Parse and convert to NodeId
    }
}
```

---

## 5. Phase 2: Polish & Optimization

**Goal:** Address remaining gaps and optimize for production.
**Duration:** 1-2 days (optional for v1.0)
**Status:** Blocked by Phase 1 completion

### 5.1 Concurrency Model Resolution

**Decision Needed:** Should `SourceMap` be frozen after lowering?

**Option A: Immutable After Lowering**

```rust
pub struct SourceMap {
    // ... fields
    frozen: bool,
}

impl SourceMap {
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn record_lowering(&mut self, core: NodeId, rust: NodeId) {
        assert!(!self.frozen, "Cannot modify frozen SourceMap");
        self.core_to_rust.insert(core, rust);
    }
}
```

**Option B: Arc<RwLock<SourceMap>>**

```rust
use std::sync::{Arc, RwLock};

pub type SharedSourceMap = Arc<RwLock<SourceMap>>;
```

**Recommendation for v1.0:** Use Option A (frozen flag) for simplicity. The Arc is for sharing across threads during error translation, not for concurrent modification.

### 5.2 Performance Instrumentation

```rust
// Add to SourceMap
impl SourceMap {
    pub fn lookup_stats(&self) -> LookupStats {
        LookupStats {
            avg_chain_length: self.calculate_avg_chain_length(),
            max_chain_length: self.calculate_max_chain_length(),
        }
    }
}
```

### 5.3 Fuzzy Matching (Future)

**Deferred to v1.1+** - Document the requirement but don't implement yet.

```rust
impl SourceMap {
    /// Fuzzy lookup when exact NodeId match fails
    ///
    /// Uses heuristics like:
    /// - Nearest node within same line
    /// - Parent node in AST
    /// - First/last node in multi-line expression
    pub fn fuzzy_lookup(&self, rust_node: &NodeId) -> Option<SourcePos> {
        // TODO: Implement in v1.1
        None
    }
}
```

### 5.4 Serialization Support (Future)

**Deferred to v1.1+** - Add serde support for cache persistence.

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"], optional = true }

[features]
serialization = ["serde"]
```

---

## 6. Testing Strategy

### 6.1 Unit Tests (Per-Module)

**node_id.rs:**

- [x] Basic creation and equality
- [x] Generator increments correctly
- [x] Thread-safe generation (10 threads × 100 IDs)
- [x] No duplicate IDs across threads

**source_pos.rs:**

- [x] Basic construction
- [x] REPL constructor
- [x] Contains check
- [x] Panic on 0-indexed values

**source_map.rs:**

- [x] Empty map returns None
- [x] Record and retrieve surface nodes
- [x] Full transformation chain lookup
- [x] Broken chain returns None
- [x] Statistics collection

**hash.rs:**

- [x] Deterministic hashing
- [x] Position-independent hashing
- [x] Structure-sensitive hashing

### 6.2 Integration Tests

**tests/basic.rs:**

```rust
#[test]
fn test_repl_workflow() {
    // Simulate REPL: parse → expand → lower → error
    let mut map = SourceMap::new();

    // Parse phase
    let surface = new_node_id();
    map.record_surface_node(surface, SourcePos::repl(1, 1, 20));

    // Expansion phase
    let core = new_node_id();
    map.record_expansion(surface, core);

    // Lowering phase
    let rust = new_node_id();
    map.record_lowering(core, rust);

    // Error translation
    let pos = map.lookup(&rust).expect("Lookup failed");
    assert_eq!(pos.line, 1);
}
```

**tests/lookup.rs:**

```rust
#[test]
fn test_multiple_transforms() {
    // Test complex transformation graph
}

#[test]
fn test_missing_links() {
    // Test graceful degradation
}
```

**tests/integration.rs:**

```rust
#[test]
fn test_cache_key_generation() {
    // Test content_hash() for caching
}
```

### 6.3 Benchmark Tests (Optional)

```rust
// benches/lookup.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_lookup(c: &mut Criterion) {
    c.bench_function("lookup_1000_nodes", |b| {
        let map = build_large_source_map(1000);
        b.iter(|| {
            map.lookup(black_box(&NodeId::from_raw(1000)))
        });
    });
}
```

---

## 7. Implementation Checklist

### Phase 0: Core Implementation (2-3 days)

- [ ] Create `oxur-smap` crate structure
- [ ] Configure `Cargo.toml` (zero dependencies)
- [ ] Implement `NodeId` type
  - [ ] Basic type with u32 storage
  - [ ] Equality, Hash, Debug traits
  - [ ] Display trait
  - [ ] Unit tests
- [ ] Implement `NodeIdGenerator`
  - [ ] Atomic counter
  - [ ] Thread-safe generation
  - [ ] Global singleton
  - [ ] Unit tests + thread safety test
- [ ] Implement `SourcePos` type
  - [ ] Basic fields (file, line, column, length)
  - [ ] Constructor with validation (1-indexed)
  - [ ] REPL constructor
  - [ ] Contains check
  - [ ] Display trait
  - [ ] Unit tests
- [ ] Implement `SourceMap` type
  - [ ] Three HashMaps (surface_positions, surface_to_core, core_to_rust)
  - [ ] `record_surface_node()`
  - [ ] `record_expansion()`
  - [ ] `record_lowering()`
  - [ ] `lookup()` with backward traversal
  - [ ] Helper getters for testing
  - [ ] `stats()` method
  - [ ] Unit tests (empty, single record, full chain, broken chain)
- [ ] Implement `content_hash()`
  - [ ] Deterministic sorting
  - [ ] Structure-only (no positions)
  - [ ] Unit tests (deterministic, position-independent, structure-sensitive)
- [ ] Write `lib.rs` public API
  - [ ] Re-exports
  - [ ] Documentation
  - [ ] Usage examples
- [ ] Integration tests
  - [ ] REPL workflow simulation
  - [ ] Multiple transformations
  - [ ] Missing links
  - [ ] Cache key generation
- [ ] Documentation
  - [ ] API docs for all public items
  - [ ] Architecture overview
  - [ ] Usage examples
  - [ ] README.md

### Phase 1: Integration (2-3 days)

- [ ] Update `oxur-lang`
  - [ ] Add `oxur-smap` dependency
  - [ ] Remove old source_map.rs stub
  - [ ] Update parser API (`parse_lisp(&mut SourceMap)`)
  - [ ] Record surface nodes during parsing
  - [ ] Update expander API (`expand(&mut SourceMap)`)
  - [ ] Record expansions during macro expansion
  - [ ] Update tests
- [ ] Update `oxur-comp`
  - [ ] Add `oxur-smap` dependency
  - [ ] Update lowering API (`lower(&mut SourceMap)`)
  - [ ] Record lowering transformations
  - [ ] Embed NodeId in Rust comments
  - [ ] Update tests
- [ ] Update `oxur-repl`
  - [ ] Add `oxur-smap` dependency
  - [ ] Integrate SourceMap into CachedCompiler
  - [ ] Thread SourceMap through pipeline
  - [ ] Implement ErrorTranslator
  - [ ] Extract NodeId from rustc errors
  - [ ] Format Oxur error messages
  - [ ] Update tests

### Phase 2: Polish (1-2 days, optional for v1.0)

- [ ] Resolve concurrency model
  - [ ] Decide on frozen flag vs RwLock
  - [ ] Document decision
  - [ ] Implement if needed
- [ ] Performance instrumentation
  - [ ] Add lookup_stats() method
  - [ ] Measure chain lengths
  - [ ] Profile memory overhead
- [ ] Benchmarks
  - [ ] Lookup performance
  - [ ] Hash generation
  - [ ] Memory usage
- [ ] Documentation polish
  - [ ] Design decisions documented
  - [ ] Trade-offs explained
  - [ ] Future work identified

---

## 8. Open Design Decisions

### 8.1 NodeId Generation Strategy

**Status:** Decided - Single global atomic counter

**Rationale:**

- Simplest implementation
- No risk of range exhaustion
- Thread-safe without coordination
- NodeIds are opaque (internal structure doesn't matter)

**Alternative Considered:** Per-stage ranges (100-199 surface, 200-299 core, etc.)

- **Pros:** More debuggable NodeIds
- **Cons:** Risk of range exhaustion, requires coordination
- **Verdict:** Can add stage tagging later without breaking API if needed

### 8.2 Concurrency Model

**Status:** Open - Needs decision before Phase 2

**Options:**

**A. Frozen After Lowering** (Recommended for v1.0)

```rust
pub fn freeze(&mut self) {
    self.frozen = true;
}
```

- **Pros:** Simple, catches bugs
- **Cons:** Requires freezing step

**B. Arc<RwLock<SourceMap>>**

- **Pros:** Allows concurrent modification
- **Cons:** More complex, runtime overhead

**Recommendation:** Use Option A for v1.0. Lowering is the final transformation, so freezing makes sense.

### 8.3 Fuzzy Matching Algorithm

**Status:** Deferred to v1.1+

**Requirement:** Handle cases where exact NodeId match fails.

**Potential Heuristics:**

- Nearest node on same line
- Parent node in AST
- First/last node in multi-line expression

**Decision:** Document requirement but don't implement in v1.0. Wait for real-world error patterns.

### 8.4 Serialization Format

**Status:** Deferred to v1.1+

**Requirement:** Serialize SourceMap for cache persistence.

**Options:**

- JSON (human-readable, large)
- Bincode (compact, fast)
- Postcard (very compact, requires schema)

**Decision:** Add serde support in v1.1 when caching is stable.

---

## 9. Success Criteria

### 9.1 Phase 0 Complete When

- ✅ All unit tests pass
- ✅ Integration tests simulate full pipeline
- ✅ Zero compiler warnings
- ✅ Documentation complete with examples
- ✅ Thread-safety verified
- ✅ `cargo test` passes
- ✅ `cargo clippy` clean
- ✅ `cargo doc --open` builds cleanly

### 9.2 Phase 1 Complete When

- ✅ All dependent crates use `oxur-smap` types
- ✅ Old stub code removed
- ✅ Pipeline threads SourceMap correctly
- ✅ Error translator extracts NodeIds
- ✅ End-to-end test: Oxur code → Rust error → Original position
- ✅ All tests pass across all crates

### 9.3 Production Ready When

- ✅ Rustc errors correctly map to Oxur source
- ✅ Error messages match quality of rustc
- ✅ Cache keys work correctly (no false negatives)
- ✅ Performance acceptable (<1ms lookup)
- ✅ Memory overhead reasonable (<100MB for large session)
- ✅ Documentation complete
- ✅ No known bugs

---

## Appendix A: API Reference

### Public API Surface

```rust
// Types
pub struct NodeId(u32);
pub struct SourcePos { /* ... */ }
pub struct SourceMap { /* ... */ }
pub struct SourceMapStats { /* ... */ }
pub struct NodeIdGenerator { /* ... */ }

// Functions
pub fn new_node_id() -> NodeId;

// Methods on SourceMap
impl SourceMap {
    pub fn new() -> Self;
    pub fn record_surface_node(&mut self, node: NodeId, pos: SourcePos);
    pub fn record_expansion(&mut self, surface: NodeId, core: NodeId);
    pub fn record_lowering(&mut self, core: NodeId, rust: NodeId);
    pub fn lookup(&self, rust_node: &NodeId) -> Option<SourcePos>;
    pub fn content_hash(&self) -> u64;
    pub fn stats(&self) -> SourceMapStats;
}

// Methods on SourcePos
impl SourcePos {
    pub fn new(file: String, line: u32, column: u32, length: u32) -> Self;
    pub fn repl(line: u32, column: u32, length: u32) -> Self;
    pub fn end_column(&self) -> u32;
    pub fn contains(&self, other: &SourcePos) -> bool;
}

// Methods on NodeId
impl NodeId {
    pub const fn from_raw(id: u32) -> Self;
    pub const fn as_raw(&self) -> u32;
}
```

---

## Appendix B: Timeline

**Phase 0: Core Implementation**

- Day 1: NodeId, SourcePos types + tests
- Day 2: SourceMap implementation + tests
- Day 3: Hash, integration tests, documentation

**Phase 1: Integration**

- Day 4: oxur-lang integration
- Day 5: oxur-comp integration + error translator
- Day 6: oxur-repl integration + end-to-end testing

**Phase 2: Polish** (Optional)

- Day 7: Concurrency model + performance
- Day 8: Documentation polish + benchmarks

**Total:** 6-8 days to production-ready

---

## Appendix C: Dependencies

### Upstream (none - foundation crate)

- No dependencies

### Downstream (crates that depend on oxur-smap)

- oxur-lang (parser, expander)
- oxur-comp (lowering)
- oxur-repl (error translation, caching)
- oxur-ast (future - for IDE integration)

---

## Appendix D: References

**Primary Sources:**

- ODD-0038: Oxur REPL Architecture (v1.1, 2026-01-05) - Definitive specification
- ODD-0030: Oxur REPL Implementation Specification - Error translator details
- ODD-0026: Oxur REPL Evaluation Strategy - Strategic context
- ODD-0001: Oxur Letter of Intent - Vision and goals

**Supporting Research:**

- source-map-analysis.md - Claude Code report on existing mentions
- evcxr research - Validation of architecture decisions

---

**Document Status:** Complete - Ready for implementation
**Next Action:** Begin Phase 0 core implementation
**Owner:** TBD (waiting for assignment)
