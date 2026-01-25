# Implementation Plan: `odm add --dev` Feature

## Overview
Add support for `odm add --dev <filename> [--subdir <subdir>] [--force]` to manage development documents with automatic numbering and organization.

## Requirements
1. Add dev docs to `dev_directory` (configured or default `./docs/dev`)
2. Support `--subdir` for organizing dev docs into subdirectories
3. Each directory/subdirectory has **independent numbering** starting from 0001
4. Use 4-digit prefix format (0001-, 0002-, etc.)
5. File must be markdown with a title/heading (use existing validation)
6. Rename file based on title (reuse existing title extraction logic)
7. Create target directory if it doesn't exist
8. Respect `auto_stage_git` config setting for git staging
9. Support `--force` flag to overwrite existing files (with warning)
10. **No state tracking** - pure file operations, don't touch `.odm/state.json`

## Implementation Steps

### Step 1: Update CLI Definition
**File:** `crates/oxur-odm/src/cli.rs` (lines ~166-189)

Modify the `Commands::Add` variant:
```rust
Add {
    path: String,

    // NEW FLAGS
    #[arg(long)]
    dev: bool,

    #[arg(long, requires = "dev")]
    subdir: Option<String>,

    #[arg(short, long, requires = "dev")]
    force: bool,

    // EXISTING FLAGS (add conflicts)
    #[arg(short, long, conflicts_with = "dev")]
    state: Option<String>,

    #[arg(long)]
    dry_run: bool,

    #[arg(short, long, conflicts_with = "dev")]
    interactive: bool,

    #[arg(short = 'y', long)]
    yes: bool,

    #[arg(long, conflicts_with = "dev")]
    preview: bool,
}
```

**Key design:**
- `--dev` conflicts with `--state`, `--interactive`, `--preview` (simpler workflow for dev docs)
- `--subdir` requires `--dev`
- `--force` only applies to dev adds (requires `--dev`)

### Step 2: Create New Module for Dev Document Logic
**File:** `crates/oxur-odm/src/commands/add_dev.rs` (NEW FILE)

Create module with these functions:

**Main function:**
```rust
pub fn add_dev_document(
    config: &Config,
    doc_path: &str,
    subdir: Option<&str>,
    force: bool,
    dry_run: bool,
) -> Result<()>
```

**Logic flow:**
1. Validate source file exists
2. Read and validate markdown content
3. Extract title using `ExtractedMetadata::from_content()` and `determine_title_auto()`
4. Build target directory path using `build_target_directory()`
5. Find next number using `find_next_dev_number()`
6. Build filename using `build_filename(number, title)`
7. Check for existing file (error unless `--force`)
8. Create directory with `fs::create_dir_all()`
9. **Copy** file to target (keep source file - user confirmed)
10. Git add if `config.auto_stage_git == true`

**Helper functions:**
```rust
fn build_target_directory(config: &Config, subdir: Option<&str>) -> Result<PathBuf>
// Returns: dev_directory or dev_directory/subdir path

fn find_next_dev_number(dir: &Path) -> Result<u32>
// Scans directory for nnnn-*.md files, returns max + 1

fn extract_number_prefix(filename: &str) -> Option<u32>
// Extracts number from "0042-foo.md" -> Some(42)
```

**Reuse from existing code:**
- `determine_title_auto()` from `add.rs`
- `build_filename()` from `filename.rs`
- `is_valid_markdown()` from `extract.rs`
- `git::git_add()` from `git.rs`

### Step 3: Export New Module
**File:** `crates/oxur-odm/src/commands/mod.rs`

Add:
```rust
pub mod add_dev;
pub use add_dev::add_dev_document;
```

### Step 4: Update Command Dispatch
**File:** `crates/oxur-odm/src/main.rs`

**Change 1:** Update `execute_command` signature (line ~112):
```rust
pub(crate) fn execute_command(
    command: Commands,
    index: &DocumentIndex,
    state_mgr: &mut StateManager,
    config: &Config,  // ADD THIS PARAMETER
) -> Result<()>
```

**Change 2:** Update call site in `main()` (line ~63):
```rust
if let Err(e) = execute_command(cli.command, &index, &mut state_mgr, &config) {
```

**Change 3:** Update Add command dispatch (line ~134):
```rust
Commands::Add { path, dev, subdir, force, state, dry_run, interactive, yes, preview } => {
    if dev {
        add_dev_document(config, &path, subdir.as_deref(), force, dry_run)
    } else if preview {
        preview_add(&path, state_mgr)
    } else {
        add_document(state_mgr, &path, state.as_deref(), dry_run, interactive, yes)
    }
}
```

### Step 5: Add Comprehensive Tests
**File:** `crates/oxur-odm/src/commands/add_dev.rs` (tests module)

Test scenarios:
1. Extract number from filename patterns
2. Find next number in empty directory (should return 1)
3. Find next number with existing files (should return max + 1)
4. Build target directory without subdir
5. Build target directory with subdir
6. Add dev doc to root dev directory
7. Add dev doc to subdirectory
8. Independent numbering per directory
9. Force overwrite existing file
10. Reject existing file without --force
11. Dry run mode
12. Git staging with auto_stage_git=true
13. No git staging with auto_stage_git=false
14. Invalid markdown rejection
15. Missing title fallback to filename

## Critical Files

1. **`crates/oxur-odm/src/commands/add_dev.rs`** (NEW) - Core implementation
2. **`crates/oxur-odm/src/cli.rs`** - CLI flags
3. **`crates/oxur-odm/src/main.rs`** - Command routing + config passing
4. **`crates/oxur-odm/src/commands/mod.rs`** - Module exports
5. **`crates/oxur-odm/src/commands/add.rs`** - Reference for existing patterns

## Design Decisions

**Numbering:** Each directory/subdirectory has independent sequence
- `dev/0001-foo.md`, `dev/0002-bar.md`
- `dev/plans/0001-baz.md`, `dev/plans/0002-qux.md`

**Validation:** Looser than design docs - only check valid markdown + title

**Error Handling:**
- Missing file → bail immediately
- Missing title → fall back to filename
- Existing file → error unless `--force` (with warning)
- Git errors → warn but continue

**File Operations:**
- **Source file:** Kept in place (COPY operation, not move)
- **Target file:** Written to dev_directory/[subdir]/nnnn-title.md
- **Force flag:** Only available with `--dev` (design docs don't support --force due to state tracking)

**Config Integration:** Pass `Config` object through to access `dev_directory` and `auto_stage_git`

## Verification

**Manual testing:**
```bash
# Build and install
cargo build --package oxur-odm --release
cargo install --path crates/oxur-odm --force

# Test basic add
echo "# Test Doc" > /tmp/test.md
odm add --dev /tmp/test.md

# Test with subdirectory
odm add --dev /tmp/test2.md --subdir planning

# Test force overwrite
odm add --dev /tmp/test.md --force

# Test dry run
odm add --dev /tmp/test3.md --dry-run

# Verify numbering
ls -la <dev_directory>
ls -la <dev_directory>/planning
```

**Unit tests:**
```bash
cargo test --package oxur-odm add_dev
```

**All tests:**
```bash
cargo test --package oxur-odm
```

## Estimated Complexity
- **Lines of Code:** ~300 (200 implementation + 100 tests)
- **Risk Level:** Low - isolated feature, no state management
- **Dependencies:** None new - reuses existing helpers
