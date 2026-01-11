# Treebeard Design Synthesis: Answering Key Questions

**Date:** 2026-01-10
**Purpose:** Answer specific design questions for Treebeard based on LFE/BEAM/Elixir analysis

---

## Question 1: How does LFE's evaluator manage bindings during tree-walking evaluation?

### Answer

**LFE uses a layered environment structure with lexical scoping and closure capture.**

### Environment Structure (`lfe_env.erl`)

```erlang
-record(env, {
    vars=null,   % Variable bindings (map: Symbol → Value)
    funs=null,   % Function/macro bindings (map: Name → {function|macro, Def})
    recs=null    % Record definitions (map: Name → Fields)
}).
```

### Binding Management During Evaluation

**1. Variable Lookup (`lfe_eval.erl:344-348`):**

```erlang
eval_expr(Symb, Env) when is_atom(Symb) ->
    case lfe_env:get_vbinding(Symb, Env) of
        {yes,Val} -> Val;
        no -> unbound_symbol_error(Symb)
    end.
```

**2. Local Bindings in `let` (`lfe_eval.erl:eval_let`):**

```erlang
eval_let([Vbs|Body], Env0) ->
    % Bind all variables
    Env1 = lists:foldl(
        fun ([Pat,Exp], Env) ->
            Val = eval_expr(Exp, Env0),    % Eval in old env (parallel semantics)
            bind_pattern(Pat, Val, Env)     % Add to new env
        end,
        Env0,
        Vbs
    ),
    % Evaluate body with extended environment
    eval_body(Body, Env1).
```

**3. Lambda Closures (`lfe_eval.erl:eval_lambda`):**

```erlang
eval_lambda([lambda, Args, Body], Env) ->
    % Closure captures the current environment
    fun(ActualArgs) ->
        % Extend the captured environment with arg bindings
        NewEnv = bind_args(Args, ActualArgs, Env),
        eval_body(Body, NewEnv)
    end.
```

**4. Function Definitions (`lfe_eval.erl:add_dynamic_func`):**

```erlang
add_dynamic_func(Name, Arity, Def, Env) ->
    % Functions are stored as closures
    Closure = eval_lambda(Def, Env),  % Capture environment at definition time
    lfe_env:add_fbinding(Name, Arity, Closure, Env).
```

### Key Patterns for Treebeard

**Pattern 1: Environment Immutability**

```rust
// LFE-inspired approach
fn eval_let(bindings: &[(Pattern, Expr)], body: &[Expr], env: &Environment) -> Value {
    // Create new environment (immutable extension)
    let mut new_env = env.clone();

    // Evaluate all binding values in ORIGINAL environment (parallel semantics)
    let values: Vec<Value> = bindings
        .iter()
        .map(|(_, expr)| eval_expr(expr, env))  // Use old env
        .collect();

    // Bind all patterns in NEW environment
    for ((pattern, _), value) in bindings.iter().zip(values) {
        bind_pattern(pattern, value, &mut new_env)?;
    }

    // Evaluate body in extended environment
    eval_block(body, &new_env)
}
```

**Pattern 2: Closure Capture**

```rust
#[derive(Clone)]
enum Value {
    Closure {
        params: Vec<Symbol>,
        body: Block,
        captured_env: Environment,  // ← Captures environment at definition
    },
    // ...
}

fn eval_lambda(params: &[Symbol], body: &Block, env: &Environment) -> Value {
    Value::Closure {
        params: params.to_vec(),
        body: body.clone(),
        captured_env: env.clone(),  // ← Lexical scoping
    }
}

fn apply_closure(closure: &Closure, args: Vec<Value>) -> Value {
    // Extend the CAPTURED environment, not current environment
    let mut call_env = closure.captured_env.clone();

    // Bind parameters
    for (param, arg) in closure.params.iter().zip(args) {
        call_env.bind(param, arg);
    }

    // Evaluate body
    eval_block(&closure.body, &call_env)
}
```

**Pattern 3: Shadowing Rules (LFE's function/macro shadowing)**

```rust
impl Environment {
    fn bind_function(&mut self, name: Symbol, arity: usize, def: FunctionDef) {
        // Remove any macro with same name (shadowing)
        self.macros.remove(&name);

        // Add or replace function with this arity
        self.functions.entry((name.clone(), arity))
            .or_insert_with(Vec::new)
            .push(def);
    }

    fn bind_macro(&mut self, name: Symbol, def: MacroDef) {
        // Remove ALL functions with same name (macro shadows all arities)
        self.functions.retain(|(fn_name, _), _| fn_name != &name);

        // Add macro
        self.macros.insert(name, def);
    }
}
```

### Recommendation for Treebeard

**Use immutable environment extension + closure capture:**

```rust
#[derive(Clone)]
pub struct Environment {
    /// Variable bindings (innermost scope first)
    bindings: im::HashMap<Symbol, Value>,

    /// Parent environment (for lexical scoping)
    parent: Option<Arc<Environment>>,
}

impl Environment {
    /// Look up variable (search up scope chain)
    pub fn lookup(&self, name: &Symbol) -> Option<&Value> {
        self.bindings.get(name)
            .or_else(|| self.parent.as_ref()?.lookup(name))
    }

    /// Extend environment with new binding (immutable)
    pub fn extend(&self, name: Symbol, value: Value) -> Self {
        let mut new_bindings = self.bindings.clone();
        new_bindings.insert(name, value);

        Environment {
            bindings: new_bindings,
            parent: Some(Arc::new(self.clone())),
        }
    }
}
```

**Use `im::HashMap` for structural sharing** (efficient clone).

---

## Question 2: What mechanisms does LFE use to support hot code loading and dynamic function redefinition?

### Answer

**LFE itself does NOT implement hot code loading. It delegates to BEAM's two-version mechanism.**

### What LFE Does

**LFE's role:**
1. **Compiles to Erlang AST** → Core Erlang → BEAM bytecode
2. **BEAM handles all code loading**
   - Two-version system
   - Export table management
   - Process migration
   - Purging

**LFE's contribution: Qualified calls**

```lisp
;; BAD: Stays in old version
(defun server-loop (state)
  (receive
    (msg (server-loop (handle msg state)))))  ; Local call

;; GOOD: Can migrate to new version
(defun server-loop (state)
  (receive
    (msg (SERVER-MODULE:server-loop (handle msg state)))))  ; Qualified call
```

LFE's compiler generates Erlang AST with `call_ext` instructions for qualified calls.

### REPL-Level Dynamic Redefinition

**LFE's REPL (`lfe_shell.erl`) DOES support dynamic redefinition:**

```erlang
eval_form_1(['define-function',Name,_Meta,Def], #state{curr=Ce0}=St) ->
    Arity = function_arity(Def),
    % Replace function in current environment
    Ce1 = lfe_eval:add_dynamic_func(Name, Arity, Def, Ce0),
    {Name,St#state{curr=Ce1}}.  % Update REPL state
```

**Mechanism:**
- Functions stored as **closures** in environment
- Redefinition replaces the closure
- All subsequent calls use new definition (late binding)

### Treebeard Implications

**Treebeard has BETTER hot reload than BEAM!**

**Why:**
1. **Late binding by default** - All calls do function lookup
2. **No export table** - Just replace HashMap entry
3. **No two-version limit** - Can reload indefinitely
4. **No purging** - Old code GC'd automatically

**Implementation** (from Hot Code Loading report):

```rust
struct ModuleRegistry {
    modules: Arc<RwLock<HashMap<String, Module>>>,
}

impl ModuleRegistry {
    fn reload_module(&mut self, module: Module) {
        let mut registry = self.modules.write().unwrap();
        registry.insert(module.name.clone(), module);  // Atomic replace
    }

    fn get_function(&self, module: &str, name: &str, arity: usize) -> Option<FunctionDef> {
        let registry = self.modules.read().unwrap();
        registry.get(module)
            .and_then(|m| m.functions.get(&(name.to_string(), arity)))
            .cloned()
    }
}

fn eval_call(call: &Call, env: &Environment, registry: &ModuleRegistry) -> Value {
    // Always lookup latest definition
    let func_def = registry.get_function(&call.module, &call.name, call.args.len())?;
    apply_function(&func_def, eval_args(&call.args, env))
}
```

**Key difference from LFE:**
- LFE: Delegates to BEAM's bytecode loader
- Treebeard: Direct AST replacement in HashMap

---

## Question 3: How does LFE integrate with Erlang's macro expansion and module system?

### Answer

**LFE has its OWN macro system (separate from Erlang). LFE macros expand to LFE forms, which then compile to Erlang AST.**

### LFE Does NOT Use Erlang Macros

**Erlang's "macros" are preprocessor directives** (`-define`, `-ifdef`), not true macros.

**LFE implements Lisp-style macros:**
- Hygienic (can be, via gensym)
- Receive environment as parameter
- Full code-as-data (S-expressions)

### LFE Macro Expansion Pipeline

```
┌─────────────────────┐
│   LFE Source        │
│  (define-macro ...) │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Macro Expansion    │  ← LFE's macro expander (lfe_macro.erl)
│  (User + Predefined)│
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Expanded LFE Forms │
│  (No more macros)   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Code Generation    │  ← lfe_codegen.erl
│  (To Erlang AST)    │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│   Erlang Compiler   │  ← Erlang takes over here
│  (AST → Core → BEAM)│
└─────────────────────┘
```

### Module System Integration

**LFE modules ARE Erlang modules:**

```lisp
;; LFE module definition
(defmodule calculator
  (export (add 2) (multiply 2)))

(defun add (x y)
  (+ x y))
```

**Compiles to Erlang module:**

```erlang
-module(calculator).
-export([add/2, multiply/2]).

add(X, Y) -> X + Y.
```

**Key Points:**
1. **LFE modules use Erlang module system** (module names, exports, imports)
2. **LFE macros are compile-time only** (don't exist at runtime)
3. **LFE functions are Erlang functions** (same calling convention)

### LFE Macro Example

```lisp
;; Define a macro
(define-macro unless
  (lambda (test then else env)
    `(if (not ,test) ,then ,else)))

;; Use the macro
(unless (> x 10)
  (print "x is small")
  (print "x is large"))

;; Expands to:
(if (not (> x 10))
  (print "x is small")
  (print "x is large"))
```

**The `env` parameter** gives macros access to compile-time environment.

### Treebeard Implications

**Oxur should have its own macro system (like LFE), not try to use Rust's macros.**

**Why:**
1. **Rust macros are hygiene-only** (no code inspection)
2. **Rust macros are token-based** (not AST-based)
3. **Rust macros can't be defined at runtime**

**Recommended Architecture:**

```
┌─────────────────────┐
│   Oxur Source       │
│  (defmacro unless ...)│
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Macro Expansion    │  ← Oxur's macro expander (Rust crate)
│  (User + Predefined)│
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Expanded S-exprs   │
│  (No more macros)   │
└──────────┬──────────┘
           │
           ├─────────────────┐
           │                 │
           ▼                 ▼
   ┌──────────────┐   ┌──────────────┐
   │  Convert to  │   │  Convert to  │
   │  syn AST     │   │  syn AST     │
   └──────┬───────┘   └──────┬───────┘
          │                  │
          ▼                  ▼
   ┌──────────────┐   ┌──────────────┐
   │  Treebeard   │   │  rustc       │
   │ (Interpret)  │   │ (Compile)    │
   └──────────────┘   └──────────────┘
      REPL Path         Production Path
```

**Key Principle:** Macros are **Oxur's responsibility**, not Rust's.

---

## Question 4: How should Treebeard handle function definition storage and lookup in an interpreted environment?

### Answer

**Use a two-level structure: Module Registry + Environment Stack**

### Recommended Architecture

```rust
/// Global module registry (thread-safe)
pub struct ModuleRegistry {
    modules: Arc<RwLock<HashMap<String, Module>>>,
}

/// Per-module storage
pub struct Module {
    name: String,
    functions: HashMap<(String, usize), FunctionDef>,  // (name, arity) → def
    types: HashMap<String, TypeDef>,
    // ... more
}

/// Per-call-stack environment
pub struct Environment {
    /// Local variable bindings
    locals: im::HashMap<Symbol, Value>,

    /// Current module (for local calls)
    current_module: Option<String>,

    /// Parent environment (for closures)
    parent: Option<Arc<Environment>>,
}
```

### Function Lookup Algorithm

```rust
fn resolve_function_call(
    call: &FunctionCall,
    env: &Environment,
    registry: &ModuleRegistry,
) -> Result<FunctionDef> {
    match &call.target {
        // Local call: foo(x, y)
        CallTarget::Unqualified(name) => {
            let module = env.current_module()
                .ok_or(Error::NoCurrentModule)?;

            registry.get_function(module, name, call.args.len())
                .ok_or(Error::UndefinedFunction)
        }

        // Qualified call: Module::foo(x, y)
        CallTarget::Qualified(module, name) => {
            registry.get_function(module, name, call.args.len())
                .ok_or(Error::UndefinedFunction)
        }
    }
}
```

### Closure Storage

```rust
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),

    /// Closure captures environment at definition time
    Closure {
        params: Vec<Symbol>,
        body: Block,
        captured_env: Arc<Environment>,  // ← Lexical scope
    },

    /// Native function (Rust function)
    NativeFunction {
        name: String,
        arity: usize,
        func: Arc<dyn Fn(Vec<Value>) -> Result<Value>>,
    },
}
```

### Function Application

```rust
fn apply_value(value: &Value, args: Vec<Value>, registry: &ModuleRegistry) -> Result<Value> {
    match value {
        Value::Closure { params, body, captured_env } => {
            // Create new environment from CAPTURED environment
            let mut call_env = captured_env.as_ref().clone();

            // Bind parameters
            for (param, arg) in params.iter().zip(args) {
                call_env.bind(param.clone(), arg);
            }

            // Evaluate body
            eval_block(body, &call_env, registry)
        }

        Value::NativeFunction { func, .. } => {
            // Call Rust function directly
            func(args)
        }

        _ => Err(Error::NotCallable),
    }
}
```

### REPL Function Storage

```rust
/// REPL-specific: Allow defining functions interactively
pub struct ReplState {
    /// REPL's temporary module (for functions defined at REPL)
    repl_module: Module,

    /// Current environment
    env: Environment,

    /// Module registry (shared with interpreter)
    registry: Arc<ModuleRegistry>,
}

impl ReplState {
    pub fn define_function(&mut self, name: String, params: Vec<Symbol>, body: Block) {
        let arity = params.len();

        // Create closure in current environment
        let closure = Value::Closure {
            params,
            body,
            captured_env: Arc::new(self.env.clone()),
        };

        // Store in REPL module
        self.repl_module.functions.insert((name.clone(), arity), FunctionDef {
            name,
            params: Vec::new(),  // Already captured in closure
            body,
        });

        // Update registry
        self.registry.update_module(self.repl_module.clone());
    }
}
```

### Comparison: LFE vs Treebeard

| Feature | LFE | Treebeard |
|---------|-----|-----------|
| **Function storage** | Environment + Erlang module table | Module Registry + Environment |
| **Lookup** | lfe_env:get_fbinding → erlang:apply | registry.get_function → eval |
| **Closures** | Erlang fun() | Value::Closure with captured env |
| **REPL defs** | Add to current environment | Add to REPL module → registry |
| **Hot reload** | BEAM's code loading | Module registry update |

---

## Question 5: What patterns from LFE's REPL can inform the design of an Oxur REPL backed by Treebeard?

### Answer

**LFE's REPL demonstrates several patterns critical for a robust interpreted REPL.**

### Pattern 1: Three-Environment Architecture

**LFE Pattern** (`lfe_shell.erl`):

```erlang
-record(state, {
    curr,       % Current working environment
    save,       % Saved environment (before slurp)
    base,       % Base environment (predefined functions/vars)
    slurp=false
}).
```

**Treebeard Adaptation:**

```rust
pub struct ReplState {
    /// Base environment (prelude, built-ins)
    base: Environment,

    /// Saved snapshot (before slurp)
    save: Option<Environment>,

    /// Current working environment
    curr: Environment,

    /// Are we in a slurped state?
    slurped: bool,

    /// Module registry
    registry: Arc<ModuleRegistry>,
}

impl ReplState {
    pub fn reset(&mut self) {
        self.curr = self.base.clone();
        self.save = None;
        self.slurped = false;
    }

    pub fn slurp(&mut self, module_path: &Path) -> Result<()> {
        // Save current environment
        self.save = Some(self.curr.clone());

        // Load module
        let module = load_module(module_path)?;

        // Add all functions to environment
        for ((name, arity), func_def) in &module.functions {
            self.curr.bind_function(name.clone(), *arity, func_def.clone());
        }

        self.slurped = true;
        Ok(())
    }

    pub fn unslurp(&mut self) {
        if let Some(saved) = self.save.take() {
            self.curr = saved;
            self.slurped = false;
        }
    }
}
```

### Pattern 2: Separate Evaluator Process

**LFE Pattern** (`lfe_shell.erl:start_eval`):

```erlang
start_eval(St) ->
    Self = self(),
    spawn_link(fun () -> eval_init(Self, St) end).

shell_eval(Form, Eval0, St0) ->
    Eval0 ! {eval_expr,self(),Form},
    receive
        {eval_value,Eval0,_Value,St1} -> {Eval0,St1};
        {eval_error,Eval0,Error} ->
            Eval1 = start_eval(St0),  % Restart
            {Eval1,St0}  % Keep old state
    end.
```

**Treebeard Adaptation (using Rust channels):**

```rust
pub struct ReplServer {
    eval_tx: Sender<EvalRequest>,
    result_rx: Receiver<EvalResult>,
    state: ReplState,
}

struct EvalRequest {
    expr: Expr,
    env: Environment,
}

enum EvalResult {
    Success { value: Value, new_env: Environment },
    Error { error: Error },
}

impl ReplServer {
    pub fn new(state: ReplState) -> Self {
        let (eval_tx, eval_rx) = channel();
        let (result_tx, result_rx) = channel();

        // Spawn evaluator thread
        std::thread::spawn(move || {
            Self::evaluator_loop(eval_rx, result_tx);
        });

        ReplServer {
            eval_tx,
            result_rx,
            state,
        }
    }

    fn evaluator_loop(rx: Receiver<EvalRequest>, tx: Sender<EvalResult>) {
        while let Ok(req) = rx.recv() {
            // Eval with panic catching
            let result = std::panic::catch_unwind(|| {
                eval_expr(&req.expr, &req.env, &registry)
            });

            match result {
                Ok(Ok((value, new_env))) => {
                    tx.send(EvalResult::Success { value, new_env }).ok();
                }
                Ok(Err(error)) | Err(_) => {
                    tx.send(EvalResult::Error { error }).ok();
                }
            }
        }
    }

    pub fn eval(&mut self, expr: Expr) -> Result<Value> {
        // Send request
        self.eval_tx.send(EvalRequest {
            expr,
            env: self.state.curr.clone(),
        })?;

        // Wait for result
        match self.result_rx.recv()? {
            EvalResult::Success { value, new_env } => {
                self.state.curr = new_env;  // Update on success
                Ok(value)
            }
            EvalResult::Error { error } => {
                // Keep old environment on error
                Err(error)
            }
        }
    }
}
```

**Benefits:**
- Panics in evaluator don't crash REPL
- Environment only updated on success
- Can restart evaluator on fatal error

### Pattern 3: History Variables

**LFE Pattern** (`lfe_shell.erl:update_shell_vars`):

```erlang
'+'    % Most recent form
'++'   % Second most recent form
'+++'  % Third most recent form
'*'    % Most recent value
'**'   % Second most recent value
'***'  % Third most recent value
'-'    % Current form (being evaluated)
```

**Treebeard Adaptation:**

```rust
impl ReplState {
    pub fn update_history(&mut self, form: &Expr, value: &Value) {
        // Shift history variables
        if let Some(old_plus) = self.curr.lookup(&"+".into()) {
            self.curr.bind("++".into(), old_plus.clone());
        }
        if let Some(old_plusplus) = self.curr.lookup(&"++".into()) {
            self.curr.bind("+++".into(), old_plusplus.clone());
        }

        if let Some(old_star) = self.curr.lookup(&"*".into()) {
            self.curr.bind("**".into(), old_star.clone());
        }
        if let Some(old_starstar) = self.curr.lookup(&"**".into()) {
            self.curr.bind("***".into(), old_starstar.clone());
        }

        // Bind current values
        self.curr.bind("+".into(), Value::Expr(form.clone()));
        self.curr.bind("*".into(), value.clone());
    }
}
```

**Usage in REPL:**

```rust
treebeard> (+ 2 3)
5
treebeard> (* 10 *)  ; Use previous result
50
treebeard> +         ; Show previous form
(* 10 5)
```

### Pattern 4: Slurp Mechanism

**LFE's slurp** loads an entire file into the REPL, making all functions available.

**Treebeard Implementation:**

```rust
impl ReplState {
    pub fn slurp(&mut self, path: &Path) -> Result<String> {
        // Save current environment
        self.save = Some(self.curr.clone());

        // Parse and expand file
        let source = std::fs::read_to_string(path)?;
        let ast = parse(&source)?;
        let expanded = expand_macros(&ast, &self.curr)?;

        // Build module
        let module = build_module(&expanded)?;

        // Add all functions to environment
        for ((name, arity), func_def) in &module.functions {
            let closure = create_closure(func_def, &self.curr);
            self.curr.bind_function(name.clone(), *arity, closure);
        }

        // Add all records
        for (name, record_def) in &module.records {
            self.curr.bind_record(name.clone(), record_def.clone());
        }

        self.slurped = true;
        Ok(module.name.clone())
    }

    pub fn unslurp(&mut self) -> Result<()> {
        match self.save.take() {
            Some(saved) => {
                self.curr = saved;
                self.slurped = false;
                Ok(())
            }
            None => Err(Error::NotSlurped),
        }
    }
}
```

**REPL commands:**

```
treebeard> (slurp "calculator.oxur")
✓ Slurped module: calculator
treebeard> (add 2 3)
5
treebeard> (unslurp)
✓ Reverted slurp
treebeard> (add 2 3)
✗ Error: undefined function add/2
```

### Pattern 5: Pretty Printing with Depth Limits

**LFE Pattern** (`lfe_shell.erl:eval_form`):

```erlang
eval_form(Form, Shell, St0) ->
    Value = lfe_eval:expr(Form, St0#state.curr),
    % Print to depth 30
    VS = lfe_io:prettyprint1(Value, 30),
    io:requests([{put_chars,unicode,VS},nl]),
    ...
```

**Treebeard Adaptation:**

```rust
impl ReplServer {
    fn print_value(&self, value: &Value) {
        const MAX_DEPTH: usize = 30;
        println!("{}", value.pretty_print(MAX_DEPTH));
    }
}

impl Value {
    fn pretty_print(&self, max_depth: usize) -> String {
        self.pretty_print_impl(max_depth, 0)
    }

    fn pretty_print_impl(&self, max_depth: usize, current_depth: usize) -> String {
        if current_depth >= max_depth {
            return "...".to_string();
        }

        match self {
            Value::Int(i) => i.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::List(items) => {
                let items_str: Vec<_> = items.iter()
                    .map(|v| v.pretty_print_impl(max_depth, current_depth + 1))
                    .collect();
                format!("[{}]", items_str.join(", "))
            }
            // ... more variants
        }
    }
}
```

### Summary: Key REPL Patterns from LFE

| Pattern | LFE Implementation | Treebeard Adaptation |
|---------|-------------------|----------------------|
| **Three environments** | base/save/curr | Same structure |
| **Separate evaluator** | Erlang process | Rust thread + channels |
| **History variables** | +, ++, +++, *, **, *** | Same bindings |
| **Error recovery** | Restart evaluator, keep old state | Same strategy |
| **Slurp/unslurp** | Load file into environment | Same, with snapshot |
| **Pretty printing** | Depth-limited | Same approach |
| **Definition persistence** | Add to environment | Add to REPL module |

---

## Summary

### Key Takeaways for Treebeard

1. **Environment Management:**
   - Use immutable environment extension (clone + extend)
   - Closures capture environment at definition time
   - Implement shadowing rules (macros shadow functions)

2. **Hot Code Loading:**
   - Tree-walkers have BETTER hot reload than BEAM
   - Use `Arc<RwLock<HashMap>>` for module registry
   - Late binding means all calls use latest version

3. **Macro System:**
   - Build Oxur's own macro system (don't use Rust macros)
   - Macros expand before interpretation/compilation
   - Macros are compile-time only

4. **Function Storage:**
   - Module Registry (global, thread-safe)
   - Environment (per-call-stack, local bindings)
   - Closures capture environment at definition

5. **REPL Design:**
   - Three environments (base/save/curr)
   - Separate evaluator thread (panic-safe)
   - History variables (+, *, etc.)
   - Slurp/unslurp for file loading
   - Pretty printing with depth limits

---

**End of Report**
