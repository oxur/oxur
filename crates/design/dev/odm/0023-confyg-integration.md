# Implementation Plan: Confyg Integration for oxur-odm

## Overview

Integrate the `confyg` library into `oxur-odm` to improve configuration UX for projects outside the oxur repository. This will add intelligent config file discovery while maintaining backward compatibility with existing `.odmrc` files.

## Objectives

1. ✅ Add `docs_directory` field to `PartialConfig` struct
2. ✅ Integrate confyg with smart search paths during ODM startup
3. ✅ Implement config file search order: `./odm.toml` → confyg search paths → defaults
4. ✅ Update `oxur-odm/Cargo.toml` to use own version `0.3.0` instead of workspace version
5. ✅ Maintain backward compatibility with `.odmrc` files

## Configuration Loading Priority (High to Low)

1. **CLI arguments** (`--docs-dir`) - Overrides everything
2. **./odm.toml** - Current directory (highest file priority)
3. **Confyg search paths** - Git root, ~/.config/odm/, /etc/odm/
4. **Legacy .odmrc** - Backward compatibility (with deprecation warning)
5. **Smart defaults** - Workspace detection (existing behavior)
6. **Hardcoded defaults** - From `Config::default()`

## Critical Files to Modify

### 1. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-odm/Cargo.toml`

**Change**: Update version from workspace to own version

```toml
# Line 3: Change from
version.workspace = true

# To
version = "0.3.0"
```

**Reason**: oxur-odm should track its own version independently of the workspace

---

### 2. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-odm/src/config.rs`

This is the main file with significant changes needed.

#### Change 2.1: Add imports

**Location**: Top of file (after existing imports)

```rust
use confyg::searchpath::Finder;
use std::env;
```

#### Change 2.2: Update PartialConfig to include docs_directory

**Location**: Line 105-111 (PartialConfig struct)

```rust
#[derive(Debug, Deserialize)]
struct PartialConfig {
    project_root: Option<PathBuf>,
    docs_directory: Option<PathBuf>,  // ADD THIS LINE
    dustbin_directory: Option<PathBuf>,
    preserve_dustbin_structure: Option<bool>,
    auto_stage_git: Option<bool>,
}
```

#### Change 2.3: Update merge() method

**Location**: Line 79-92 (merge method)

Add docs_directory handling:

```rust
fn merge(&mut self, other: PartialConfig) {
    if let Some(val) = other.project_root {
        self.project_root = val;
    }
    // ADD THESE LINES
    if let Some(val) = other.docs_directory {
        self.docs_directory = val.clone();
        self.state_file = val.join(".odm/state.json");
        self.dustbin_directory = val.join(".dustbin");
    }
    // END NEW LINES
    if let Some(val) = other.dustbin_directory {
        self.dustbin_directory = val;
    }
    if let Some(val) = other.preserve_dustbin_structure {
        self.preserve_dustbin_structure = val;
    }
    if let Some(val) = other.auto_stage_git {
        self.auto_stage_git = val;
    }
}
```

#### Change 2.4: Replace load() method

**Location**: Line 41-60 (entire load method)

Replace with confyg-enabled version:

```rust
/// Load configuration with confyg search paths
pub fn load(docs_dir: Option<&str>) -> Result<Self> {
    // Start with defaults
    let mut config = Config::default();

    // 1. Try to load from confyg search paths (odm.toml)
    if let Some(confyg_config) = Self::load_from_confyg()? {
        config.merge(confyg_config);
    }

    // 2. Try legacy .odmrc for backward compatibility (deprecated)
    if let Some(legacy_config) = Self::load_legacy_odmrc()? {
        oxur_cli::common::output::warning(
            ".odmrc is deprecated, please migrate to odm.toml"
        );
        config.merge(legacy_config);
    }

    // 3. Override docs_directory if CLI provided (highest priority)
    if let Some(dir) = docs_dir {
        let path = PathBuf::from(dir);
        config.docs_directory = path.clone();
        config.state_file = path.join(".odm/state.json");
        config.dustbin_directory = path.join(".dustbin");
    }

    // 4. Try to load from .odm/config.toml in docs directory
    if let Some(file_config) = Self::load_from_file(&config.docs_directory)? {
        config.merge(file_config);
    }

    Ok(config)
}
```

#### Change 2.5: Add new helper methods

**Location**: After load() method, before load_from_file()

Add three new methods:

```rust
/// Load configuration using confyg's search path mechanism
fn load_from_confyg() -> Result<Option<PartialConfig>> {
    // Build search paths
    let mut finder = Finder::new();

    // 1. Current directory (highest priority for local overrides)
    finder = finder.add_path(".");

    // 2. Git repository root (if in a git repo)
    if let Some(repo_root) = crate::git::get_repo_root() {
        finder = finder.add_path(repo_root);
    }

    // 3. User config directory (~/.config/odm/)
    if let Some(config_dir) = Self::get_user_config_dir() {
        finder = finder.add_path(config_dir);
    }

    // 4. System config directory (optional)
    finder = finder.add_path("/etc/odm");

    // Search for odm.toml
    match finder.find("odm.toml") {
        Ok(config_path) => {
            let contents = std::fs::read_to_string(&config_path)
                .context(format!("Failed to read {:?}", config_path))?;

            let config: PartialConfig = toml::from_str(&contents)
                .context(format!("Failed to parse {:?}", config_path))?;

            Ok(Some(config))
        }
        Err(_) => Ok(None), // No config file found, use defaults
    }
}

/// Get user config directory (~/.config/odm or platform equivalent)
fn get_user_config_dir() -> Option<PathBuf> {
    // Use XDG_CONFIG_HOME if set, otherwise ~/.config
    if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        Some(PathBuf::from(xdg_config).join("odm"))
    } else if let Ok(home) = env::var("HOME") {
        Some(PathBuf::from(home).join(".config/odm"))
    } else {
        None
    }
}

/// Load legacy .odmrc from git root (backward compatibility)
fn load_legacy_odmrc() -> Result<Option<PartialConfig>> {
    let Some(root) = crate::git::get_repo_root() else {
        return Ok(None);
    };

    let config_path = root.join(".odmrc");

    if !config_path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(&config_path)
        .context("Failed to read .odmrc")?;

    // .odmrc has a different format - just docs_dir
    #[derive(Debug, serde::Deserialize)]
    struct LegacyConfig {
        docs_dir: Option<String>,
    }

    let legacy: LegacyConfig = toml::from_str(&contents)
        .context("Failed to parse .odmrc")?;

    Ok(legacy.docs_dir.map(|dir| PartialConfig {
        project_root: None,
        docs_directory: Some(PathBuf::from(dir)),
        dustbin_directory: None,
        preserve_dustbin_structure: None,
        auto_stage_git: None,
    }))
}
```

#### Change 2.6: Add comprehensive tests

**Location**: In the `#[cfg(test)]` module at the end of the file (after line 180)

Add these new tests:

```rust
#[test]
fn test_partial_config_with_docs_directory() {
    let temp = TempDir::new().unwrap();
    let docs_dir = temp.path();

    fs::create_dir_all(docs_dir.join(".odm")).unwrap();
    fs::write(
        docs_dir.join(".odm/config.toml"),
        r#"
docs_directory = "/custom/docs"
preserve_dustbin_structure = false
"#,
    )
    .unwrap();

    let config = Config::load(Some(docs_dir.to_str().unwrap())).unwrap();
    // docs_directory from .odm/config.toml should be merged
    // but then CLI override wins
    assert_eq!(config.docs_directory.to_str().unwrap(), docs_dir.to_str().unwrap());
}

#[test]
fn test_confyg_current_dir_priority() {
    // Test that ./odm.toml takes precedence
    // (Requires setting up temp directory as current dir)
}

#[test]
fn test_legacy_odmrc_shows_warning() {
    // Test that .odmrc works but shows deprecation
    // (May require capturing output)
}

#[test]
fn test_cli_override_beats_all() {
    // Verify CLI --docs-dir overrides everything
}
```

---

### 3. `/Users/oubiwann/lab/oxur/oxur/crates/oxur-odm/src/main.rs`

**Minimal changes** - The existing code should work with the new Config::load() implementation.

#### Optional Enhancement (Line 82-114)

Simplify `apply_smart_default()` by adding a comment that confyg now handles most of the logic:

```rust
/// Apply smart default for docs directory
/// Note: Most config search is now handled by Config::load() via confyg
pub(crate) fn apply_smart_default(cli: &mut Cli) {
    // Only apply workspace-specific smart defaults if user didn't override --docs-dir
    if cli.docs_dir != "docs" {
        return;
    }

    // Rest of function unchanged...
}
```

---

## Example odm.toml Configuration

Create this as documentation reference:

```toml
# ODM Configuration File
# Place in: ./, <repo-root>/, ~/.config/odm/, or /etc/odm/

# Path to design documents directory
# Default: "./design/docs"
docs_directory = "./docs/design"

# Path to dustbin directory for removed documents
# Default: "{docs_directory}/.dustbin"
dustbin_directory = "./docs/design/.dustbin"

# Preserve state directory structure in dustbin
# Default: true
preserve_dustbin_structure = true

# Automatically stage changes with git
# Default: true
auto_stage_git = true
```

---

## Edge Cases Handled

### 1. **Malformed odm.toml**

- Returns error with clear message and example syntax
- User can fix the file or remove it to use defaults

### 2. **No config file found**

- Gracefully falls back to hardcoded defaults
- Tool works out-of-box

### 3. **Both .odmrc and odm.toml exist**

- odm.toml takes precedence
- Shows deprecation warning for .odmrc
- Encourages migration

### 4. **CLI override semantics**

- `--docs-dir` always wins (highest priority)
- Predictable behavior for users

### 5. **Permission issues**

- Clear error message if config file exists but can't be read
- Suggests checking file permissions

---

## Verification Steps

### Step 1: Version Update

```bash
cd crates/oxur-odm
grep "^version" Cargo.toml
# Expected: version = "0.3.0"
```

### Step 2: Config in Current Directory

```bash
cat > odm.toml << EOF
docs_directory = "./test-docs"
preserve_dustbin_structure = false
EOF

odm info  # Should use ./test-docs
```

### Step 3: Config Search Paths

```bash
# Test git root
cd /path/to/repo
echo 'docs_directory = "./repo-docs"' > odm.toml
cd subdir
odm info  # Should find ../odm.toml
```

### Step 4: CLI Override

```bash
odm --docs-dir /override/path info
# Should use /override/path regardless of config files
```

### Step 5: Legacy .odmrc

```bash
cat > .odmrc << EOF
docs_dir = "legacy/docs"
EOF

odm info
# Should work but show deprecation warning
```

### Step 6: No Config Fallback

```bash
rm -f odm.toml .odmrc
odm info
# Should use default "./design/docs"
```

### Step 7: Run Tests

```bash
cargo test --package oxur-odm --lib config
cargo test --package oxur-odm
```

---

## Implementation Sequence

### Phase 1: Version Update (5 min)

1. Update `Cargo.toml` version to `0.3.0`
2. Verify build still works

### Phase 2: PartialConfig Update (10 min)

1. Add `docs_directory` field to `PartialConfig`
2. Update `merge()` method
3. Run existing tests to ensure no regressions

### Phase 3: Confyg Integration (30 min)

1. Add imports
2. Add three new methods: `load_from_confyg()`, `get_user_config_dir()`, `load_legacy_odmrc()`
3. Replace `load()` method with new implementation
4. Manual testing: Create `./odm.toml` and verify it's loaded

### Phase 4: Testing (20 min)

1. Add new unit tests
2. Run full test suite
3. Manual testing with various config scenarios

### Phase 5: Documentation (10 min)

1. Add example `odm.toml` to docs or README
2. Update any existing documentation about configuration

**Total Estimated Time: ~75 minutes**

---

## Rollback Plan

If issues arise:

1. **Immediate**: Revert changes to `config.rs`, keep `Cargo.toml` change
2. **Partial**: Keep confyg but disable search paths (only use `./odm.toml`)
3. **Full**: Revert all changes via git

---

## Success Criteria

- ✅ `cargo test --package oxur-odm` passes
- ✅ `./odm.toml` is loaded and used when present
- ✅ Config search finds files in git root and user config dir
- ✅ CLI `--docs-dir` override works correctly
- ✅ `.odmrc` still works with deprecation warning
- ✅ No config file scenario works with defaults
- ✅ Version is `0.3.0` in `Cargo.toml`
- ✅ All existing functionality preserved
