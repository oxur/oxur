# Hot Code Loading Patterns for Tree-Walking Interpreters

**Date:** 2026-01-10
**Purpose:** Analyze BEAM's hot code loading and adapt patterns for Treebeard (tree-walking interpreter)
**Context:** BEAM uses a two-version bytecode system. How can we achieve similar benefits in a tree-walking interpreter?

---

## 1. BEAM's Hot Code Loading (Summary)

### 1.1 Two-Version Mechanism

**BEAM allows exactly TWO versions of a module to coexist:**

```
┌──────────────────────────────────────────────────────┐
│  Load Sequence                                        │
├──────────────────────────────────────────────────────┤
│                                                       │
│  Initial Load:                                        │
│  ┌────────────┐                                       │
│  │  Module v1 │  ← "Current" (in export table)      │
│  └────────────┘                                       │
│                                                       │
│  Second Load:                                         │
│  ┌────────────┐    ┌────────────┐                   │
│  │  Module v1 │    │  Module v2 │                    │
│  │  "Old"     │    │  "Current" │ ← export table    │
│  └────────────┘    └────────────┘                   │
│                                                       │
│  Third Load:                                          │
│  (v1 purged)       ┌────────────┐    ┌────────────┐ │
│                    │  Module v2 │    │  Module v3 │  │
│                    │  "Old"     │    │  "Current" │  │
│                    └────────────┘    └────────────┘  │
└──────────────────────────────────────────────────────┘
```

**Key Properties:**
1. **Current code** - Latest version, functions exported
2. **Old code** - Previous version, functions NOT exported
3. **Purge** - Third load kills processes in old code, removes it

### 1.2 Process Migration

**How processes switch from old → current:**

```erlang
% BAD: Stays in current version forever
loop(State) ->
    receive
        Msg -> loop(handle(Msg, State))
    end.

% GOOD: Can upgrade to new version
loop(State) ->
    receive
        Msg -> ?MODULE:loop(handle(Msg, State))  % Qualified call → new version
    end.
```

**Mechanism:**
- **Local calls** (`loop(...)`) → Direct jump, stays in current version
- **Qualified calls** (`?MODULE:loop(...)`) → Export table lookup, gets latest version

### 1.3 Export Table

**Global function registry:**

```
┌──────────────────────────────────────┐
│        Export Table (Global)          │
├──────────────────────────────────────┤
│  mymodule:foo/2  →  [BEAM addr]     │  ← Points to "current"
│  mymodule:bar/1  →  [BEAM addr]     │     version only
│  othermod:baz/0  →  [BEAM addr]     │
└──────────────────────────────────────┘
```

**Updated atomically when new version loads**

---

## 2. Why BEAM's Approach Doesn't Directly Apply

### 2.1 Bytecode vs AST

**BEAM:**
```
Source → Bytecode → Memory
         ↑
         │
      Atomic update of function pointer
```

**Treebeard:**
```
Source → AST → Interpreter
         ↑
         │
      AST is data structure, not executable
```

**Problem:** No "export table" with function pointers. Function lookup is a tree walk.

### 2.2 No Separate "Old" and "Current" Code Locations

**BEAM:** Old and current versions occupy different memory regions, each with executable bytecode.

**Treebeard:** AST nodes are data. There's no "executable" vs "non-executable" distinction.

### 2.3 Local vs Qualified Calls

**BEAM:** Compiler emits different instructions for local (`call`) vs qualified (`call_ext`).

**Treebeard:** All calls are late-bound lookups in the environment/module map. No distinction at "compile" time.

---

## 3. Tree-Walker Advantages for Hot Loading

### 3.1 Late Binding by Default

**BEAM needs qualified calls** to get new version.

**Treebeard:** ALL function calls do environment/module lookup!

```rust
// Pseudo-code for tree-walking function call
fn eval_call(func: &FunctionRef, args: Vec<Value>, env: &Environment) -> Value {
    // Look up function definition EVERY TIME
    let func_def = env.lookup_function(func)?;  // ← Always latest!

    // Bind parameters and evaluate
    let new_env = bind_params(&func_def.params, args, env);
    eval_block(&func_def.body, &new_env)
}
```

**Consequence:** **Functions automatically use latest version** without qualified calls!

### 3.2 Module Definitions Are Data

**BEAM:** Module is compiled bytecode in memory.

**Treebeard:** Module is a struct containing AST nodes.

```rust
struct Module {
    name: String,
    functions: HashMap<(String, usize), FunctionDef>,  // (name, arity) → AST
    types: HashMap<String, TypeDef>,
    // ... more
}
```

**Consequence:** Replacing a module is just updating a HashMap entry!

```rust
// Hot reload is trivial
interpreter.modules.insert(module_name, new_module);
```

### 3.3 No "Purging" Needed

**BEAM:** Must kill processes lingering in old code when third version loads.

**Treebeard:** No processes "lingering in code" - they're lingering in **call stacks**.

**Call stack doesn't hold code, it holds:**
- Frames (environment + return point)
- AST node references (which are looked up dynamically)

**Consequence:** Old code can be dropped immediately. If a stack frame references it, that's fine - it's just data.

---

## 4. Hot Code Loading Design for Treebeard

### 4.1 Core Pattern: Version-Tracked Module Registry

```rust
#[derive(Clone)]
struct ModuleRegistry {
    modules: Arc<RwLock<HashMap<String, VersionedModule>>>,
}

struct VersionedModule {
    name: String,
    version: u64,
    functions: HashMap<(String, usize), FunctionDef>,
    types: HashMap<String, TypeDef>,
    loaded_at: Instant,
}

impl ModuleRegistry {
    fn load_module(&self, module: Module) {
        let mut registry = self.modules.write().unwrap();

        let versioned = match registry.get(&module.name) {
            Some(existing) => VersionedModule {
                version: existing.version + 1,  // Increment version
                ..module.into()
            },
            None => VersionedModule {
                version: 1,
                ..module.into()
            },
        };

        registry.insert(module.name.clone(), versioned);
    }

    fn get_function(&self, module: &str, name: &str, arity: usize) -> Option<FunctionDef> {
        let registry = self.modules.read().unwrap();
        registry.get(module)
            .and_then(|m| m.functions.get(&(name.to_string(), arity)))
            .cloned()
    }
}
```

**Key Points:**
- `Arc<RwLock<>>` allows concurrent reads, exclusive writes
- Version number tracks reloads (useful for debugging)
- Lookups always get latest version

### 4.2 Function Call Resolution

```rust
fn eval_function_call(
    call: &FunctionCall,
    args: Vec<Value>,
    env: &Environment,
    registry: &ModuleRegistry,
) -> Result<Value> {
    // Resolve function at call time (late binding)
    let func_def = match &call.target {
        CallTarget::Local(name) => {
            // Local call: look in current module
            env.current_module()
                .and_then(|m| registry.get_function(m, name, args.len()))
                .ok_or_else(|| Error::UndefinedFunction)?
        }
        CallTarget::Qualified(module, name) => {
            // Qualified call: look in specified module
            registry.get_function(module, name, args.len())
                .ok_or_else(|| Error::UndefinedFunction)?
        }
    };

    // Evaluate with latest definition
    eval_function_body(&func_def, args, env, registry)
}
```

**Benefit:** No special handling for hot reload. Lookups always get current version.

### 4.3 Module Reload API

```rust
impl Interpreter {
    /// Reload a module from source file
    pub fn reload_module(&mut self, path: &Path) -> Result<String> {
        // 1. Parse source
        let source = std::fs::read_to_string(path)?;
        let ast = self.parser.parse(&source)?;

        // 2. Expand macros
        let expanded = self.macro_expander.expand(&ast)?;

        // 3. Build module
        let module = self.builder.build_module(&expanded)?;

        // 4. Replace in registry (atomic update)
        let module_name = module.name.clone();
        self.module_registry.load_module(module);

        Ok(module_name)
    }

    /// Reload all modules in a directory (for development)
    pub fn reload_workspace(&mut self, dir: &Path) -> Result<Vec<String>> {
        let mut reloaded = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension() == Some("oxur".as_ref()) {
                match self.reload_module(&path) {
                    Ok(name) => reloaded.push(name),
                    Err(e) => eprintln!("Failed to reload {:?}: {}", path, e),
                }
            }
        }

        Ok(reloaded)
    }
}
```

### 4.4 REPL Integration

```rust
// REPL commands for hot reloading
match input.trim() {
    cmd if cmd.starts_with("(reload ") => {
        // (reload "path/to/module.oxur")
        let path = parse_path(cmd)?;
        match interpreter.reload_module(&path) {
            Ok(name) => println!("✓ Reloaded module: {}", name),
            Err(e) => eprintln!("✗ Reload failed: {}", e),
        }
    }

    "(reload-all)" => {
        // Reload all modules in current workspace
        match interpreter.reload_workspace(Path::new(".")) {
            Ok(modules) => println!("✓ Reloaded {} module(s)", modules.len()),
            Err(e) => eprintln!("✗ Reload failed: {}", e),
        }
    }

    // ... other REPL commands
}
```

---

## 5. Advanced Patterns

### 5.1 Dependency-Aware Reloading

**Problem:** Module A depends on module B. Reloading B should maybe reload A?

```rust
struct ModuleRegistry {
    modules: Arc<RwLock<HashMap<String, VersionedModule>>>,
    dependencies: HashMap<String, HashSet<String>>,  // module → deps
}

impl ModuleRegistry {
    fn reload_module_and_dependents(&mut self, name: &str) {
        // Find all modules that depend on this one
        let dependents = self.find_dependents(name);

        // Reload the module
        self.load_module_from_source(name)?;

        // Reload dependents (topologically sorted)
        for dep in self.topological_sort(&dependents) {
            if let Err(e) = self.load_module_from_source(&dep) {
                eprintln!("Warning: Failed to reload dependent {}: {}", dep, e);
            }
        }
    }
}
```

**Trade-off:**
- ✅ Ensures dependent code uses updated definitions
- ❌ More complex, potentially reloads many modules
- ❌ May break if dependents fail to reload

**Recommendation:** Start simple (single-module reload), add this later if needed.

### 5.2 Snapshot/Rollback

**Use case:** Reload broke something, need to revert.

```rust
struct ModuleRegistry {
    modules: Arc<RwLock<HashMap<String, VersionedModule>>>,
    history: VecDeque<Snapshot>,  // Keep last N snapshots
}

struct Snapshot {
    timestamp: Instant,
    modules: HashMap<String, VersionedModule>,
}

impl ModuleRegistry {
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            timestamp: Instant::now(),
            modules: self.modules.read().unwrap().clone(),
        }
    }

    fn rollback(&mut self) {
        if let Some(snapshot) = self.history.pop_back() {
            *self.modules.write().unwrap() = snapshot.modules;
        }
    }
}
```

**REPL Integration:**

```
> (reload "broken_module.oxur")
✗ Reload failed: type error on line 42
> (rollback)
✓ Rolled back to previous snapshot
```

### 5.3 Hot Reload with State Migration

**Problem:** Module defines a struct/record. Hot reload changes its fields. What happens to existing instances?

**Option 1: Fail if instances exist**

```rust
fn reload_module_with_validation(
    &mut self,
    name: &str,
    heap: &Heap,
) -> Result<()> {
    // Check if any instances of this module's types exist
    if heap.has_instances_of_module(name) {
        return Err(Error::CannotReloadWithInstances);
    }

    // Safe to reload
    self.load_module_from_source(name)
}
```

**Option 2: Provide migration function**

```rust
struct VersionedModule {
    name: String,
    version: u64,
    functions: HashMap<(String, usize), FunctionDef>,
    types: HashMap<String, TypeDef>,

    // Optional: function to migrate instances from old version
    migrator: Option<MigratorFn>,
}

// User provides migration in module:
// (define (--migrate-- old-version old-data)
//   (match old-version
//     (1 (struct Point (x (get-x old-data)) (y (get-y old-data)) (z 0)))  ; Add z field
//     ...))
```

**Recommendation:** Start with Option 1 (fail-fast), add Option 2 if users need it.

### 5.4 Watch Mode (Development Feature)

**Automatically reload modules when files change:**

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

fn start_watch_mode(interpreter: Arc<Mutex<Interpreter>>, workspace: PathBuf) {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    watcher.watch(&workspace, RecursiveMode::Recursive).unwrap();

    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(event) => {
                    if let Some(path) = event.path() {
                        if path.extension() == Some("oxur".as_ref()) {
                            let mut interp = interpreter.lock().unwrap();
                            match interp.reload_module(&path) {
                                Ok(name) => println!("↻ Auto-reloaded: {}", name),
                                Err(e) => eprintln!("↻ Auto-reload failed: {}", e),
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {}", e),
            }
        }
    });
}
```

**REPL command:**

```
> (watch)
✓ Watching ./src for changes...
↻ Auto-reloaded: my_module (file changed)
```

---

## 6. Comparison: BEAM vs Treebeard

| Feature | BEAM | Treebeard |
|---------|------|-----------|
| **Versions** | Max 2 (current + old) | Unlimited (but only latest used) |
| **Lookup** | Export table (global) | Module registry (HashMap) |
| **Call types** | Local vs qualified | All late-bound |
| **Process migration** | Requires qualified calls | Automatic (late binding) |
| **Purging** | Manual (or auto on v3) | Automatic (GC'd when unused) |
| **Atomicity** | Yes (export table update) | Yes (RwLock write) |
| **Rollback** | Not built-in | Easy to add |
| **Dependency reload** | Not built-in | Easy to add |

---

## 7. Implementation Checklist for Treebeard

### Phase 1: Basic Hot Reload

- [ ] `ModuleRegistry` with `Arc<RwLock<HashMap>>`
- [ ] `reload_module(path)` method
- [ ] Version tracking (for debugging/logging)
- [ ] REPL command: `(reload "path")`
- [ ] Error handling: module not found, parse errors

### Phase 2: REPL Integration

- [ ] `(reload-all)` command to reload workspace
- [ ] Pretty printing of reload results
- [ ] Error messages with line/column info
- [ ] History integration (don't lose REPL state on reload)

### Phase 3: Developer Experience

- [ ] Watch mode for auto-reload
- [ ] Reload statistics (time taken, changes detected)
- [ ] Warning if reload might break running code
- [ ] `(modules)` command to list loaded modules with versions

### Phase 4: Advanced Features (Optional)

- [ ] Snapshot/rollback capability
- [ ] Dependency-aware reloading
- [ ] State migration hooks
- [ ] Performance monitoring (reload time)

---

## 8. Key Takeaways

### 8.1 Tree-Walking Advantages

1. **Late binding by default** - No need for qualified calls
2. **Simple data structure replacement** - Module is just a HashMap entry
3. **No purging complexity** - Old code GC'd automatically
4. **Easy rollback** - Just restore old HashMap

### 8.2 What to Avoid

1. **Don't replicate BEAM's two-version limit** - It's an optimization for bytecode, not needed for AST
2. **Don't distinguish local vs qualified calls** - Late binding handles both
3. **Don't try to "purge" old code** - Rust's ownership handles cleanup

### 8.3 Focus Areas for Treebeard

1. **Atomic module updates** - Use `RwLock` for thread-safe replacement
2. **Error recovery** - Failed reload shouldn't break interpreter
3. **Developer experience** - Watch mode, clear error messages
4. **REPL integration** - Seamless reload without losing session state

---

## 9. Example REPL Session

```
treebeard> (load "calculator.oxur")
✓ Loaded module: calculator (version 1)

treebeard> (calculator:add 2 3)
5

treebeard> ; Oops, add is wrong, let me fix it...
treebeard> (reload "calculator.oxur")
✓ Reloaded module: calculator (version 2)

treebeard> (calculator:add 2 3)
5  ; ← Now uses new definition!

treebeard> (reload "broken_calculator.oxur")
✗ Parse error on line 12: unexpected token '{'
✗ Module not reloaded (keeping version 2)

treebeard> (calculator:add 2 3)
5  ; ← Still works! Old version preserved.

treebeard> ; Fix the error, reload again
treebeard> (reload "calculator.oxur")
✓ Reloaded module: calculator (version 3)

treebeard> (watch)
✓ Watching ./src for changes...
↻ Auto-reloaded: calculator (file saved)
↻ Auto-reloaded: calculator (file saved)
```

---

## 10. References

- **BEAM Code Loading:** `otp/lib/kernel/src/code.erl`
- **Erlang Reference Manual:** "Compilation and Code Loading" chapter
- **BEAM Book:** Chapter on "Modules and Code Loading"
- **LFE:** Shows thin layer approach (doesn't replicate BEAM's loader)

---

**End of Report**
