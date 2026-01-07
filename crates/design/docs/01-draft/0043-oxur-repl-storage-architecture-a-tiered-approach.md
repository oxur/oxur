---
number: 43
title: "Oxur REPL Storage Architecture: A Tiered Approach"
author: "analysis of"
component: All
tags: [change-me]
created: 2026-01-06
updated: 2026-01-06
state: Draft
supersedes: null
superseded-by: null
version: 1.0
---

# Oxur REPL Storage Architecture: A Tiered Approach

## Document Purpose

This document provides a detailed design for Oxur's REPL storage subsystem, addressing:

- **Performance:** Sub-microsecond access for hot paths (variable lookup, completion)
- **Persistence:** Crash recovery and session restore
- **Queryability:** SQL access for history search, analytics, and tooling
- **Resilience:** Graceful degradation and data integrity

The design is informed by analysis of Erlang's ETS (Erlang Term Storage), SQLite usage patterns in modern terminal tools, and the specific access patterns of interactive REPL development.

**Related Documents:**

- ODD-0038: Oxur REPL Architecture Overview (execution engine)
- ODD-0041: Terminal UX Research (UI requirements driving storage needs)
- ODD-0042: Towards a Rust VM (conceptual context)

---

## Table of Contents

1. [Motivation and Requirements](#1-motivation-and-requirements)
2. [Storage Requirements Analysis](#2-storage-requirements-analysis)
3. [The Case Against SQLite-Everywhere](#3-the-case-against-sqlite-everywhere)
4. [The Case For SQLite-Where-Appropriate](#4-the-case-for-sqlite-where-appropriate)
5. [Tiered Storage Architecture](#5-tiered-storage-architecture)
6. [Tier 1: Hot Storage (In-Memory)](#6-tier-1-hot-storage-in-memory)
7. [Tier 2: Warm Storage (Persistent KV)](#7-tier-2-warm-storage-persistent-kv)
8. [Tier 3: Cold Storage (SQLite)](#8-tier-3-cold-storage-sqlite)
9. [Tier 4: Archive Storage (Filesystem)](#9-tier-4-archive-storage-filesystem)
10. [Cross-Tier Operations](#10-cross-tier-operations)
11. [Implementation Strategy](#11-implementation-strategy)
12. [API Design](#12-api-design)
13. [Performance Targets](#13-performance-targets)
14. [Migration and Evolution](#14-migration-and-evolution)

---

## 1. Motivation and Requirements

### 1.1 The Storage Problem

The Oxur REPL needs to manage several categories of data with vastly different access patterns:

| Data | Access Pattern | Speed Need | Durability Need |
|------|----------------|------------|-----------------|
| Current variables | Read every codegen | Critical | Session |
| Symbol table | Read every keystroke | Critical | Session |
| Recent history | Append + recent-N | Important | Persistent |
| Full history | Search, analytics | Moderate | Persistent |
| Artifact cache | Content-lookup | Important | Persistent |
| Preferences | Read on startup | Low | Persistent |

Using a single storage solution for all of these creates unnecessary trade-offs.

### 1.2 Lessons from Erlang/OTP

Erlang provides multiple storage mechanisms for different needs:

| Erlang Mechanism | Use Case | Oxur Equivalent |
|------------------|----------|-----------------|
| **Process dictionary** | Fast per-process state | Struct fields |
| **ETS** | Shared concurrent tables | Tier 1 (DashMap) |
| **DETS** | Disk-backed ETS | Tier 2 (redb) |
| **Mnesia** | Distributed database | Tier 3 (SQLite) |

The BEAM gives you ETS "for free" with O(1) concurrent reads. We need to build equivalent capability, but we can optimize for exactly our access patterns.

### 1.3 Lessons from Modern Terminal Tools

ODD-0041 research revealed SQLite usage in several exemplary tools:

- **Nushell:** History with rich metadata
- **llm CLI:** Interaction logging and cost tracking
- **Various tools:** Configuration, caching, search indexes

However, none of these use SQLite for hot-path data. The pattern is:

- **Hot path:** In-memory data structures
- **Cold path:** SQLite for persistence, search, export

### 1.4 Design Principles

1. **Match storage to access pattern** - Don't use a database for O(1) lookups
2. **Separate query interface from storage** - Provide SQL without paying SQL cost
3. **Graceful degradation** - Tier 2/3 failures shouldn't break Tier 1
4. **Progressive persistence** - Fast path first, durability async
5. **Observable** - Easy to inspect, debug, export

---

## 2. Storage Requirements Analysis

### 2.1 Data Categories

#### Session Variables (Hot)

```rust
// Access pattern: get by name during code generation
// Frequency: 10-100 times per eval
// Size: Small to medium (most values < 1KB)
// Lifetime: Current session only

struct VariableAccess {
    pattern: "key-value lookup",
    frequency: "every eval, multiple times",
    latency_budget: "< 1μs",
    consistency: "read-your-writes",
}
```

#### Symbol Table (Hot)

```rust
// Access pattern: prefix search for completion
// Frequency: Every keystroke during typing
// Size: Small (name + type + metadata per symbol)
// Lifetime: Current session, reconstructible

struct SymbolAccess {
    pattern: "prefix search, enumeration",
    frequency: "every keystroke",
    latency_budget: "< 100μs for completion",
    consistency: "eventually consistent OK",
}
```

#### Command History (Warm → Cold)

```rust
// Access pattern: append, recent-N, full-text search
// Frequency: Every command (append), occasional (search)
// Size: Growing unbounded
// Lifetime: Persistent across sessions

struct HistoryAccess {
    pattern: "append-mostly, search",
    frequency: "every command + occasional search",
    latency_budget: "< 1ms append, < 100ms search",
    consistency: "durable after response",
}
```

#### Artifact Cache (Cold)

```rust
// Access pattern: content-addressed lookup
// Frequency: Once per eval (cache check)
// Size: Large (compiled .so files, MB each)
// Lifetime: Persistent, LRU eviction

struct ArtifactAccess {
    pattern: "content-addressed, LRU",
    frequency: "once per eval",
    latency_budget: "< 10ms",
    consistency: "immutable once written",
}
```

### 2.2 Access Pattern Summary

```
                    Frequency
                       ↑
                       │
        Symbol Table ● │ ● Variables
       (completion)    │   (codegen)
                       │
                       │
                       │      ● Recent History
                       │        (display)
                       │
                       │
           ● Full History        ● Artifacts
             (search)              (cache check)
                       │
                       └──────────────────────→ Size
                      Small              Large
```

---

## 3. The Case Against SQLite-Everywhere

### 3.1 Performance Reality

SQLite is excellent, but it has inherent overhead:

```rust
// Benchmark: Variable lookup

// HashMap: ~10-50ns
let value = variables.get("x").unwrap();

// DashMap (concurrent): ~20-100ns
let value = variables.get("x").unwrap();

// SQLite (best case, prepared statement, WAL mode): ~1-10μs
let value: Value = stmt.query_row(params![name], |row| row.get(0))?;

// That's 100-1000x difference!
```

For variable lookup during code generation (potentially 100+ times per eval), this adds up:

| Storage | Per-lookup | 100 lookups | Impact |
|---------|------------|-------------|--------|
| HashMap | 50ns | 5μs | Negligible |
| DashMap | 100ns | 10μs | Negligible |
| SQLite | 5μs | 500μs | Noticeable |

### 3.2 Completion Latency

For tab completion, we need prefix search on every keystroke:

```rust
// Trie-based prefix search: O(k) where k = prefix length
// Typical: ~100ns - 1μs
let completions = symbol_trie.prefix_search("foo");

// SQLite LIKE query: O(n) scan or index lookup
// Typical: ~100μs - 1ms
let completions: Vec<String> = stmt
    .query_map(params![format!("{}%", prefix)], |row| row.get(0))?
    .collect();
```

At 100ms between keystrokes (fast typing), 1ms for completion is 1% of the budget. At 50ms (very fast), it's 2%. With a trie, it's 0.001%.

### 3.3 Type System Mismatch

SQLite types don't map cleanly to Rust or Lisp types:

```
SQL Types:    INTEGER, REAL, TEXT, BLOB, NULL
Rust Types:   i32, i64, f32, f64, String, Vec<u8>, Option<T>, structs, enums, ...
Lisp Types:   integers, floats, symbols, strings, lists, vectors, maps, ...
```

Every access requires serialization/deserialization, which has both CPU and cognitive overhead.

### 3.4 When SQLite Hurts

- ❌ **Hot path lookups** - Variable access, type checking
- ❌ **Real-time completion** - Every keystroke matters
- ❌ **Streaming data** - SQLite transactions have overhead
- ❌ **Type-rich data** - Serialization cost

---

## 4. The Case For SQLite-Where-Appropriate

### 4.1 Where SQLite Excels

SQLite is genuinely excellent for:

- ✅ **Complex queries** - "Show commands from last week with errors"
- ✅ **Full-text search** - FTS5 is very good
- ✅ **Durability** - ACID, crash-safe, battle-tested
- ✅ **Interoperability** - Users can inspect with standard tools
- ✅ **Schema evolution** - Migrations are well-understood
- ✅ **Aggregation** - COUNT, SUM, GROUP BY, etc.

### 4.2 History Search Example

```sql
-- Find all commands matching a pattern from last week
SELECT input, output_preview, timestamp
FROM history
WHERE input MATCH 'defn*'
  AND timestamp > datetime('now', '-7 days')
ORDER BY timestamp DESC
LIMIT 20;

-- This is much better expressed in SQL than in Rust
```

### 4.3 Analytics and Export

```sql
-- Command frequency analysis
SELECT
    substr(input, 1, instr(input, ' ')-1) as command,
    COUNT(*) as frequency,
    AVG(duration_ms) as avg_duration
FROM history
GROUP BY command
ORDER BY frequency DESC;

-- Export to CSV
.mode csv
.output history_export.csv
SELECT * FROM history WHERE session_id = 'abc123';
```

### 4.4 When SQLite Wins

✅ **History search** - Full-text, date ranges, complex filters
✅ **Analytics** - Aggregation, reporting
✅ **Export** - Standard format, easy tooling
✅ **Persistence** - Long-term storage
✅ **User inspection** - `sqlite3 ~/.oxur/history.db`

---

## 5. Tiered Storage Architecture

### 5.1 Overview

```
┌────────────────────────────────────────────────────────────────┐
│                     TIER 1: HOT (In-Memory)                    │
│                                                                │
│  ┌─────────────────────┐  ┌─────────────────────┐              │
│  │   VariableStore     │  │    SymbolIndex      │              │
│  │   (DashMap)         │  │    (Trie + Map)     │              │
│  │                     │  │                     │              │
│  │   - Current session │  │   - All defined     │              │
│  │   - Lock-free read  │  │     symbols         │              │
│  │   - Type-aware      │  │   - O(k) prefix     │              │
│  └─────────────────────┘  │     completion      │              │
│                           └─────────────────────┘              │
│                                                                │
│  Access: <100ns           Durability: None (volatile)          │
│  Use: Every eval, every keystroke                              │
└────────────────────────────────────────────────────────────────┘
                              │
                              │ Async checkpoint (batch, non-blocking)
                              ↓
┌────────────────────────────────────────────────────────────────┐
│                    TIER 2: WARM (Memory-Mapped KV)             │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    redb Database                         │  │
│  │                                                          │  │
│  │   Tables:                                                │  │
│  │   - session_snapshots: Session state for recovery        │  │
│  │   - recent_history: Last N commands (ring buffer)        │  │
│  │   - symbol_definitions: For session restore              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│  Access: 1-10μs           Durability: Crash-safe (fsync)       │
│  Use: Recovery, recent queries, session restore                │
└────────────────────────────────────────────────────────────────┘
                              │
                              │ Background sync (periodic, batched)
                              ↓
┌────────────────────────────────────────────────────────────────┐
│                    TIER 3: COLD (SQLite)                       │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    SQLite Database                       │  │
│  │                    ~/.local/share/oxur/oxur.db           │  │
│  │                                                          │  │
│  │   Tables:                                                │  │
│  │   - history: Full command history with FTS5              │  │
│  │   - sessions: Session metadata                           │  │
│  │   - preferences: User settings                           │  │
│  │   - analytics: Aggregated statistics                     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│  Access: 100μs-10ms       Durability: Full ACID                │
│  Use: Search, analytics, export, user queries                  │
└────────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌────────────────────────────────────────────────────────────────┐
│                    TIER 4: ARCHIVE (Filesystem)                │
│                                                                │
│  ~/.cache/oxur/artifacts/   - Compiled .so files (by hash)     │
│  ~/.local/share/oxur/exports/ - User exports                   │
│  ~/.local/share/oxur/logs/    - Debug logs                     │
│                                                                │
│  Access: 1-100ms          Durability: Filesystem               │
│  Use: Artifact cache, large exports, diagnostics               │
└────────────────────────────────────────────────────────────────┘
```

### 5.2 Data Flow

```
User types: (defn square [x] (* x x))
                │
                ↓
┌───────────────────────────────────────────────────────┐
│ 1. Parse & Evaluate                                   │
│    - Read variables from Tier 1 (VariableStore)       │
│    - Type inference uses Tier 1 (SymbolIndex)         │
└───────────────────────────────────────────────────────┘
                │
                ↓
┌───────────────────────────────────────────────────────┐
│ 2. Update Hot State (synchronous)                     │
│    - Write new symbol to Tier 1 SymbolIndex           │
│    - Update VariableStore if applicable               │
└───────────────────────────────────────────────────────┘
                │
                ↓
┌───────────────────────────────────────────────────────┐
│ 3. Return Result to User                              │
│    (Hot path complete - user sees response)           │
└───────────────────────────────────────────────────────┘
                │
                ↓ (async, non-blocking)
┌───────────────────────────────────────────────────────┐
│ 4. Background Persistence                             │
│    a. Checkpoint to Tier 2 (redb) - crash recovery    │
│    b. Append to Tier 3 (SQLite) - history             │
└───────────────────────────────────────────────────────┘
```

---

## 6. Tier 1: Hot Storage (In-Memory)

### 6.1 VariableStore

**Purpose:** Store current session variables with type information

**Implementation:**

```rust
use dashmap::DashMap;
use std::any::{Any, TypeId};

/// Type-aware variable storage with concurrent access
pub struct VariableStore {
    /// Primary storage: name → boxed value
    values: DashMap<String, StoredValue>,

    /// Type registry for display/serialization
    types: DashMap<String, TypeInfo>,
}

struct StoredValue {
    value: Box<dyn Any + Send + Sync>,
    type_id: TypeId,
}

#[derive(Clone)]
pub struct TypeInfo {
    pub name: String,        // "Vec<i32>"
    pub display_hint: DisplayHint,
    pub serializable: bool,
}

#[derive(Clone)]
pub enum DisplayHint {
    Primitive,
    List { element_type: Box<TypeInfo> },
    Map { key_type: Box<TypeInfo>, value_type: Box<TypeInfo> },
    Table { columns: Vec<String> },
    Opaque,
}

impl VariableStore {
    /// Get variable with zero-copy reference
    /// Returns None if not found or type mismatch
    pub fn get<T: 'static>(&self, name: &str) -> Option<dashmap::mapref::one::Ref<'_, String, StoredValue>> {
        self.values.get(name).filter(|v| v.type_id == TypeId::of::<T>())
    }

    /// Set variable with type tracking
    pub fn set<T: Any + Send + Sync + 'static>(&self, name: String, value: T, type_info: TypeInfo) {
        self.values.insert(name.clone(), StoredValue {
            value: Box::new(value),
            type_id: TypeId::of::<T>(),
        });
        self.types.insert(name, type_info);
    }

    /// List all variable names (for completion)
    pub fn names(&self) -> Vec<String> {
        self.values.iter().map(|r| r.key().clone()).collect()
    }

    /// Get type info (for display, serialization)
    pub fn type_of(&self, name: &str) -> Option<TypeInfo> {
        self.types.get(name).map(|r| r.clone())
    }
}
```

**Performance:**

| Operation | Complexity | Typical Latency |
|-----------|------------|-----------------|
| `get` | O(1) | 20-100ns |
| `set` | O(1) | 50-200ns |
| `names` | O(n) | 1-10μs |
| `type_of` | O(1) | 20-100ns |

### 6.2 SymbolIndex

**Purpose:** Fast symbol lookup and prefix-based completion

**Implementation:**

```rust
use std::collections::HashMap;
use radix_trie::{Trie, TrieCommon};

/// Symbol table with O(k) prefix search
pub struct SymbolIndex {
    /// Full symbol information
    symbols: HashMap<String, SymbolInfo>,

    /// Trie for prefix completion
    prefix_trie: Trie<String, ()>,

    /// By-kind index for filtered completion
    by_kind: HashMap<SymbolKind, Vec<String>>,
}

#[derive(Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub type_signature: String,  // "(fn [x: i32] -> i32)"
    pub doc: Option<String>,
    pub defined_at: Option<SourcePos>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Variable,
    Function,
    Macro,
    Type,
    Module,
}

impl SymbolIndex {
    /// O(k) prefix search where k = prefix length
    pub fn complete(&self, prefix: &str) -> Vec<&SymbolInfo> {
        self.prefix_trie
            .get_raw_descendant(prefix)
            .map(|subtrie| {
                subtrie.keys()
                    .filter_map(|name| self.symbols.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// O(k) prefix search filtered by kind
    pub fn complete_filtered(&self, prefix: &str, kind: SymbolKind) -> Vec<&SymbolInfo> {
        self.complete(prefix)
            .into_iter()
            .filter(|s| s.kind == kind)
            .collect()
    }

    /// O(1) exact lookup
    pub fn get(&self, name: &str) -> Option<&SymbolInfo> {
        self.symbols.get(name)
    }

    /// Add or update symbol
    pub fn insert(&mut self, info: SymbolInfo) {
        let name = info.name.clone();
        let kind = info.kind;

        // Update main storage
        self.symbols.insert(name.clone(), info);

        // Update trie
        self.prefix_trie.insert(name.clone(), ());

        // Update kind index
        self.by_kind.entry(kind).or_default().push(name);
    }
}
```

**Performance:**

| Operation | Complexity | Typical Latency |
|-----------|------------|-----------------|
| `get` | O(1) | 20-50ns |
| `complete` | O(k + m) | 100ns-1μs |
| `insert` | O(k) | 100-500ns |

Where k = prefix/key length, m = number of matches.

### 6.3 RecentHistory (Ring Buffer)

**Purpose:** Fast access to most recent commands for display and suggestions

**Implementation:**

```rust
use std::collections::VecDeque;

/// Bounded recent history with O(1) operations
pub struct RecentHistory {
    entries: VecDeque<HistoryEntry>,
    capacity: usize,
}

#[derive(Clone)]
pub struct HistoryEntry {
    pub id: u64,
    pub input: String,
    pub output_preview: String,  // Truncated for display
    pub output_type: OutputType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Clone)]
pub enum OutputType {
    Value,
    Table { columns: usize, rows: usize },
    Error,
    Definition,
    Empty,
}

impl RecentHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// O(1) append
    pub fn push(&mut self, entry: HistoryEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// O(1) access to last N
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.iter().rev().take(n)
    }

    /// O(n) search (for small n, this is fine)
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        self.entries.iter()
            .filter(|e| e.input.contains(query))
            .collect()
    }
}
```

**Capacity:** 1000 entries (configurable)

**Memory:** ~100KB for 1000 entries with 100-byte average

---

## 7. Tier 2: Warm Storage (Persistent KV)

### 7.1 Technology Choice: redb

**Why redb over sled:**

| Aspect | redb | sled |
|--------|------|------|
| Maintenance | Active | Uncertain |
| Complexity | Simple | Complex (LSM) |
| Transactions | ACID | ACID |
| API | Clean | Clean |
| Pure Rust | Yes | Yes |

### 7.2 Schema Design

```rust
use redb::{Database, TableDefinition, ReadableTable, WriteTransaction};

// Table definitions
const SESSIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("sessions");
const SNAPSHOTS: TableDefinition<&str, &[u8]> = TableDefinition::new("snapshots");
const RECENT: TableDefinition<u64, &[u8]> = TableDefinition::new("recent");

pub struct WarmStorage {
    db: Database,
}

impl WarmStorage {
    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::create(path)?;
        Ok(Self { db })
    }

    /// Checkpoint session state for crash recovery
    pub fn checkpoint_session(&self, session_id: &str, state: &SessionSnapshot) -> Result<()> {
        let encoded = bincode::serialize(state)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(SNAPSHOTS)?;
            table.insert(session_id, encoded.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Restore session from checkpoint
    pub fn restore_session(&self, session_id: &str) -> Result<Option<SessionSnapshot>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(SNAPSHOTS)?;
        match table.get(session_id)? {
            Some(data) => Ok(Some(bincode::deserialize(data.value())?)),
            None => Ok(None),
        }
    }

    /// Append to recent history (for fast recent queries)
    pub fn append_history(&self, entry: &HistoryEntry) -> Result<()> {
        let encoded = bincode::serialize(entry)?;
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(RECENT)?;
            table.insert(entry.id, encoded.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: String,
    pub variables: Vec<(String, SerializedValue)>,
    pub symbols: Vec<SymbolInfo>,
    pub history_ids: Vec<u64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

### 7.3 Sync Strategy

```rust
impl WarmStorage {
    /// Background checkpoint - non-blocking
    pub async fn background_checkpoint(
        &self,
        session_id: &str,
        state: SessionSnapshot,
    ) {
        // Spawn on background task pool
        tokio::spawn(async move {
            if let Err(e) = self.checkpoint_session(session_id, &state) {
                log::warn!("Checkpoint failed: {}", e);
                // Don't panic - Tier 1 still has data
            }
        });
    }
}
```

---

## 8. Tier 3: Cold Storage (SQLite)

### 8.1 Schema Design

```sql
-- ~/.local/share/oxur/oxur.db

-- Full command history with FTS5 for search
CREATE TABLE history (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    input TEXT NOT NULL,
    output_preview TEXT,
    output_type TEXT NOT NULL,  -- 'value', 'table', 'error', 'definition'
    success INTEGER NOT NULL,
    duration_ms INTEGER,
    timestamp TEXT NOT NULL,    -- ISO 8601
    context_hash TEXT,          -- For grouping related commands

    -- Metadata for analytics
    input_length INTEGER,
    output_length INTEGER,
    variables_used TEXT,        -- JSON array of variable names
    functions_called TEXT       -- JSON array of function names
);

-- FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE history_fts USING fts5(
    input,
    output_preview,
    content='history',
    content_rowid='id'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER history_ai AFTER INSERT ON history BEGIN
    INSERT INTO history_fts(rowid, input, output_preview)
    VALUES (new.id, new.input, new.output_preview);
END;

-- Session metadata
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    last_active TEXT NOT NULL,
    command_count INTEGER DEFAULT 0,
    mode TEXT DEFAULT 'lisp',   -- 'lisp' or 'sexpr'
    metadata TEXT               -- JSON for extensibility
);

-- User preferences
CREATE TABLE preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Indexes for common queries
CREATE INDEX idx_history_session ON history(session_id);
CREATE INDEX idx_history_timestamp ON history(timestamp);
CREATE INDEX idx_history_success ON history(success);
```

### 8.2 Query Examples

```rust
use rusqlite::{Connection, params};

pub struct ColdStorage {
    conn: Connection,
}

impl ColdStorage {
    /// Full-text search with ranking
    pub fn search_history(&self, query: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(r#"
            SELECT h.*, bm25(history_fts) as rank
            FROM history h
            JOIN history_fts ON h.id = history_fts.rowid
            WHERE history_fts MATCH ?1
            ORDER BY rank
            LIMIT ?2
        "#)?;

        stmt.query_map(params![query, limit], |row| {
            // Map to HistoryEntry
        })?.collect()
    }

    /// Recent history with date filter
    pub fn recent_history(
        &self,
        session_id: Option<&str>,
        since: Option<chrono::DateTime<chrono::Utc>>,
        limit: usize,
    ) -> Result<Vec<HistoryEntry>> {
        let mut sql = String::from("SELECT * FROM history WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(sid) = session_id {
            sql.push_str(" AND session_id = ?");
            params.push(Box::new(sid.to_string()));
        }

        if let Some(ts) = since {
            sql.push_str(" AND timestamp > ?");
            params.push(Box::new(ts.to_rfc3339()));
        }

        sql.push_str(" ORDER BY timestamp DESC LIMIT ?");
        params.push(Box::new(limit as i64));

        // Execute query...
    }

    /// Analytics: Command frequency
    pub fn command_frequency(&self, days: u32) -> Result<Vec<(String, u64)>> {
        let mut stmt = self.conn.prepare(r#"
            SELECT
                substr(input, 1, instr(input || ' ', ' ') - 1) as command,
                COUNT(*) as frequency
            FROM history
            WHERE timestamp > datetime('now', '-' || ?1 || ' days')
            GROUP BY command
            ORDER BY frequency DESC
            LIMIT 20
        "#)?;

        stmt.query_map(params![days], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?.collect()
    }
}
```

### 8.3 SQL as Query Interface

The key insight: **SQL is a *view* into the data, not the source of truth for hot paths.**

```rust
impl OxurStore {
    /// Materialize current state to SQLite for querying
    pub fn materialize_for_query(&self) -> Result<()> {
        // 1. Flush Tier 1 recent history to Tier 3
        let recent = self.tier1.recent_history.entries();
        self.tier3.batch_insert_history(&recent)?;

        // 2. Now user can query with full SQL
        Ok(())
    }

    /// Execute user SQL query
    pub fn query(&self, sql: &str) -> Result<QueryResult> {
        // Ensure data is materialized
        self.materialize_for_query()?;

        // Execute against SQLite
        self.tier3.execute_query(sql)
    }
}
```

---

## 9. Tier 4: Archive Storage (Filesystem)

### 9.1 Artifact Cache

**Location:** `~/.cache/oxur/artifacts/`

**Structure:**

```
~/.cache/oxur/artifacts/
├── a1b2c3d4e5f6...89.so      # SHA256 hash of content
├── b2c3d4e5f6g7...90.so
├── index.json                 # Metadata index
└── ...
```

**Implementation:** See ODD-0038 Section 10.5 for ArtifactCache design.

### 9.2 Exports and Logs

**Location:** `~/.local/share/oxur/`

```
~/.local/share/oxur/
├── oxur.db                    # SQLite (Tier 3)
├── exports/
│   ├── history-2026-01-06.csv
│   └── session-abc123.json
└── logs/
    └── debug-2026-01-06.log
```

---

## 10. Cross-Tier Operations

### 10.1 Write Path

```rust
impl OxurStore {
    /// Record a completed evaluation
    pub async fn record_eval(&self, eval: EvalRecord) {
        // 1. Update Tier 1 (synchronous, fast)
        self.tier1.variables.set(/* ... */);
        self.tier1.symbols.insert(/* ... */);
        self.tier1.recent.push(eval.to_history_entry());

        // 2. Return to user immediately
        // (Persistence happens async below)

        // 3. Checkpoint to Tier 2 (async, non-blocking)
        let snapshot = self.tier1.snapshot();
        self.tier2.background_checkpoint(&self.session_id, snapshot).await;

        // 4. Persist to Tier 3 (async, batched)
        self.tier3_buffer.push(eval.to_history_entry());
        if self.tier3_buffer.len() >= BATCH_SIZE || self.tier3_timer.elapsed() {
            self.flush_to_tier3().await;
        }
    }
}
```

### 10.2 Read Path

```rust
impl OxurStore {
    /// Get variable (hot path)
    pub fn get_variable<T: 'static>(&self, name: &str) -> Option</* ... */> {
        // Always Tier 1 - no fallback needed
        self.tier1.variables.get::<T>(name)
    }

    /// Get completions (hot path)
    pub fn complete(&self, prefix: &str) -> Vec<CompletionItem> {
        // Always Tier 1
        self.tier1.symbols.complete(prefix)
            .map(|s| s.to_completion_item())
            .collect()
    }

    /// Search history (cold path)
    pub fn search_history(&self, query: &str) -> Result<Vec<HistoryEntry>> {
        // Flush recent to ensure complete results
        self.flush_to_tier3().await?;

        // Query Tier 3
        self.tier3.search_history(query, 100)
    }

    /// Recent history (warm path)
    pub fn recent_history(&self, n: usize) -> Vec<HistoryEntry> {
        if n <= self.tier1.recent.len() {
            // Tier 1 has enough
            self.tier1.recent.recent(n).cloned().collect()
        } else {
            // Need Tier 2 or 3
            // ...
        }
    }
}
```

### 10.3 Recovery Path

```rust
impl OxurStore {
    /// Restore session after crash
    pub fn restore_session(session_id: &str) -> Result<Self> {
        // 1. Try Tier 2 snapshot (most recent)
        if let Some(snapshot) = tier2.restore_session(session_id)? {
            return Self::from_snapshot(snapshot);
        }

        // 2. Fall back to Tier 3 (rebuild from history)
        let history = tier3.session_history(session_id)?;
        Self::replay_history(history)
    }
}
```

---

## 11. Implementation Strategy

### 11.1 Phase 1: Basic Tiers (v1.0)

**Implement:**

- Tier 1: VariableStore (DashMap), SymbolIndex (HashMap + Vec, trie later)
- Tier 3: SQLite for history (simple schema)
- Tier 4: Artifact cache (existing from ODD-0038)

**Skip for now:**

- Tier 2 (redb) - add when crash recovery is prioritized
- FTS5 - add when search is prioritized
- Trie - use simple prefix filter initially

### 11.2 Phase 2: Rich Features (v1.1)

**Add:**

- Tier 2: redb for crash recovery
- SymbolIndex with trie for O(k) completion
- FTS5 for history search
- Background sync between tiers

### 11.3 Phase 3: Query Interface (v1.2)

**Add:**

- User-facing SQL query capability
- Export functionality
- Analytics views

---

## 12. API Design

### 12.1 Public Interface

```rust
/// Main storage interface
pub struct OxurStore {
    session_id: String,
    tier1: HotStorage,
    tier2: Option<WarmStorage>,
    tier3: ColdStorage,
}

impl OxurStore {
    // === Hot Path (Tier 1) ===

    pub fn get_variable<T: 'static>(&self, name: &str) -> Option<Ref<T>>;
    pub fn set_variable<T: 'static>(&self, name: String, value: T, type_info: TypeInfo);
    pub fn complete(&self, prefix: &str) -> Vec<CompletionItem>;
    pub fn get_symbol(&self, name: &str) -> Option<&SymbolInfo>;

    // === Warm Path (Tier 1 + 2) ===

    pub fn recent_history(&self, n: usize) -> Vec<HistoryEntry>;
    pub async fn checkpoint(&self) -> Result<()>;
    pub fn restore(session_id: &str) -> Result<Self>;

    // === Cold Path (Tier 3) ===

    pub async fn search_history(&self, query: &str) -> Result<Vec<HistoryEntry>>;
    pub fn query_sql(&self, sql: &str) -> Result<QueryResult>;
    pub fn export_history(&self, format: ExportFormat) -> Result<Vec<u8>>;
}
```

### 12.2 Internal Traits

```rust
/// Storage tier abstraction
trait StorageTier {
    type Key;
    type Value;

    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn set(&mut self, key: Self::Key, value: Self::Value);
    fn sync(&self) -> Result<()>;
}
```

---

## 13. Performance Targets

### 13.1 Latency Budgets

| Operation | Target | Tier | Notes |
|-----------|--------|------|-------|
| Variable get | < 100ns | 1 | Per-access during codegen |
| Variable set | < 200ns | 1 | After eval |
| Completion | < 1ms | 1 | Per-keystroke |
| Symbol lookup | < 100ns | 1 | Type checking |
| History append | < 1ms | 1→2 | User-perceived |
| History search | < 100ms | 3 | Interactive |
| Checkpoint | < 10ms | 2 | Background |
| Full sync | < 100ms | 3 | Periodic |

### 13.2 Memory Budgets

| Component | Target | Notes |
|-----------|--------|-------|
| VariableStore | < 10MB | Typical session |
| SymbolIndex | < 1MB | ~10K symbols |
| RecentHistory | < 1MB | 1000 entries |
| Total Tier 1 | < 20MB | Per session |

### 13.3 Durability Guarantees

| Tier | Guarantee | Recovery Time |
|------|-----------|---------------|
| 1 | None (volatile) | N/A |
| 2 | Crash-safe | < 100ms |
| 3 | Full ACID | < 1s |
| 4 | Filesystem | Immediate |

---

## 14. Migration and Evolution

### 14.1 Schema Versioning

```sql
-- Track schema version
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

-- Migrations applied in order
-- migrations/001_initial.sql
-- migrations/002_add_fts.sql
-- etc.
```

### 14.2 Forward Compatibility

- All Tier 2/3 data includes version numbers
- Unknown fields are preserved (JSON columns)
- Old clients can read new data (ignore unknown)

### 14.3 Data Export

```rust
impl OxurStore {
    /// Export all data for migration/backup
    pub fn export_all(&self) -> Result<ExportBundle> {
        ExportBundle {
            version: SCHEMA_VERSION,
            history: self.tier3.all_history()?,
            preferences: self.tier3.all_preferences()?,
            sessions: self.tier3.all_sessions()?,
        }
    }

    /// Import from export bundle
    pub fn import(&mut self, bundle: ExportBundle) -> Result<()> {
        // Validate version compatibility
        // Import data
        // Rebuild indexes
    }
}
```

---

## Version History

### Version 1.0 (2026-01-06)

Initial design document for tiered storage architecture, addressing performance requirements identified in ODD-0041 (Terminal UX) while maintaining the reliability goals of ODD-0038 (REPL Architecture).

---

**Document Status:** Draft - Ready for review
