# LFE Architecture Analysis Report

**Date:** 2026-01-10
**Purpose:** Analyze LFE's architecture for insights applicable to Oxur/Treebeard

---

## 1. Architecture Overview

### 1.1 Compilation/Evaluation Pipeline

LFE implements a **dual-path system**:

```
┌─────────────────────────────────────────────────┐
│              LFE Source (.lfe)                   │
└──────────────────┬──────────────────────────────┘
                   │
       ┌───────────┴──────────┐
       │                      │
       ▼                      ▼
┌──────────────┐       ┌──────────────┐
│  Scan/Parse  │       │  Scan/Parse  │
│ (lfe_scan.erl│       │ (lfe_scan.erl│
│  lfe_parse.erl)│     │  lfe_parse.erl)│
└──────┬───────┘       └──────┬───────┘
       │                      │
       ▼                      ▼
┌──────────────┐       ┌──────────────┐
│Macro Expand  │       │Macro Expand  │
│(lfe_macro.erl)│      │(lfe_macro.erl)│
└──────┬───────┘       └──────┬───────┘
       │                      │
       ▼                      │
┌──────────────┐              │
│  Code Gen    │              │
│(lfe_codegen.erl)            │
│      ↓       │              │
│ Erlang AST   │              │
│      ↓       │              │
│ Core Erlang  │              ▼
│      ↓       │       ┌──────────────┐
│   BEAM       │       │  Evaluator   │
└──────────────┘       │(lfe_eval.erl)│
                       │  Tree-Walk   │
                       └──────────────┘
    COMPILER PATH          REPL PATH
```

**Key Insight:** LFE maintains separate paths for compilation and interpretation, but shares the same macro expansion logic.

### 1.2 Key Modules and Their Responsibilities

| Module | Responsibility | LOC (approx) |
|--------|---------------|--------------|
| `lfe_scan.erl` | Lexical analysis of S-expressions | ~300 |
| `lfe_parse.erl` | Parsing S-expressions into LFE AST | ~400 |
| `lfe_macro.erl` | Macro expansion (user & predefined) | ~1433 |
| `lfe_eval.erl` | Tree-walking interpreter | ~1200 |
| `lfe_shell.erl` | REPL server/client | ~900 |
| `lfe_env.erl` | Environment (vars/funs/macros/records) | ~267 |
| `lfe_codegen.erl` | Code generation to Erlang AST | ~1400 |

### 1.3 Relationship to BEAM

**LFE follows the "Thin Layer" principle:**

```
┌─────────────────────────────────────┐
│          LFE Syntax Layer            │
│  - S-expression parsing              │
│  - Macro expansion                   │
│  - Syntax transformation             │
└──────────────┬──────────────────────┘
               │ Erlang AST
               ▼
┌─────────────────────────────────────┐
│         Erlang/BEAM Layer            │
│  - Type system                       │
│  - Pattern matching                  │
│  - Process model                     │
│  - Code loading                      │
│  - Everything else                   │
└─────────────────────────────────────┘
```

**Key Principle:** LFE does **syntax + macros**. BEAM does **everything else**.

---

## 2. Macro System

### 2.1 Macro Expansion Architecture

**State Structure** (`#mac` record in `lfe_macro.erl`):

```erlang
-record(mac, {
    deep=false,     % Deep expansion flag
    keep=false,     % Keep macro definitions flag
    line=1,         % Current line number
    file,           % Source file name
    opts=[],        % Compiler options
    ipath=[],       % Include path
    errors=[],      % Accumulated errors
    warnings=[],    % Accumulated warnings
    module,         % Current module name
    vc=0,           % Variable counter (for gensym)
    fc=0,           % Function counter
    unloadable=[]   % Modules that couldn't be loaded
}).
```

### 2.2 Macro Expansion Algorithm

**Entry Point:** `exp_macro(Call, Env, State) -> {yes, Expansion, State} | no`

```erlang
exp_macro([Name|_]=Call, Env, St) ->
    case is_atom(Name) andalso lfe_internal:is_core_form(Name) of
        true -> no;    % Never expand core forms
        false ->
            case get_mbinding(Name, Env) of
                {yes,Def} -> exp_userdef_macro(Call, Def, Env, St);
                no -> exp_predef_macro(Call, Env, St)
            end
    end.
```

**Expansion Flow:**

1. **Check if core form** → Never expand (`quote`, `lambda`, `cons`, etc.)
2. **Check user-defined macros** → Expand via `exp_userdef_macro`
3. **Check predefined macros** → Expand via `exp_predef_macro`
4. **Return `no`** → Not a macro, process as regular form

### 2.3 User-Defined Macros

**Definition:** Stored as `lambda` or `match-lambda` in environment

```erlang
% Macro definition: (define-macro name [args] body)
% Stored in environment as:
{macro, ['lambda', [Args, '$ENV'], Body]}
% or
{macro, ['match-lambda', [[Args1, '$ENV'], Body1], ...]}
```

**Expansion Process:**

```erlang
exp_userdef_macro([Mac|Args], Def0, Env, St0) ->
    try
        {Def1,St1} = exp_form(Def0, Env, St0),  % Expand the macro definition first
        Exp = lfe_eval:apply(Def1, [Args,Env], Env),  % Apply to args + $ENV
        {yes,Exp,St1}
    catch
        error:Error ->
            erlang:raise(error, {expand_macro,[Mac|Args],Error}, Stack)
    end.
```

**Key Points:**
- Macros receive **two arguments**: the argument list **and** the current environment
- The macro definition itself is expanded before application
- Errors during expansion are caught and re-raised with context

### 2.4 Predefined Macros

**Examples from `exp_predef`:**

```erlang
% Backquote (quasiquote)
exp_predef([backquote,Bq], _, St) ->
    {yes,exp_backquote(Bq),St};

% defun (syntactic sugar for define-function)
exp_predef([defun,Name|Rest], _, St) ->
    {Meta,Def} = exp_defun(Rest),
    {yes,['define-function',Name,Meta,Def],St};

% defmacro (syntactic sugar for define-macro)
exp_predef([defmacro,Name|Rest], _, St) ->
    {Meta,Def} = exp_defmacro(Rest),
    {yes,['define-macro',Name,Meta,Def],St};

% list* (improper list constructor)
exp_predef(['list*'|As], _, St) ->
    Exp = exp_list_star(As),
    {yes,Exp,St};
```

### 2.5 When Does Expansion Happen?

**Two Modes:**

1. **Pass Mode** (`pass_form` in `lfe_macro.erl`):
   - Used during file compilation
   - Expands forms at top-level to check structure
   - Can optionally deep-expand (`deep=true`)
   - Collects and removes macro definitions from output

2. **Expression Mode** (`exp_form` in `lfe_macro.erl`):
   - Used during evaluation
   - Always performs deep expansion
   - Leaves macro definitions in place

**Macro Definition Collection:**

```erlang
pass_form(['define-macro'|Def]=M, Env0, St0) ->
    case pass_define_macro(Def, Env0, St0) of
        {yes,Env1,St1} ->
            Ret = ?IF(St1#mac.keep, M, [progn]),  % Keep or remove
            {Ret,Env1,St1};
        {no,St1} ->
            {['progn'],Env0,St1}
    end;
```

### 2.6 Macro Environment Separation

**Critical Distinction:**

```
┌────────────────────────────────────────┐
│      Compile-Time Environment           │
│  - Macro definitions                    │
│  - Record definitions                   │
│  - Type definitions                     │
│  - eval-when-compile bindings          │
└────────────────────────────────────────┘
                 │
                 │ Separate from
                 ▼
┌────────────────────────────────────────┐
│       Runtime Environment               │
│  - Variable bindings                    │
│  - Function definitions                 │
│  - Dynamic function bindings            │
└────────────────────────────────────────┘
```

**Implementation:**
- Both use the same `#env{}` structure from `lfe_env.erl`
- Distinction is **contextual**: which environment is passed where
- Macros have access to the compile-time environment via `$ENV` parameter

---

## 3. Environment/Binding Model

### 3.1 Environment Structure

```erlang
-record(env, {
    vars=null,   % Variable bindings (map/orddict)
    funs=null,   % Function/macro bindings (map/orddict)
    recs=null    % Record definitions (map/orddict)
}).
```

### 3.2 Variable Bindings

**Simple key-value pairs:**

```erlang
% add_vbinding(Name, Value, Env) -> Env
Env#env{vars=maps:put(Name, Value, Vars)}

% get_vbinding(Name, Env) -> {yes, Value} | no
case maps:find(Name, Vars) of
    {ok,V} -> {yes,V};
    error -> no
end
```

### 3.3 Function/Macro Bindings

**Complex due to shadowing:**

```erlang
% Function bindings: {function, [{Arity, Definition}, ...]}
% Macro bindings: {macro, Definition}

% When a macro is defined, it shadows ALL function bindings with same name
% When a function is defined, it shadows a macro with same name
%   OR replaces a function with same name/arity
```

**Storage Format:**

```erlang
Funs = #{
    'foo' => {function, [{2, Def1}, {3, Def2}]},  % foo/2 and foo/3
    'bar' => {macro, MacroDef},                    % bar macro
    'baz' => {function, [{1, {erlang, length}}]}   % imported function
}
```

### 3.4 Compile-Time vs Runtime

**Key Difference:**

| Aspect | Compile-Time | Runtime |
|--------|--------------|---------|
| **Macros** | Available | Not available |
| **Functions** | Some via `eval-when-compile` | All defined |
| **Variables** | Limited (in `eval-when-compile`) | All in scope |
| **Records** | Definitions available | Expanded to tuples |

### 3.5 Closures and Environment Capture

**Lambda Closures:**

```erlang
% When evaluating (lambda [x] (+ x y))
% The lambda captures the current environment Env at definition time

eval_lambda([lambda, Args, Body], Env) ->
    % Create a closure that captures Env
    fun(ActualArgs) ->
        % Create new environment with Args bound to ActualArgs
        NewEnv = bind_args(Args, ActualArgs, Env),
        % Evaluate body in NewEnv
        eval_body(Body, NewEnv)
    end.
```

**Key Point:** LFE uses **lexical scoping** - closures capture the environment at definition time.

---

## 4. REPL Implementation

### 4.1 Three-Environment Architecture

**State Structure** (`lfe_shell.erl`):

```erlang
-record(state, {
    curr,       % Current environment
    save,       % Saved environment (before slurp)
    base,       % Base environment (predefined vars/funs/macros)
    slurp=false % Are we in slurped state?
}).
```

**Purpose of Each:**

1. **Base Environment:**
   - Contains predefined shell functions (`c/1`, `h/1`, `help/0`, etc.)
   - Contains predefined shell macros
   - Contains predefined shell variables (`+`, `-`, `*`, `$ENV`)
   - Immutable - used to reset environment

2. **Current Environment:**
   - Active working environment
   - Contains all user definitions
   - Updated with each successful evaluation

3. **Saved Environment:**
   - Snapshot before `slurp` operation
   - Allows rollback via `unslurp`

### 4.2 Shell Variables

**Special Variables:**

```erlang
% History variables
'+'    % Most recent input form
'++'   % Second most recent input form
'+++'  % Third most recent input form

'*'    % Most recent result value
'**'   % Second most recent result value
'***'  % Third most recent result value

'-'    % Current input form (being evaluated)

'$ENV' % The entire current environment
```

**Implementation:**

```erlang
update_shell_vars(Form, Value, Env0) ->
    Env1 = foldl(fun ({Symb,Val}, E) ->
                     lfe_env:add_vbinding(Symb, Val, E)
                 end,
                 Env0,
                 [{'+++', fetch_vbinding('++', Env0)},
                  {'++',  fetch_vbinding('+',  Env0)},
                  {'+',   Form},
                  {'***', fetch_vbinding('**', Env0)},
                  {'**',  fetch_vbinding('*',  Env0)},
                  {'*',   Value}]),
    % Update $ENV with self-reference (carefully to avoid infinite growth)
    Env2 = del_vbinding('$ENV', Env1),
    add_vbinding('$ENV', Env2, Env2).
```

### 4.3 Definition Persistence

**How definitions persist:**

```erlang
eval_form_1(['define-function',Name,_Meta,Def], #state{curr=Ce0}=St) ->
    Ar = function_arity(Def),
    Ce1 = lfe_eval:add_dynamic_func(Name, Ar, Def, Ce0),
    {Name,St#state{curr=Ce1}};

eval_form_1(['define-macro',Name,_Meta,Def], #state{curr=Ce0}=St) ->
    Ce1 = lfe_env:add_mbinding(Name, Def, Ce0),
    {Name,St#state{curr=Ce1}};

eval_form_1(['define-record',Name,Fields], #state{curr=Ce0}=St) ->
    Ce1 = lfe_env:add_record(Name, Fields, Ce0),
    {Name,St#state{curr=Ce1}};
```

**Key Points:**
- Definitions are added to the **current environment**
- The updated environment is stored in the shell state
- State persists across REPL inputs
- Errors don't update the environment (old state retained)

### 4.4 Error Handling Without State Loss

**Separate Evaluator Process:**

```erlang
% Shell spawns an evaluator process
start_eval(St) ->
    Self = self(),
    spawn_link(fun () -> eval_init(Self, St) end).

% Shell sends expression to evaluator
shell_eval(Form, Eval0, St0) ->
    Eval0 ! {eval_expr,self(),Form},
    receive
        {eval_value,Eval0,_Value,St1} ->
            {Eval0,St1};  % Success: update state
        {eval_error,Eval0,{Class,Reason,StackTrace}} ->
            report_exception(Class, Reason, StackTrace),
            Eval1 = start_eval(St0),  % Restart evaluator
            {Eval1,St0};  % Error: keep old state
        {'EXIT',Eval0,Reason} ->
            report_exception(exit, Reason, []),
            Eval1 = start_eval(St0),
            {Eval1,St0}
    end.
```

**Benefits:**
- Evaluator crash doesn't crash shell
- Old state is preserved on error
- Can restart fresh evaluator

### 4.5 Slurp Mechanism

**Purpose:** Load an entire file into the REPL, making all functions/macros available

**Implementation:**

```erlang
slurp([File], St0) ->
    {ok,#state{curr=Ce0}=St1} = unslurp(St0),   % First, revert any previous slurp
    Name = lfe_eval:expr(File, Ce0),             % Evaluate file name
    case slurp_file(Name) of
        {ok,Mod,Funs,Env0,Ws} ->
            % Add imported functions
            Env1 = add_imports(Imports, Env0),
            % Add all functions
            Env2 = add_functions(Funs, Env1),
            % Add records
            Env3 = add_records(Records, Env2),
            % Save current environment and activate slurped
            {{ok,Mod},St1#state{save=Ce0,curr=Env3,slurp=true}}
    end.
```

---

## 5. Hot Code Loading (From BEAM)

### 5.1 Two-Version System

**BEAM Mechanism:**

```
┌──────────────┐
│   Module M   │
│  Version 1   │  ← "Current" code
│   (loaded)   │
└──────────────┘
       │
       │ Load Version 2
       ▼
┌──────────────┐    ┌──────────────┐
│   Module M   │    │   Module M   │
│  Version 1   │    │  Version 2   │
│   "Old"      │    │  "Current"   │
└──────────────┘    └──────────────┘
       │                   │
       │ Load Version 3    │
       ▼                   ▼
   (purged)           ┌──────────────┐    ┌──────────────┐
                      │   Module M   │    │   Module M   │
                      │  Version 2   │    │  Version 3   │
                      │   "Old"      │    │  "Current"   │
                      └──────────────┘    └──────────────┘
```

**Rules:**
1. First load → Code becomes "current"
2. Second load → Previous becomes "old", new becomes "current"
3. Third load → "Old" is purged (processes killed), "current" becomes "old", new becomes "current"

### 5.2 Export Table and Global Calls

**Key Distinction:**

```erlang
% Local call (stays in current version)
foo() -> bar().

% Qualified call (always gets latest version)
foo() -> ?MODULE:bar().
```

**Why this works:**
- Export table only points to "current" code
- Local calls are compiled to direct jumps
- Qualified calls go through export table
- This allows processes to "jump" to new code

### 5.3 Process Migration

**Processes must explicitly opt into new code:**

1. **Qualified calls** (`Module:Function()`) force jump to new version
2. **Code replacement points** - places where process can safely switch
3. **Purging** - old processes are killed when third version loads

**LFE Pattern:**

```lisp
;; This stays in current version
(defun server-loop (state)
  (receive
    (msg (handle-message msg state))
    ('stop 'ok)))

;; This allows code upgrade
(defun server-loop (state)
  (receive
    (msg (let ((new-state (handle-message msg state)))
           (?MODULE:server-loop new-state)))  ; Qualified call!
    ('stop 'ok)))
```

---

## 6. Patterns to Adopt for Treebeard

### 6.1 Thin Layer Architecture

**LFE Lesson:** Do **one thing well** (syntax transformation), delegate everything else.

**For Treebeard:**
```
┌────────────────────────────────────┐
│      Treebeard (Interpreter)        │
│  - Tree-walking execution           │
│  - Environment management           │
│  - Control flow                     │
│  - NO type checking                 │
│  - NO ownership checking            │
└──────────────┬─────────────────────┘
               │ syn AST
               │
               ▼
┌────────────────────────────────────┐
│     Rust Compiler (rustc)           │
│  - Type checking                    │
│  - Borrow checking                  │
│  - Optimization                     │
│  - Everything else                  │
└────────────────────────────────────┘
```

### 6.2 Dual-Path Execution

**LFE Pattern:** Maintain both interpreter and compiler paths

**For Treebeard:**
- **Interpreter path**: Fast startup, interactive development, REPL
- **Compiler path**: Escape hatch for performance-critical code
- **Shared**: Macro expansion, parsing, validation

### 6.3 Three-Environment REPL Pattern

**Adopt for treebeard-repl:**

```rust
struct ReplState {
    base: Environment,    // Predefined bindings
    save: Environment,    // Snapshot before slurp
    curr: Environment,    // Current working environment
    slurped: bool,
}
```

**Benefits:**
- Easy reset to clean state
- Slurp/unslurp functionality
- State preservation on error

### 6.4 Separate Evaluator Process Pattern

**LFE Pattern:** Shell spawns evaluator as separate process

**For Treebeard (with Rust concurrency):**
```rust
// Shell thread
let (tx, rx) = channel();
let evaluator = spawn_evaluator(tx.clone(), state.clone());

// Send expression
evaluator.send(EvalRequest { form, env });

// Receive result
match rx.recv() {
    Ok(EvalSuccess { value, new_env }) => {
        // Update state
    }
    Err(EvalError { error, old_env }) => {
        // Keep old state, restart evaluator
    }
}
```

**Benefits:**
- Panic-safe REPL
- State isolation
- Clean error recovery

### 6.5 Macro Expansion Strategy

**LFE Lesson:** Expand macros **before** evaluation/compilation

**For Oxur:**
1. Parse source → S-expressions
2. Expand macros → Expanded S-expressions
3. **Fork:**
   - **REPL:** Convert to syn AST, interpret via Treebeard
   - **Compiler:** Convert to syn AST, generate Rust source, compile via rustc

### 6.6 Environment Structure

**Adopt LFE's clean separation:**

```rust
struct Environment {
    vars: HashMap<Symbol, Value>,
    funs: HashMap<(Symbol, Arity), FunctionDef>,
    macros: HashMap<Symbol, MacroDef>,
    records: HashMap<Symbol, RecordDef>,
}
```

**With shadowing rules:**
- Macro definition shadows all functions with same name
- Function definition shadows macro with same name
- Functions distinguished by arity

### 6.7 REPL History Variables

**Adopt LFE's pattern:**

```rust
// In REPL environment
env.bind("+",   most_recent_form);
env.bind("++",  second_recent_form);
env.bind("+++", third_recent_form);
env.bind("*",   most_recent_value);
env.bind("**",  second_recent_value);
env.bind("***", third_recent_value);
env.bind("-",   current_form);
```

---

## 7. Patterns That Don't Apply

### 7.1 BEAM Bytecode Format

**LFE compiles to BEAM bytecode** - Treebeard doesn't need this.

- Treebeard interprets `syn` AST directly
- No bytecode format needed
- Compilation escape hatch goes straight to rustc

### 7.2 Actor Model / Process Scheduler

**BEAM's actor model** - Use Rust's concurrency instead.

- LFE uses lightweight Erlang processes
- Treebeard can use Rust threads, async/await, channels
- Don't try to replicate BEAM's scheduler

### 7.3 Garbage Collection

**BEAM's GC** - Use Rust ownership model.

- LFE/BEAM has GC per process
- Treebeard uses Rust's ownership and RAII
- Lifetime management is compile-time (via rustc path)

### 7.4 Distribution Protocol

**BEAM's distribution** - Not needed for Treebeard.

- Erlang's distributed Erlang protocol
- Treebeard is a local interpreter
- Network communication via Rust libraries

### 7.5 Hot Code Loading Details

**BEAM's implementation** - Different for tree-walking.

- BEAM uses two-version bytecode mechanism
- Treebeard can reload syn AST directly
- No export table needed (late-bound function lookup by default)

---

## 8. Key Takeaways

### 8.1 What Makes LFE Work

1. **Thin layer** - Only handles syntax, delegates to BEAM
2. **100% interop** - Can call any Erlang function
3. **Semantic alignment** - Embraces Erlang semantics, doesn't fight them
4. **Separate compilation** - Macros expand, then compile OR interpret
5. **Clean environments** - Separation of concerns (vars/funs/macros/records)

### 8.2 Critical Design Decisions

1. **Macros receive environment** - `$ENV` parameter gives macros compile-time context
2. **Two execution paths** - Compiler path (BEAM) and interpreter path (eval)
3. **Three REPL environments** - base/save/curr for flexibility
4. **Separate evaluator process** - Error isolation
5. **Late-bound macro expansion** - Macros can call macros, environment tracks definitions

### 8.3 Lessons for Oxur/Treebeard

1. **Do one thing well**: Treebeard = tree-walking interpreter, not a type checker
2. **Embrace Rust semantics**: Don't try to add GC or actors
3. **Dual-path execution**: Interpreter for dev, compiler for production
4. **Clean separation**: Compile-time (macros) vs runtime (evaluation)
5. **REPL resilience**: Separate evaluator, preserve state on error
6. **Slurp is powerful**: Load entire files into REPL for exploration

---

## 9. Code Patterns to Extract

### Pattern 1: Macro Expansion Loop

**Found in:** `lfe_macro.erl:exp_form/3`

```erlang
exp_form([Fun|_]=Call, Env, St0) when is_atom(Fun) ->
    % Try to expand as macro
    case exp_macro(Call, Env, St0) of
        {yes,Exp,St1} -> exp_form(Exp, Env, St1);  % Recursively expand result
        no -> exp_tail(Call, Env, St0)              % Not a macro, expand args
    end.
```

**Lesson for Oxur:** Recursive macro expansion until no more macros found.

### Pattern 2: Environment Handling

**Found in:** `lfe_env.erl:add_vbinding/3, add_fbinding/4`

```erlang
add_vbinding(N, V, #env{vars=Vs}=Env) ->
    Env#env{vars=maps:put(N, V, Vs)}.

add_fbinding(N, A, V, #env{funs=Fs0}=Env) ->
    Def = {function,[{A,V}]},
    Upd = fun ({function,Fas}) ->
              {function,lists:keystore(A, 1, Fas, {A,V})};
          (_) -> Def  % Overwrite macros
          end,
    Fs1 = maps:update_with(N, Upd, Def, Fs0),
    Env#env{funs=Fs1}.
```

**Lesson for Oxur:** Clean separation, shadowing rules enforced at binding time.

### Pattern 3: REPL State Management

**Found in:** `lfe_shell.erl:shell_eval/3`

```erlang
shell_eval(Form, Eval0, St0) ->
    Eval0 ! {eval_expr,self(),Form},
    receive
        {eval_value,Eval0,_Value,St1} ->
            {Eval0,St1};  % Success: use new state
        {eval_error,Eval0,{Class,Reason,StackTrace}} ->
            Eval1 = start_eval(St0),
            {Eval1,St0}  % Error: keep old state, restart evaluator
    end.
```

**Lesson for Oxur:** State updates only on success, errors preserve previous state.

### Pattern 4: Slurp File Loading

**Found in:** `lfe_shell.erl:slurp/2`

```erlang
slurp([File], St0) ->
    {ok,#state{curr=Ce0}=St1} = unslurp(St0),  % Revert previous slurp
    case slurp_file(Name) of
        {ok,Mod,Funs,Env0,Ws} ->
            Env3 = add_all_definitions(Funs, Env0),
            {{ok,Mod},St1#state{save=Ce0,curr=Env3,slurp=true}}
    end.
```

**Lesson for Oxur:** Save old environment, load all definitions, mark as slurped state.

---

## 10. References

- **LFE Source:** `github.com/lfe/lfe`
- **Key Files:**
  - `src/lfe_macro.erl` - Macro expansion
  - `src/lfe_eval.erl` - Tree-walking interpreter
  - `src/lfe_shell.erl` - REPL implementation
  - `src/lfe_env.erl` - Environment management
  - `src/lfe_codegen.erl` - Code generation to Erlang AST
- **LFE Book:** `lfe.io/books/tutorial`
- **Robert Virding's Talks:** Search for "LFE" on YouTube

---

**End of Report**
