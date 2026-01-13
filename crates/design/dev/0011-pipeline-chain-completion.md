# Development Plan: Completing ODD-0013 Architecture

**Date:** 2026-01-12
**Purpose:** Roadmap to complete the 6-stage compilation pipeline per ODD-0013
**Status:** Based on pipeline analysis and STATUS.md current state

---

## Current State Assessment

### ODD-0013 Phase Completion

| Phase | ODD-0013 Goal | Current Status | % Complete |
|-------|---------------|----------------|------------|
| **Phase 0** | Foundation | ✅ Done | 100% |
| **Phase 1** | Parse & Source Maps | 🚧 Partial | 60% |
| **Phase 2** | Core Forms & Lowering | 🚧 Partial | 40% |
| **Phase 3** | Expansion | 🚧 Partial | 30% |
| **Phase 4** | Generation & End-to-End | 🚧 Partial | 70% |
| **Phase 5** | Core Macros Library | ❌ Not Started | 0% |
| **Phase 6** | REPL | 🚧 Partial | 60% |
| **Phase 7** | CLI & Tooling | 🚧 Partial | 45% |
| **Phase 8** | v1.0 Release | ❌ Not Started | 0% |

### Critical Architectural Gaps

**From ODD-0013 Specification:**

1. **Stage 4 Not Integrated** (Phase 2 incomplete)
   - Current: Stage 3 outputs `syn::File` directly
   - Required: Stage 3 outputs Oxur AST S-expressions
   - Impact: No buffer zone, oxur-comp depends on syn

2. **Source Mapping Not Populated** (Phase 1 incomplete)
   - Current: NodeIds generated but positions not recorded
   - Required: Full position tracking and mapping
   - Impact: Error messages point to generated Rust, not Oxur source

3. **Limited Core Forms** (Phase 2 incomplete)
   - Current: Only DefineFunc, IfExpr, MatchExpr defined
   - Required: Operators, calls, let bindings, etc.
   - Impact: Can only compile trivial programs

4. **Minimal Expansion** (Phase 3 incomplete)
   - Current: Only `deffn` macro expanded
   - Required: Core macro framework + essential macros
   - Impact: No control flow, no syntactic sugar

5. **REPL Subprocess IPC** (Phase 6 incomplete)
   - Current: Subprocess lifecycle management only
   - Required: IPC protocol for LOAD/RUN commands
   - Impact: Tier 2/3 execution doesn't work

---

## Recommended Development Path

### Strategy: "Complete the Phases in Order"

**Philosophy:** ODD-0013 phases were designed to build on each other. Complete them sequentially for solid foundation.

---

## Phase 1 Completion: Source Mapping (2 weeks)

**Goal:** Make error messages point to Oxur source

### Week 1: Position Tracking in Parser

**Tasks:**

1. **Add Span to SurfaceForm**

   ```rust
   #[derive(Debug, Clone)]
   pub struct Span {
       pub start: SourcePos,  // line, column, offset
       pub end: SourcePos,
   }

   pub enum SurfaceForm {
       Symbol { value: String, span: Span },
       Number { value: i64, span: Span },
       String { value: String, span: Span },
       List { elements: Vec<SurfaceForm>, span: Span },
   }
   ```

2. **Update Parser to Track Positions**

   ```rust
   impl Parser {
       fn current_position(&self) -> SourcePos {
           // Calculate line/column from byte offset
           let mut line = 1;
           let mut column = 1;
           for (i, ch) in self.source.chars().enumerate() {
               if i >= self.position { break; }
               if ch == '\n' {
                   line += 1;
                   column = 1;
               } else {
                   column += 1;
               }
           }
           SourcePos { line, column, offset: self.position }
       }

       fn parse_form(&mut self) -> Result<SurfaceForm> {
           let start = self.current_position();
           let form = /* parse logic */;
           let end = self.current_position();
           Ok(form.with_span(Span { start, end }))
       }
   }
   ```

3. **Tests:**
   - Parse with position tracking
   - Verify spans for nested forms
   - Check line/column accuracy

**Deliverables:**

- ✅ All SurfaceForm variants have Span
- ✅ Parser records accurate positions
- ✅ Tests verify position tracking

### Week 2: Mapping Chains Through Pipeline

**Tasks:**

1. **Record Surface → Core Mappings**

   ```rust
   impl Expander {
       fn expand_form(&mut self, form: SurfaceForm) -> Result<CoreForm> {
           let surface_span = form.span();
           let core_id = NodeId::new();

           // Record mapping
           self.source_map.record_span(core_id, surface_span);

           // Continue expansion
           match form {
               SurfaceForm::Symbol { value, .. } => {
                   Ok(CoreForm::Symbol { id: core_id, name: value })
               }
               // ...
           }
       }
   }
   ```

2. **Record Core → Oxur AST Mappings** (will need Stage 4 integration)
   - Defer to Phase 2 completion
   - For now, record Core → syn mappings

3. **Implement Error Translation**

   ```rust
   pub fn translate_rustc_error(
       diagnostic: RustcDiagnostic,
       source_map: &SourceMap,
   ) -> OxurError {
       // 1. Extract rustc position (file:line:col)
       // 2. Map generated Rust position to NodeId
       // 3. Look up original Oxur position via source map
       // 4. Format error with Oxur source context
   }
   ```

4. **Tests:**
   - Mapping preservation through expansion
   - Error translation accuracy
   - Source map lookup performance

**Deliverables:**

- ✅ SourceMap populated through expansion
- ✅ Error translation from rustc to Oxur positions
- ✅ Tests verify mapping accuracy

**Phase 1 Outcome:** Error messages point to original Oxur source

---

## Phase 2 Completion: Stage 4 Integration (3 weeks)

**Goal:** Implement buffer zone architecture per ODD-0013

### Week 1: Update Stage 3 to Output S-expressions

**Tasks:**

1. **Change Lowerer Output Type**

   ```rust
   // Before:
   pub fn lower(&mut self, forms: Vec<CoreForm>) -> Result<syn::File>

   // After:
   pub fn lower(&mut self, forms: Vec<CoreForm>) -> Result<Vec<SExp>>
   ```

2. **Implement S-expression Builders for Rust Concepts**

   ```rust
   impl Lowerer {
       fn lower_item(&mut self, form: CoreForm) -> Result<SExp> {
           match form {
               CoreForm::DefineFunc { name, params, body, .. } => {
                   self.lower_function_to_sexp(name, params, body)
               }
               // ...
           }
       }

       fn lower_function_to_sexp(
           &mut self,
           name: String,
           params: Vec<String>,
           body: CoreForm,
       ) -> Result<SExp> {
           // Generate: (Item :vis (Inherited) :ident (Ident :name "add") :kind (Fn ...))
           SExp::list(vec![
               SExp::symbol("Item"),
               SExp::keyword("vis"),
               SExp::list(vec![SExp::symbol("Inherited")]),
               SExp::keyword("ident"),
               self.lower_ident(&name),
               SExp::keyword("kind"),
               self.lower_fn_item(params, body)?,
           ])
       }
   }
   ```

3. **Tests:**
   - DefineFunc → Oxur AST S-expression
   - Verify S-expression structure matches ODD-0003 spec
   - Round-trip comparison with direct syn generation

**Deliverables:**

- ✅ Stage 3 outputs Oxur AST S-expressions
- ✅ Tests verify S-expression format
- ✅ No regression in functionality

### Week 2: Create Stage 4 Processor

**Tasks:**

1. **New Module: oxur-comp/src/de_sexpr.rs**

   ```rust
   use oxur_ast::Builder;

   pub struct DeSExprProcessor {
       builder: Builder,
   }

   impl DeSExprProcessor {
       pub fn new() -> Self {
           Self { builder: Builder::new() }
       }

       pub fn process(&self, oxur_ast: Vec<SExp>) -> Result<syn::File> {
           let items: Vec<syn::Item> = oxur_ast
               .into_iter()
               .map(|sexp| self.builder.build_item(&sexp))
               .collect::<Result<Vec<_>>>()?;

           Ok(syn::File {
               shebang: None,
               attrs: vec![],
               items,
           })
       }
   }
   ```

2. **Update Compiler Pipeline**

   ```rust
   impl Compiler {
       pub fn compile(&mut self, forms: Vec<CoreForm>, output: &Path) -> Result<()> {
           // Stage 3: Lower to Oxur AST S-expressions
           let oxur_ast = self.lowerer.lower(forms)?;

           // Stage 4: De-S-expression (NEW)
           let syn_ast = self.de_sexpr.process(oxur_ast)?;

           // Stage 5: Generate Rust source
           let source = self.codegen.generate(&syn_ast)?;

           // Stage 6: Compile with rustc
           std::fs::write(&rs_file, source)?;
           self.compile_with_rustc(&rs_file, output)?;

           Ok(())
       }
   }
   ```

3. **Tests:**
   - End-to-end: Oxur source → binary (via Stage 4)
   - Verify same output as before
   - Performance comparison

**Deliverables:**

- ✅ Stage 4 integrated into pipeline
- ✅ Tests pass
- ✅ No performance regression

### Week 3: Remove syn Dependency from oxur-comp

**Tasks:**

1. **Update Cargo.toml**

   ```toml
   [dependencies]
   oxur-ast = { path = "../oxur-ast" }  # NEW
   # syn = "2.0"        # REMOVE
   # quote = "1.0"      # REMOVE
   # proc-macro2 = "1.0" # REMOVE
   ```

2. **Verify Compilation**
   - Ensure oxur-comp builds without syn
   - Only oxur-ast depends on syn
   - Clean dependency graph

3. **Update Documentation**
   - Document buffer zone architecture
   - Update design docs
   - Add architectural diagram

**Deliverables:**

- ✅ oxur-comp no longer depends on syn
- ✅ Buffer zone architecture complete
- ✅ Documentation updated

**Phase 2 Outcome:** Architectural compliance with ODD-0013, buffer zone protection

---

## Phase 3 Completion: Core Forms Expansion (3 weeks)

**Goal:** Expand Core Forms to support real programs

### Week 1: Operators and Calls

**Tasks:**

1. **Add to Core Forms**

   ```rust
   pub enum CoreForm {
       // Existing...
       DefineFunc { ... },
       IfExpr { ... },
       MatchExpr { ... },

       // NEW:
       BinaryOp {
           id: NodeId,
           op: BinOp,  // Add, Sub, Mul, Div, Eq, Ne, Lt, Gt, etc.
           left: Box<CoreForm>,
           right: Box<CoreForm>,
       },

       UnaryOp {
           id: NodeId,
           op: UnOp,  // Neg, Not
           operand: Box<CoreForm>,
       },

       Call {
           id: NodeId,
           func: Box<CoreForm>,
           args: Vec<CoreForm>,
       },

       MethodCall {
           id: NodeId,
           receiver: Box<CoreForm>,
           method: String,
           args: Vec<CoreForm>,
       },
   }
   ```

2. **Update Expander**

   ```rust
   impl Expander {
       fn expand_form(&mut self, form: SurfaceForm) -> Result<CoreForm> {
           match form {
               SurfaceForm::List(elements) => {
                   if let Some(SurfaceForm::Symbol(first)) = elements.first() {
                       match first.as_str() {
                           "+" | "-" | "*" | "/" => self.expand_binary_op(elements),
                           "deffn" => self.expand_deffn(elements),
                           _ => self.expand_call(elements),  // Function call
                       }
                   }
               }
               // ...
           }
       }

       fn expand_binary_op(&mut self, elements: Vec<SurfaceForm>) -> Result<CoreForm> {
           // (+ a b) → BinaryOp { op: Add, left: a, right: b }
       }

       fn expand_call(&mut self, elements: Vec<SurfaceForm>) -> Result<CoreForm> {
           // (func arg1 arg2) → Call { func, args: [arg1, arg2] }
       }
   }
   ```

3. **Update Lowerer**

   ```rust
   impl Lowerer {
       fn lower_expr(&mut self, form: CoreForm) -> Result<SExp> {
           match form {
               CoreForm::BinaryOp { op, left, right, .. } => {
                   self.lower_binary_op(op, *left, *right)
               }
               CoreForm::Call { func, args, .. } => {
                   self.lower_call(*func, args)
               }
               // ...
           }
       }
   }
   ```

4. **Tests:**
   - Arithmetic: `(+ 1 2)` → Rust binary op
   - Calls: `(add 1 2)` → Rust function call
   - Method calls: `(x:pow 2)` → Rust method call

**Deliverables:**

- ✅ Operators work in Core Forms
- ✅ Function calls work
- ✅ Can compile: `(deffn add (a b) (+ a b))`

### Week 2: Local Bindings and Type Annotations

**Tasks:**

1. **Add to Core Forms**

   ```rust
   pub enum CoreForm {
       // Existing...

       // NEW:
       Let {
           id: NodeId,
           bindings: Vec<(String, Option<Type>, CoreForm)>,  // name, type, value
           body: Box<CoreForm>,
       },

       Def {
           id: NodeId,
           name: String,
           ty: Option<Type>,
           value: Box<CoreForm>,
       },
   }

   pub struct Type {
       pub name: String,  // "i32", "String", etc.
       pub generics: Vec<Type>,
   }
   ```

2. **Update Parser to Recognize Type Annotations**

   ```rust
   // Parse: (def x:i32 42)
   // Parse: (deffn add (a:i32 b:i32) (:> i32) (+ a b))
   ```

3. **Update Expander**

   ```rust
   impl Expander {
       fn expand_form(&mut self, form: SurfaceForm) -> Result<CoreForm> {
           match form {
               SurfaceForm::List(elements) => {
                   if let Some(SurfaceForm::Symbol(first)) = elements.first() {
                       match first.as_str() {
                           "let" => self.expand_let(elements),
                           "def" => self.expand_def(elements),
                           // ...
                       }
                   }
               }
               // ...
           }
       }
   }
   ```

4. **Update Lowerer**
   - Lower Let → Rust let statements
   - Lower Def → Rust top-level static/const
   - Include type annotations in generated Rust

5. **Tests:**
   - `(let ((x 42)) (+ x 1))` → Rust let binding
   - `(def x:i32 42)` → Rust const/static
   - Type annotations preserved

**Deliverables:**

- ✅ Local bindings work
- ✅ Type annotations supported
- ✅ Can write typed functions

### Week 3: Conditionals and Pattern Matching

**Tasks:**

1. **Update Expander for IfExpr and MatchExpr**
   - Currently defined but not expanded
   - Implement expansion logic

2. **Update Lowerer**
   - Lower IfExpr → Rust if expression
   - Lower MatchExpr → Rust match expression

3. **Tests:**
   - `(if (> x 0) "pos" "neg")` → Rust if
   - `(match x (Some v) v None 0)` → Rust match

**Deliverables:**

- ✅ Conditionals work
- ✅ Pattern matching works
- ✅ Can write control flow logic

**Phase 3 Outcome:** Can compile real, useful Oxur programs

---

## Phase 5 Completion: Core Macros (2 weeks)

**Goal:** Build essential macro library

### Week 1: Core Macro Framework

**Tasks:**

1. **Design Macro System**

   ```rust
   pub struct MacroRegistry {
       macros: HashMap<String, CoreMacro>,
   }

   pub trait CoreMacro {
       fn expand(&self, args: &[SurfaceForm]) -> Result<CoreForm>;
   }
   ```

2. **Implement Control Flow Macros**
   - `when` → if without else
   - `unless` → negated if
   - `cond` → multi-way conditional

3. **Tests:**
   - Each macro expands correctly
   - Nested macro expansion

**Deliverables:**

- ✅ Macro framework in place
- ✅ 3-5 essential macros working

### Week 2: Threading and Let Variants

**Tasks:**

1. **Threading Macros**
   - `->` thread-first
   - `->>` thread-last
   - `as->` thread-as

2. **Let Variants**
   - `when-let` conditional binding
   - `if-let` conditional with else

3. **Tests:**
   - Threading transformations correct
   - Let variants work as expected

**Deliverables:**

- ✅ Core macro library complete
- ✅ Documentation for each macro

**Phase 5 Outcome:** Idiomatic Oxur programming with macros

---

## Phase 6 Completion: REPL Subprocess IPC (2 weeks)

**Goal:** Enable Tier 2/3 execution

### Week 1: Design and Implement IPC Protocol

**Tasks:**

1. **Design Message Protocol**

   ```rust
   #[derive(Serialize, Deserialize)]
   pub enum SubprocessMessage {
       Load { library_path: PathBuf },
       Run { function_name: String, args: Vec<Value> },
       Result(Value),
       Error(String),
   }
   ```

2. **Implement Communication Channel**
   - Option A: stdin/stdout with JSON messages
   - Option B: Unix domain sockets
   - Recommendation: stdin/stdout (simpler, portable)

3. **Update SubprocessExecutor**

   ```rust
   impl SubprocessExecutor {
       pub fn load_library(&mut self, path: &Path) -> Result<()> {
           let msg = SubprocessMessage::Load { library_path: path.to_path_buf() };
           self.send_message(&msg)?;
           self.recv_response()?;
           Ok(())
       }

       pub fn run_function(&mut self, name: &str, args: &[Value]) -> Result<Value> {
           let msg = SubprocessMessage::Run {
               function_name: name.to_string(),
               args: args.to_vec()
           };
           self.send_message(&msg)?;

           match self.recv_response()? {
               SubprocessMessage::Result(value) => Ok(value),
               SubprocessMessage::Error(err) => Err(err.into()),
               _ => Err("Unexpected response".into()),
           }
       }
   }
   ```

4. **Implement Subprocess Binary**

   ```rust
   // src/bin/subprocess.rs
   fn main() {
       let mut loaded_libs = HashMap::new();

       loop {
           let msg: SubprocessMessage = read_message_from_stdin()?;

           match msg {
               SubprocessMessage::Load { library_path } => {
                   let lib = load_dynamic_library(&library_path)?;
                   loaded_libs.insert(library_path.clone(), lib);
                   send_message(&SubprocessMessage::Result(Value::Unit))?;
               }
               SubprocessMessage::Run { function_name, args } => {
                   let result = call_function(&loaded_libs, &function_name, &args)?;
                   send_message(&SubprocessMessage::Result(result))?;
               }
               _ => {}
           }
       }
   }
   ```

5. **Tests:**
   - Load library and run function
   - Handle subprocess crashes
   - Ctrl-C support

**Deliverables:**

- ✅ IPC protocol working
- ✅ Load and execute dynamic libraries
- ✅ Tests verify communication

### Week 2: Integration with REPL Tiers

**Tasks:**

1. **Wire into EvalContext**

   ```rust
   impl EvalContext {
       pub async fn eval(&mut self, code: &str) -> Result<EvalResult> {
           // Tier 1: Calculator
           if let Some(result) = self.try_calculator(code)? {
               return Ok(result);
           }

           // Tier 2: Cached compilation + subprocess execution (NEW)
           if let Some(cached_lib) = self.cache.get(&cache_key) {
               let result = self.executor.run_function("eval", &[])?;
               return Ok(result);
           }

           // Tier 3: JIT compilation + subprocess execution (NEW)
           let lib_path = self.compiler.compile(forms)?;
           self.executor.load_library(&lib_path)?;
           let result = self.executor.run_function("eval", &[])?;
           Ok(result)
       }
   }
   ```

2. **End-to-End REPL Tests**
   - Start server
   - Create session
   - Eval simple expression (Tier 1)
   - Eval function definition (Tier 3, then Tier 2 on repeat)
   - Verify caching works

3. **Performance Testing**
   - Measure Tier 1 latency (<1ms)
   - Measure Tier 2 latency (1-5ms target)
   - Measure Tier 3 latency (50-300ms)

**Deliverables:**

- ✅ Three-tier execution working
- ✅ REPL fully functional
- ✅ Performance meets targets

**Phase 6 Outcome:** Working REPL with fast iteration

---

## Timeline Summary

| Phase | Tasks | Duration | Dependencies |
|-------|-------|----------|--------------|
| **Phase 1 Complete** | Source mapping | 2 weeks | None |
| **Phase 2 Complete** | Stage 4 integration | 3 weeks | Phase 1 (for error translation) |
| **Phase 3 Complete** | Core Forms expansion | 3 weeks | Phase 2 (for lowering) |
| **Phase 5** | Core macros | 2 weeks | Phase 3 (needs expanded Core Forms) |
| **Phase 6 Complete** | REPL subprocess IPC | 2 weeks | Phase 3 (needs working compiler) |
| **Phase 7** | CLI polish | 1 week | Phases 1-6 complete |
| **Phase 8** | v1.0 release | 2 weeks | All phases complete |

**Total:** ~15 weeks (3.5 months) to v1.0

---

## Parallelization Opportunities

If multiple people are working:

**Track 1: Compilation Pipeline** (Phases 1-3, 5)

- Week 1-2: Source mapping
- Week 3-5: Stage 4 integration
- Week 6-8: Core Forms expansion
- Week 9-10: Core macros

**Track 2: REPL Infrastructure** (Phase 6)

- Week 1-2: Subprocess IPC protocol
- Week 3-4: REPL integration

**Track 3: Documentation & Polish** (Phases 7-8)

- Ongoing: Documentation
- Week 11-13: CLI improvements
- Week 14-15: Release prep

**Savings:** With 3 parallel tracks, could complete in ~11 weeks instead of 15

---

## Success Criteria

**Phase 1 Complete:**

- ✅ Error messages point to Oxur source
- ✅ Line/column accuracy

**Phase 2 Complete:**

- ✅ oxur-comp no longer depends on syn
- ✅ Buffer zone architecture verified
- ✅ Can swap out syn without touching oxur-comp

**Phase 3 Complete:**

- ✅ Can compile real programs with operators, calls, control flow
- ✅ Example: Fibonacci function works

**Phase 5 Complete:**

- ✅ 10+ core macros working
- ✅ Idiomatic Oxur code possible

**Phase 6 Complete:**

- ✅ REPL three-tier execution working
- ✅ Tier 1: <1ms, Tier 2: 1-5ms, Tier 3: 50-300ms
- ✅ Can define and test functions interactively

**v1.0 Release:**

- ✅ All ODD-0013 phases complete
- ✅ Documentation complete
- ✅ Example programs work
- ✅ Ready for public use

---

## Risk Mitigation

**Risk:** Stage 4 integration breaks existing tests

- **Mitigation:** Comprehensive test suite before refactoring, incremental changes

**Risk:** Source mapping adds significant overhead

- **Mitigation:** Performance benchmarks, optimize hot paths

**Risk:** Subprocess IPC adds latency

- **Mitigation:** Measure before/after, optimize protocol, consider persistent subprocess

**Risk:** Core Forms expansion introduces bugs

- **Mitigation:** Add each feature incrementally with tests, property-based testing

---

## Next Actions

**Immediate (This Week):**

1. Review this plan
2. Decide on prioritization (sequential vs. parallel)
3. Start Phase 1: Source mapping (Week 1)

**Next Week:**

1. Complete Phase 1 Week 1 (position tracking)
2. Begin Phase 1 Week 2 (mapping chains)

**Month 1:**

- Complete Phase 1 (source mapping)
- Complete Phase 2 (Stage 4 integration)
- Start Phase 3 (Core Forms expansion)

**Month 2:**

- Complete Phase 3 (Core Forms expansion)
- Complete Phase 5 (core macros)
- Start Phase 6 (REPL IPC)

**Month 3:**

- Complete Phase 6 (REPL IPC)
- Phase 7 (CLI polish)
- Phase 8 (release prep)

---

## Conclusion

This plan completes the ODD-0013 vision:

- ✅ 6-stage compilation pipeline with buffer zone
- ✅ Full source mapping for accurate errors
- ✅ Comprehensive Core Forms for real programs
- ✅ Core macro library for idiomatic code
- ✅ Working REPL with three-tier execution
- ✅ v1.0 ready for users

**Estimated:** 15 weeks sequential, 11 weeks with 3-person team

The architecture will be solid, the compiler will be useful, and the foundation will be ready for future enhancements (VM interpretation, user macros, multi-target compilation).
