# Phase 4 Source Map Integration - Analysis Report

**Date:** 2026-01-06
**Status:** 🔄 **PARTIAL** (~50% Complete - Scaffolding exists, integration needed)
**Tests:** 257 passing (3 ErrorTranslator tests)
**Coverage:** 87.22% overall

---

## Executive Summary

**Phase 4 is PARTIALLY COMPLETE.** The ErrorTranslator infrastructure exists but source map integration is missing.

**What EXISTS:**
- ✅ ErrorTranslator struct (347 lines)
- ✅ Rustc JSON error parsing
- ✅ Error formatting methods
- ✅ Integration with CachedCompiler
- ✅ oxur-smap dependency
- ✅ 3 unit tests passing

**What's MISSING:**
- ❌ ariadne dependency for pretty error display
- ❌ regex dependency for source map comment parsing
- ❌ Actual source map lookup (line 221 TODO)
- ❌ Source map threading through pipeline
- ❌ /* oxur_node=N */ comment generation in RustAstWrapper
- ❌ SourceMap integration in EvalContext

**Completion:** ~50% (scaffolding done, integration needed)

---

## What ODD-0040 Phase 4 Requires

**From lines 1256-1417 of ODD-0040:**

### Task 4.1: Thread SourceMap Through Pipeline ❌

**Required:**
```rust
// oxur-lang API (already exists per spec)
pub fn parse_lisp(
    source: &str,
    source_map: &mut SourceMap,
) -> Result<SurfaceForms, ParseError>;

pub fn expand(
    surface: SurfaceForms,
    source_map: &mut SourceMap,
) -> Result<CoreForms, ExpandError>;

// oxur-comp API (already exists per spec)
pub fn lower(
    core: &CoreForm,
    source_map: &mut SourceMap,
) -> Result<syn::File, LowerError>;

// In EvalContext.compile_and_execute():
let mut source_map = SourceMap::new();
let surface = parse_lisp(code, &mut source_map)?;
let core = expand(surface, &mut source_map)?;
let rust_ast = lower(&core, &mut source_map)?;

// In RustAstWrapper.wrap():
let wrapped = wrapper.wrap_with_source_map(&cache_key, code, &source_map)?;
// Generated code contains /* oxur_node=123 */ comments
```

**Current Status:** ❌ Not implemented
- EvalContext doesn't create/thread SourceMap
- RustAstWrapper doesn't accept SourceMap parameter
- No /* oxur_node=N */ comments generated

### Task 4.2: ErrorTranslator with Source Map Lookup ⚠️

**Required:**
```rust
fn translate_span(&self, span: &RustcSpan) -> Result<TranslatedSpan> {
    if let Some(source_map) = &self.source_map {
        // 1. Read source line
        let source = std::fs::read_to_string(&span.file_name)?;
        let line_content = source.lines().nth(span.line_start - 1)?;

        // 2. Find /* oxur_node=N */ comment near column
        let pattern = regex::Regex::new(r"/\* oxur_node=(\d+) \*/").ok()?;
        let node_id = extract_nearest_node_id(line_content, span.column_start)?;

        // 3. Look up original position
        let original_pos = source_map.lookup(node_id)?;

        return Ok(TranslatedSpan {
            pos: original_pos,
            label: span.label.clone(),
            source_text: Some(extract_source_text(source, &original_pos)),
        });
    }

    // Fallback to Rust positions
    Ok(TranslatedSpan { ... })
}
```

**Current Status:** ⚠️ Stub (line 221 TODO in error_translator.rs)

**From error_translator.rs:217-238:**
```rust
fn translate_span(&self, span: &RustcSpan) -> Result<TranslatedSpan> {
    // If no source map, return Rust positions as-is
    let pos = if let Some(_source_map) = &self.source_map {
        // TODO: Implement actual source map lookup
        // For now, create a placeholder position
        SourcePos::repl(
            span.line_start as u32,
            span.column_start as u32,
            (span.byte_end - span.byte_start) as u32,
        )
    } else {
        // No source map - use Rust positions directly
        SourcePos::repl(
            span.line_start as u32,
            span.column_start as u32,
            (span.byte_end - span.byte_start) as u32,
        )
    };

    Ok(TranslatedSpan { pos, label: span.label.clone(), source_text: None })
}
```

### Task 4.3: Pretty Error Display with ariadne ❌

**Required:**
```rust
use ariadne::{Report, ReportKind, Label, Source};

pub fn display_error(error: &TranslatedDiagnostic, source: &str) {
    let report = Report::build(
        ReportKind::Error,
        &error.position.file,
        error.position.column as usize,
    )
    .with_code(error.code.as_deref().unwrap_or("E????"))
    .with_message(&error.message)
    .with_label(
        Label::new((
            &error.position.file,
            error.position.column as usize
                ..(error.position.column + error.position.length) as usize,
        ))
        .with_message(&error.message),
    );

    report
        .finish()
        .print(Source::from(source))
        .expect("Failed to print error");
}
```

**Current Status:** ❌ Not implemented
- ariadne not in dependencies
- Only basic format() method exists (lines 248-287)
- No rich error display

---

## What EXISTS: ErrorTranslator Infrastructure

### File: `src/compiler/error_translator.rs` (347 lines) ✅

**Complete Structures:**

```rust
// Error types
#[derive(Debug, Error)]
pub enum TranslationError {
    JsonParseFailed(#[from] serde_json::Error),
    NoSourceMap,
    LookupFailed(String),
}

// Rustc diagnostic types (Serde deserialization)
pub enum DiagnosticLevel { Error, Warning, Note, Help, FailureNote, Other }
pub struct RustcSpan { file_name, byte_start, byte_end, line_start, column_start, ... }
pub struct RustcDiagnostic { message, code, level, spans, children, rendered }
pub struct RustcCode { code, explanation }

// Translated types
pub struct TranslatedDiagnostic { message, level, code, primary_span, children, ... }
pub struct TranslatedSpan { pos: SourcePos, label, source_text }

// Main struct
pub struct ErrorTranslator {
    source_map: Option<SourceMap>,  // ⚠️ Not actually used yet
}
```

**Complete Methods:**

✅ **`new()`** - Create without source map
✅ **`with_source_map(source_map)`** - Create with source map
✅ **`parse_and_translate(stderr)`** - Parse rustc JSON output (lines 155-183)
✅ **`translate_diagnostic(diag)`** - Translate single diagnostic (lines 186-215)
⚠️ **`translate_span(span)`** - Translate span (lines 217-238) - **HAS TODO**
✅ **`format()`** - Format diagnostic as string (lines 248-287)

**Integration:**

✅ **In `src/compiler/cached.rs` (lines 174-190):**
```rust
use crate::compiler::ErrorTranslator;
let translator = ErrorTranslator::new();  // ⚠️ No source map!

match translator.parse_and_translate(&stderr) {
    Ok(diagnostics) if !diagnostics.is_empty() => {
        // Format all diagnostics
        let formatted = diagnostics
            .iter()
            .map(|d| d.format())
            .collect::<Vec<_>>()
            .join("\n\n");

        return Err(CompilerError::RustcFailed {
            stderr: formatted,
            exit_code: output.status.code(),
        });
    }
    _ => {
        // Fallback to raw stderr
        return Err(CompilerError::RustcFailed {
            stderr: stderr.to_string(),
            exit_code: output.status.code(),
        });
    }
}
```

✅ **Exported in `src/compiler/mod.rs` (lines 11-14):**
```rust
pub use error_translator::{
    DiagnosticLevel, ErrorTranslator, RustcDiagnostic, TranslatedDiagnostic,
    TranslatedSpan, TranslationError,
};
```

**Tests (3 passing):**

✅ **test_error_translator_creation** - Basic construction
✅ **test_parse_simple_error** - Parse rustc JSON with E0425 error
✅ **test_format_diagnostic** - Format error message
✅ **test_parse_empty_input** - Handle empty stderr

---

## What's MISSING: Integration & Dependencies

### 1. Missing Dependencies ❌

**From Cargo.toml:**
```toml
[dependencies]
# Has oxur-smap ✅
oxur-smap = { path = "../oxur-smap" }

# Missing for Phase 4:
# ariadne = "0.4"      ❌ For pretty error display
# regex = "1.0"        ❌ For parsing /* oxur_node=N */ comments
```

### 2. Missing Source Map in RustAstWrapper ❌

**Current (src/wrapper.rs):**
```rust
impl RustAstWrapper {
    pub fn wrap(&self, cache_key: &str, user_code: &str) -> Result<String> {
        // Generates code WITHOUT source map comments
        quote! {
            #[no_mangle]
            pub extern "C" fn #fn_name() {
                #(#stmts)*  // No /* oxur_node=N */ annotations!
            }
        }
    }
}
```

**Needed:**
```rust
impl RustAstWrapper {
    pub fn wrap_with_source_map(
        &self,
        cache_key: &str,
        user_code: &str,
        source_map: &SourceMap,
    ) -> Result<String> {
        // Generate code WITH source map comments
        let stmts_with_comments = annotate_statements(stmts, source_map);

        quote! {
            #[no_mangle]
            pub extern "C" fn #fn_name() {
                #(#stmts_with_comments)*  // Has /* oxur_node=123 */ etc.
            }
        }
    }

    fn annotate_statements(
        &self,
        stmts: Vec<Stmt>,
        source_map: &SourceMap,
    ) -> Vec<TokenStream> {
        stmts.into_iter().map(|stmt| {
            // Find NodeId for this statement
            if let Some(node_id) = get_node_id_for_stmt(&stmt, source_map) {
                quote! {
                    /* oxur_node=#node_id */ #stmt
                }
            } else {
                quote! { #stmt }
            }
        }).collect()
    }
}
```

### 3. Missing SourceMap Threading in EvalContext ❌

**Current (src/eval/context.rs:440-556):**
```rust
async fn compile_and_execute(&mut self, code: &str) -> Result<...> {
    // Step 1: Parse
    let core_forms = match self.mode {
        ReplMode::Lisp => self.lisp_eval.parse(code)?,    // ❌ No source_map
        ReplMode::Sexpr => vec![self.sexpr_eval.parse_to_core(code)?],
    };

    // Step 2: Expand (TODO line 482)
    // ❌ No source_map

    // Step 3: Wrap
    let wrapped_code = self.wrapper.wrap(&cache_key, code)?;  // ❌ No source_map

    // Continue...
}
```

**Needed:**
```rust
async fn compile_and_execute(&mut self, code: &str) -> Result<...> {
    // Create source map
    let mut source_map = SourceMap::new();

    // Step 1: Parse (thread source_map)
    let surface = parse_lisp(code, &mut source_map)?;

    // Step 2: Expand (thread source_map)
    let core = expand(surface, &mut source_map)?;

    // Step 3: Lower to Rust (thread source_map)
    let rust_ast = lower(&core, &mut source_map)?;

    // Step 4: Wrap with source map comments
    let wrapped_code = self.wrapper.wrap_with_source_map(
        &cache_key,
        &rust_ast,
        &source_map,
    )?;

    // Step 5: Compile
    let lib_path = compiler.compile(&cache_key, &wrapped_code, 2)?;

    // Step 6: Execute
    let exec_result = executor.execute(&cache_key)?;

    // If errors, use source_map for translation
    if let Err(e) = exec_result {
        let translator = ErrorTranslator::with_source_map(source_map);
        let translated = translator.translate(e)?;
        return Err(translated);
    }

    Ok(exec_result)
}
```

### 4. Missing ariadne Pretty Display ❌

**Current:**
Only basic text formatting in `TranslatedDiagnostic::format()` (lines 248-287):
```
error[E0425]: cannot find value `x` in this scope
 --> line 10, column 15
  | not found in this scope
```

**Needed:**
Rich, colorful error display like rustc:
```
error[E0425]: cannot find value `x` in this scope
  --> <repl>:3:15
   |
 3 | (def y (+ x 10))
   |           ^ not found in this scope
   |
   = help: maybe you meant to define it first: (def x:i32 42)
```

---

## Architecture Gap Analysis

### Current Flow (Partial) ⚠️

```
User Code (Oxur)
    ↓
EvalContext.compile_and_execute()
    ↓
Parse (NO source map) ❌
    ↓
[Expand - TODO] ❌
    ↓
Wrap (NO source map comments) ❌
    ↓
Compile
    ↓
If Error:
  ErrorTranslator.parse_and_translate()
    ↓
  translate_span() - TODO! ⚠️
    ↓
  Basic format() ✅
    ↓
Return error to user
```

**Problems:**
1. No source map created
2. No source map threaded through pipeline
3. Generated code has no /* oxur_node=N */ comments
4. Error translator can't look up original positions
5. Errors show Rust positions, not Oxur positions

### Required Flow (ODD-0040) ✅

```
User Code (Oxur)
    ↓
EvalContext.compile_and_execute()
    ↓
SourceMap::new() ✅
    ↓
parse_lisp(code, &mut source_map) ✅
    ↓
expand(surface, &mut source_map) ✅
    ↓
lower(core, &mut source_map) ✅
    ↓
wrapper.wrap_with_source_map(..., &source_map) ✅
  → Generates /* oxur_node=123 */ comments ✅
    ↓
Compile annotated code
    ↓
If Error:
  ErrorTranslator::with_source_map(source_map)
    ↓
  translate_span():
    1. Read generated .rs file ✅
    2. Parse /* oxur_node=N */ with regex ✅
    3. source_map.lookup(node_id) → original SourcePos ✅
    ↓
  ariadne::Report::build() ✅
    → Rich error display with original positions ✅
    ↓
Return beautiful error to user
```

---

## Implementation Tasks

### Task 4.1: Add Missing Dependencies ⏸️

**In Cargo.toml:**
```toml
[dependencies]
ariadne = "0.4"
regex = "1.10"
```

**Estimated:** 5 minutes

### Task 4.2: Implement Source Map Lookup in ErrorTranslator ⏸️

**In src/compiler/error_translator.rs (line 217-238):**

Replace TODO with actual implementation:

```rust
fn translate_span(&self, span: &RustcSpan) -> Result<TranslatedSpan> {
    let pos = if let Some(source_map) = &self.source_map {
        // 1. Read source file
        let source = std::fs::read_to_string(&span.file_name)
            .map_err(|e| TranslationError::LookupFailed(format!("Failed to read {}: {}", span.file_name, e)))?;

        // 2. Get line content
        let line_content = source.lines().nth(span.line_start - 1)
            .ok_or_else(|| TranslationError::LookupFailed(format!("Line {} not found", span.line_start)))?;

        // 3. Find /* oxur_node=N */ comment near column
        let node_id = extract_node_id_near_column(line_content, span.column_start)?;

        // 4. Look up original position
        source_map.lookup(node_id)
            .ok_or_else(|| TranslationError::LookupFailed(format!("NodeId {} not in source map", node_id)))?
    } else {
        // No source map - use Rust positions
        SourcePos::repl(
            span.line_start as u32,
            span.column_start as u32,
            (span.byte_end - span.byte_start) as u32,
        )
    };

    Ok(TranslatedSpan {
        pos,
        label: span.label.clone(),
        source_text: Some(extract_source_text_at_pos(&pos)),
    })
}

fn extract_node_id_near_column(line: &str, column: usize) -> Result<NodeId> {
    use regex::Regex;

    let pattern = Regex::new(r"/\* oxur_node=(\d+) \*/")
        .map_err(|e| TranslationError::LookupFailed(format!("Regex error: {}", e)))?;

    // Find all node_id comments in line
    let mut best_match: Option<(usize, u32)> = None;

    for cap in pattern.captures_iter(line) {
        let match_start = cap.get(0).unwrap().start();
        let node_id: u32 = cap.get(1).unwrap().as_str().parse()
            .map_err(|e| TranslationError::LookupFailed(format!("Invalid node_id: {}", e)))?;

        let distance = (match_start as i32 - column as i32).abs() as usize;

        if best_match.is_none() || distance < best_match.unwrap().0 {
            best_match = Some((distance, node_id));
        }
    }

    best_match
        .map(|(_, id)| NodeId::from(id))
        .ok_or_else(|| TranslationError::LookupFailed("No oxur_node comment found".to_string()))
}
```

**Estimated:** 2-3 hours

### Task 4.3: Add SourceMap to RustAstWrapper ⏸️

**In src/wrapper.rs:**

1. Add `wrap_with_source_map()` method
2. Generate /* oxur_node=N */ comments for each statement/expression
3. Keep existing `wrap()` for backward compatibility

**Estimated:** 1-2 days

**Example Implementation:**
```rust
pub fn wrap_with_source_map(
    &self,
    cache_key: &str,
    code: &str,
    source_map: &SourceMap,
) -> Result<String> {
    // Parse code
    let stmts = syn::parse_str::<syn::File>(code)?;

    // Annotate with source map comments
    let annotated = self.annotate_with_source_map(&stmts, source_map);

    // Wrap in function
    let fn_name = format_ident!("oxur_eval_{}", cache_key);

    let wrapped = quote! {
        #![allow(unused)]

        #[no_mangle]
        pub extern "C" fn #fn_name() {
            #annotated
        }
    };

    Ok(prettyplease::unparse(&syn::parse2(wrapped)?))
}

fn annotate_with_source_map(
    &self,
    file: &syn::File,
    source_map: &SourceMap,
) -> TokenStream {
    // For each statement, add /* oxur_node=N */ comment
    let stmts_with_comments = file.items.iter().map(|item| {
        // Look up NodeId for this item's span
        if let Some(node_id) = self.find_node_id_for_span(&item.span(), source_map) {
            let comment = format!("/* oxur_node={} */", node_id);
            quote! {
                #[doc = #comment]
                #item
            }
        } else {
            quote! { #item }
        }
    });

    quote! {
        #(#stmts_with_comments)*
    }
}
```

### Task 4.4: Thread SourceMap Through EvalContext Pipeline ⏸️

**In src/eval/context.rs:**

Update `compile_and_execute()` to:
1. Create SourceMap
2. Pass to parse/expand/lower
3. Pass to wrapper
4. Use in ErrorTranslator

**Estimated:** 2-3 days

**Note:** Depends on oxur-lang and oxur-comp API support

### Task 4.5: Add ariadne Pretty Error Display ⏸️

**In src/compiler/error_translator.rs:**

Add method:
```rust
impl TranslatedDiagnostic {
    pub fn display_with_ariadne(&self, source: &str) -> String {
        use ariadne::{Report, ReportKind, Label, Source, Color};

        let kind = match self.level {
            DiagnosticLevel::Error => ReportKind::Error,
            DiagnosticLevel::Warning => ReportKind::Warning,
            _ => ReportKind::Advice,
        };

        let mut report = Report::build(kind, "<repl>", 0)
            .with_message(&self.message);

        if let Some(code) = &self.code {
            report = report.with_code(code);
        }

        if let Some(span) = &self.primary_span {
            let start = span.pos.byte_offset();
            let end = start + span.pos.length as usize;

            let mut label = Label::new(("<repl>", start..end))
                .with_color(Color::Red);

            if let Some(msg) = &span.label {
                label = label.with_message(msg);
            }

            report = report.with_label(label);
        }

        // Add secondary spans and children...

        let mut output = Vec::new();
        report.finish()
            .write(Source::from(source), &mut output)
            .expect("Failed to write report");

        String::from_utf8(output).unwrap()
    }
}
```

**Estimated:** 1-2 days

### Task 4.6: Integration Testing ⏸️

**New tests needed:**
1. test_source_map_annotation_generation
2. test_error_position_translation
3. test_ariadne_error_display
4. test_end_to_end_with_error
5. test_multiple_errors_translation

**Estimated:** 1-2 days

---

## Blockers & Dependencies

### Hard Blocker: oxur-lang & oxur-comp APIs ⚠️

**Phase 4 requires:**
```rust
// These functions must exist and accept source_map:
oxur_lang::parse_lisp(code, &mut source_map)
oxur_lang::expand(surface, &mut source_map)
oxur_comp::lower(core, &mut source_map)
```

**Current Status:** Unknown
- Need to check if oxur-lang crate exists
- Need to check if oxur-comp crate exists
- Need to check if they have source_map parameters

**If they don't exist:**
- Phase 4 is BLOCKED until Phase 6+ implements them
- Can implement ErrorTranslator improvements as prep work
- Can add ariadne for current error display
- Full source map flow requires complete compiler

### Soft Blocker: oxur-smap API

**Need to verify oxur-smap supports:**
```rust
impl SourceMap {
    pub fn new() -> Self;
    pub fn lookup(&self, node_id: NodeId) -> Option<SourcePos>;
    pub fn record(&mut self, node_id: NodeId, pos: SourcePos);
}
```

**Check:**
```bash
cd /Users/oubiwann/lab/oxur/oxur/crates/oxur-smap
cargo doc --open
```

---

## Risk Assessment

### Low Risk ✅

- ✅ ErrorTranslator scaffolding complete
- ✅ Rustc JSON parsing working
- ✅ oxur-smap dependency exists
- ✅ Integration in CachedCompiler proven

### Medium Risk ⚠️

- ⚠️ Regex for parsing source map comments (straightforward but needs testing)
- ⚠️ ariadne API (well-documented library)
- ⚠️ Source map comment generation in quote! (need to test formatting)

### High Risk ⚠️⚠️

- ⚠️⚠️ **oxur-lang/oxur-comp dependencies** - If these don't exist or don't support source_map, Phase 4 is blocked
- ⚠️⚠️ **Source map accuracy** - Ensuring NodeId annotations map correctly through transformations
- ⚠️⚠️ **Performance impact** - Source map comment generation may slow compilation

---

## Phase 4 Completion Criteria (from ODD-0040)

- [ ] SourceMap correctly populated during parsing
- [ ] SourceMap correctly populated during expansion
- [ ] SourceMap correctly populated during lowering
- [ ] Error translation finds correct original position
- [ ] Error messages display with correct file/line/column
- [ ] ariadne produces beautiful error output
- [ ] All tests pass

**Current Status:**
- [ ] ❌ SourceMap not populated (blocked on oxur-lang/oxur-comp)
- [ ] ⚠️ Error translation exists but uses TODO stub
- [ ] ❌ Error messages show Rust positions, not Oxur
- [ ] ❌ ariadne not integrated
- [x] ✅ All 257 tests pass (but Phase 4 tests missing)

---

## Implementation Strategy

### Option A: Wait for oxur-lang/oxur-comp ⏸️

**Pros:**
- Implements Phase 4 as designed
- Full source map flow working
- Proper error position translation

**Cons:**
- Blocked until compiler phases complete
- Could be weeks/months of waiting
- Phase 4 provides immediate value for debugging

**Timeline:** Unknown (depends on Phase 6+)

### Option B: Incremental Implementation 🎯 **RECOMMENDED**

**Step 1: Prep Work (Can do now)**
1. Add ariadne & regex dependencies
2. Implement source map comment parsing (without generation)
3. Add ariadne error display
4. Write tests with mock source maps

**Step 2: When oxur-lang/oxur-comp Ready**
1. Thread SourceMap through pipeline
2. Generate source map comments in wrapper
3. Enable full position translation
4. Update tests to use real source maps

**Pros:**
- Makes progress now
- Improves error display immediately
- Ready to integrate when compiler available
- Tests infrastructure in place

**Cons:**
- Two-phase implementation
- Some duplication of effort

**Timeline:**
- Step 1: 1 week (prep work)
- Step 2: 1 week (when ready)

### Option C: Stub Completion ❌ **NOT RECOMMENDED**

Complete TODO at line 221 with placeholder that just returns Rust positions.

**Pros:** Quick "completion"

**Cons:**
- Doesn't provide value
- Misleading "complete" status
- Still needs real implementation later

**Timeline:** 1 day (wasted)

---

## Recommended Next Steps

### Immediate: Check Compiler Crates ✅

**Before deciding on strategy:**

```bash
# 1. Check if oxur-lang exists
ls -la /Users/oubiwann/lab/oxur/oxur/crates/oxur-lang/

# 2. Check if oxur-comp exists
ls -la /Users/oubiwann/lab/oxur/oxur/crates/oxur-comp/

# 3. If they exist, check for source_map support
grep -r "source_map\|SourceMap" crates/oxur-lang/src/
grep -r "source_map\|SourceMap" crates/oxur-comp/src/

# 4. Check oxur-smap API
cargo doc --package oxur-smap --open
```

**Based on findings:**
- If oxur-lang/oxur-comp exist with source_map → Full implementation (Option A)
- If they exist without source_map → Wait or add source_map support first
- If they don't exist → Incremental prep work (Option B)

### Short-Term: Prep Work (1 week) 🎯

**Regardless of blocker status, can improve current error handling:**

1. **Add Dependencies** (30 min)
   ```toml
   ariadne = "0.4"
   regex = "1.10"
   ```

2. **Implement Comment Parsing** (2-3 hours)
   - Complete extract_node_id_near_column()
   - Add tests with mock generated code

3. **Add ariadne Display** (1-2 days)
   - Implement display_with_ariadne()
   - Make errors beautiful NOW
   - Test with current Rust positions

4. **Write Tests** (1-2 days)
   - Mock source map tests
   - Error translation tests
   - ariadne output tests

**Value:** Better error display immediately, infrastructure ready for full integration

### Long-Term: Full Integration (when ready)

1. Verify oxur-lang/oxur-comp source_map support
2. Thread SourceMap through EvalContext pipeline
3. Add wrap_with_source_map() to RustAstWrapper
4. Enable full position translation
5. Update tests to use real source maps
6. Verify end-to-end error translation

**Value:** Rustc-quality error messages pointing to original Oxur source

---

## Comparison to Other Phases

### Like Phase 1 & 3 (Already Complete) ✅

- Infrastructure exists
- Tests passing
- Integrated into pipeline

### Like Phase 2 (Needed Implementation) ⚠️

- Has stub/TODO
- Needs real implementation
- Dependencies clear
- Can proceed incrementally

### Unique Challenge ⚠️⚠️

- **Hard blocker on compiler phases**
- Unlike other phases, can't fully complete in isolation
- Value proposition: Better errors NOW vs perfect errors LATER

---

## Success Metrics

### Prep Work Complete ✅

- [ ] ariadne & regex in dependencies
- [ ] Comment parsing implemented
- [ ] ariadne error display working
- [ ] 10+ new tests passing
- [ ] Current errors look better

### Full Integration Complete ✅

- [ ] SourceMap threaded through pipeline
- [ ] /* oxur_node=N */ comments generated
- [ ] Error positions translate to Oxur source
- [ ] ariadne shows original positions
- [ ] End-to-end test: Oxur error → beautiful display
- [ ] 20+ total tests passing

---

## Files Involved

### Existing (Need Modification)

1. **`src/compiler/error_translator.rs`** (347 lines)
   - Complete translate_span() TODO (line 221)
   - Add extract_node_id_near_column()
   - Add display_with_ariadne()

2. **`src/wrapper.rs`** (needs source map support)
   - Add wrap_with_source_map() method
   - Add source map comment generation
   - Keep backward compatibility

3. **`src/eval/context.rs`** (compile_and_execute)
   - Create SourceMap
   - Thread through pipeline
   - Pass to wrapper
   - Use in error handling

4. **`Cargo.toml`**
   - Add ariadne = "0.4"
   - Add regex = "1.10"

### May Need (Dependencies)

5. **`crates/oxur-lang/src/*.rs`** (if exists)
   - Verify source_map support in parse_lisp()
   - Verify source_map support in expand()

6. **`crates/oxur-comp/src/*.rs`** (if exists)
   - Verify source_map support in lower()

7. **`crates/oxur-smap/src/*.rs`**
   - Verify SourceMap::lookup() exists
   - Verify SourceMap::record() exists
   - Check API compatibility

---

## Sign-Off

**Phase 4 Status:** 🔄 **PARTIAL** (~50% complete)

**What Works:** ✅ ErrorTranslator structure, rustc JSON parsing, basic formatting

**What's Missing:** ❌ Source map integration, ariadne display, full position translation

**Blocker:** ⚠️⚠️ Depends on oxur-lang/oxur-comp (unknown status)

**Recommended:** 🎯 Option B (Incremental prep work + full integration when ready)

**Next Action:** ✅ Check if oxur-lang/oxur-comp exist and have source_map support

**Estimated Completion:**
- Prep work: 1 week (can start now)
- Full integration: 1 week (when compiler ready)
- **Total: 2 weeks** (but may span longer timeline if compiler not ready)

---

**Report Generated By:** Claude Code (Sonnet 4.5)
**Implementation Plan:** ODD-0040
**Architecture Spec:** ODD-0038 v1.2
**Phase 4 Analysis:** Partial implementation, needs blocker check + incremental approach

**Time to check for blockers and decide on strategy!** 🔍
