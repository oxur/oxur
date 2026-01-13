# Stage 1.8 Implementation Plan: Core → syn Mapping

**Date:** 2026-01-12
**Stage:** 1.8 from Phase 1 (Source Mapping Infrastructure)
**Deliverable:** Update Lowerer to record mappings (temporary until Stage 4)
**Estimated Time:** 3 hours
**Dependencies:** Stage 1.7 (Surface → Core mapping) ✅

---

## Current State Analysis

### What Exists ✅

**Lowerer Structure** (`crates/oxur-comp/src/lowering.rs:11-14`):
```rust
pub struct Lowerer {
    #[allow(dead_code)]
    node_map: HashMap<NodeId, syn::Expr>,
}
```

**CoreForm with NodeId** (from oxur-lang):
```rust
pub enum CoreForm {
    Symbol { id: NodeId, name: String },
    Number { id: NodeId, value: i64 },
    String { id: NodeId, value: i64 },
    List { id: NodeId, elements: Vec<CoreForm> },
    DefineFunc { id: NodeId, name: String, params: Vec<String>, body: Box<CoreForm> },
}
```

**Current Lowering** (`crates/oxur-comp/src/lowering.rs:22-30`):
- Lowers Core Forms directly to syn AST
- NodeIds from Core Forms are available but not tracked
- No SourceMap integration

**SourceMap API** (from oxur-smap):
- `record_lowering(core: NodeId, rust: NodeId)` - Records Core → Rust transformation

### What's Missing 🔲

1. **SourceMap field in Lowerer** - Need to add source_map field
2. **NodeId generation for syn nodes** - Generate "Rust NodeIds" to represent syn nodes
3. **Recording mappings during lowering** - Call `record_lowering()` for each transformation
4. **Public accessor for SourceMap** - Expose source_map after lowering
5. **Tests** - Verify mappings are recorded correctly

---

## Design Decisions

### Challenge: syn Nodes Don't Have NodeIds

**Problem:** syn::Expr, syn::Item, etc. don't have NodeId fields. We can't modify syn's structures.

**Solution:** Virtual NodeIds
- Generate a NodeId to *represent* each syn node conceptually
- Store mapping: `Rust NodeId → syn::Expr` in `node_map`
- Record in SourceMap: `Core NodeId → Rust NodeId`
- This is a temporary solution until Stage 4 (Oxur AST buffer zone)

### Mapping Strategy: Core NodeId → Virtual Rust NodeId

**Pattern:**
```rust
fn lower_to_expr(&mut self, form: CoreForm) -> Result<syn::Expr> {
    let core_id = form.id(); // Extract Core NodeId
    let rust_id = oxur_smap::new_node_id(); // Generate virtual Rust NodeId

    // Record the transformation chain
    self.source_map.record_lowering(core_id, rust_id);

    // Lower to syn AST
    let expr = match form {
        CoreForm::Number { value, .. } => { /* ... */ }
        // ...
    };

    // Store the mapping (for potential future use)
    self.node_map.insert(rust_id, expr.clone());

    Ok(expr)
}
```

### Which Nodes to Track?

**Track ALL Core Forms:**
- Symbol → syn::Expr
- Number → syn::Expr
- String → syn::Expr
- List → syn::Expr (macro call, function call, etc.)
- DefineFunc → syn::Item

**Rationale:**
- Complete chain needed for accurate error reporting
- Small overhead (~24 bytes per node)
- Enables precise error location

### SourceMap Ownership

**Add to Lowerer:**
```rust
pub struct Lowerer {
    source_map: SourceMap,
    node_map: HashMap<NodeId, syn::Expr>,
}
```

**OR receive from Expander:**
```rust
pub fn lower(&mut self, forms: Vec<CoreForm>, source_map: SourceMap) -> Result<(syn::File, SourceMap)>
```

**Decision: Receive from Expander (RECOMMENDED)**
- Preserves the Surface → Core mappings from Stage 1.7
- Single SourceMap tracks entire pipeline
- Returned to caller for error translation

---

## Implementation Details

### File 1: `crates/oxur-comp/src/lowering.rs` - Update Lowerer Structure

**Location:** Lines 11-14

**Before:**
```rust
pub struct Lowerer {
    #[allow(dead_code)]
    node_map: HashMap<NodeId, syn::Expr>,
}
```

**After:**
```rust
pub struct Lowerer {
    source_map: oxur_smap::SourceMap,
    node_map: HashMap<NodeId, syn::Expr>,
}
```

### File 2: `crates/oxur-comp/src/lowering.rs` - Update Constructor

**Location:** Lines 16-19

**Before:**
```rust
pub fn new() -> Self {
    Self { node_map: HashMap::new() }
}
```

**After:**
```rust
pub fn new(source_map: oxur_smap::SourceMap) -> Self {
    Self {
        source_map,
        node_map: HashMap::new(),
    }
}
```

### File 3: `crates/oxur-comp/src/lowering.rs` - Update `lower()` Signature

**Location:** Lines 21-30

**Before:**
```rust
pub fn lower(&mut self, forms: Vec<CoreForm>) -> Result<syn::File>
```

**After:**
```rust
pub fn lower(&mut self, forms: Vec<CoreForm>) -> Result<(syn::File, oxur_smap::SourceMap)>
```

**Implementation:**
```rust
pub fn lower(&mut self, forms: Vec<CoreForm>) -> Result<(syn::File, oxur_smap::SourceMap)> {
    let mut items = Vec::new();

    for form in forms {
        items.push(self.lower_top_level(form)?);
    }

    // Freeze the source map (no more modifications)
    self.source_map.freeze();

    Ok((
        syn::File { shebang: None, attrs: vec![], items },
        self.source_map.clone(),
    ))
}
```

### File 4: `crates/oxur-comp/src/lowering.rs` - Update `lower_function()`

**Location:** Lines 43-80

**Add mapping recording:**
```rust
fn lower_function(
    &mut self,
    name: String,
    params: Vec<String>,
    body: CoreForm,
    id: NodeId, // Core NodeId
) -> Result<syn::Item> {
    use quote::format_ident;
    use syn::{ItemFn, ReturnType, Signature};

    // Generate virtual Rust NodeId for this function
    let rust_id = oxur_smap::new_node_id();
    self.source_map.record_lowering(id, rust_id);

    // Create function signature
    let fn_name = format_ident!("{}", name);
    let inputs = self.lower_params(params)?;

    let sig = Signature {
        constness: None,
        asyncness: None,
        unsafety: None,
        abi: None,
        fn_token: Default::default(),
        ident: fn_name,
        generics: Default::default(),
        paren_token: Default::default(),
        inputs,
        variadic: None,
        output: ReturnType::Default,
    };

    // Create function body (will recursively record mappings)
    let block = self.lower_block(body)?;

    Ok(syn::Item::Fn(ItemFn {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        sig,
        block: Box::new(block),
    }))
}
```

### File 5: `crates/oxur-comp/src/lowering.rs` - Update `lower_to_stmt()`

**Location:** Lines 103-120

**Add mapping recording:**
```rust
fn lower_to_stmt(&mut self, form: CoreForm) -> Result<syn::Stmt> {
    match form {
        CoreForm::List { elements, id } => {
            // Generate virtual Rust NodeId for this list
            let rust_id = oxur_smap::new_node_id();
            self.source_map.record_lowering(id, rust_id);

            // Check if this is a macro call (like println!)
            if !elements.is_empty() {
                if let CoreForm::Symbol { name, .. } = &elements[0] {
                    if name.ends_with('!') {
                        return self.lower_macro_call(name.clone(), elements[1..].to_vec());
                    }
                }
            }
            Err(crate::Error::Lowering("Unsupported list form".to_string()))
        }
        _ => Err(crate::Error::Lowering(
            "Only macro calls supported in function body for now".to_string(),
        )),
    }
}
```

### File 6: `crates/oxur-comp/src/lowering.rs` - Update `lower_macro_args()`

**Location:** Lines 141-157

**Add mapping recording for string literals:**
```rust
fn lower_macro_args(&mut self, args: Vec<CoreForm>) -> Result<proc_macro2::TokenStream> {
    use quote::quote;

    if args.is_empty() {
        return Ok(quote! {});
    }

    // For now, just handle a single string argument (for println!)
    if args.len() == 1 {
        if let CoreForm::String { value, id } = &args[0] {
            // Generate virtual Rust NodeId for this string literal
            let rust_id = oxur_smap::new_node_id();
            self.source_map.record_lowering(*id, rust_id);

            let string_lit = value.as_str();
            return Ok(quote! { #string_lit });
        }
    }

    Err(crate::Error::Lowering("Only single string arguments supported for macros".to_string()))
}
```

### File 7: `crates/oxur-comp/src/lowering.rs` - Update Tests

**Location:** Tests section (lines 166-245)

**Update all test constructors:**
```rust
#[test]
fn test_lowerer_creation() {
    let source_map = oxur_smap::SourceMap::new();
    let lowerer = Lowerer::new(source_map);
    assert_eq!(lowerer.node_map.len(), 0);
}

#[test]
fn test_lowerer_default() {
    let source_map = oxur_smap::SourceMap::new();
    let lowerer = Lowerer::new(source_map);
    assert_eq!(lowerer.node_map.len(), 0);
}

#[test]
fn test_lower_empty() {
    let source_map = oxur_smap::SourceMap::new();
    let mut lowerer = Lowerer::new(source_map);
    let result = lowerer.lower(vec![]);
    assert!(result.is_ok());
    let (file, _source_map) = result.unwrap();
    assert_eq!(file.items.len(), 0);
}

// ... update all other tests similarly
```

### File 8: `crates/oxur-comp/src/lowering.rs` - Add New Tests

**Location:** After existing tests

```rust
#[test]
fn test_source_map_function_mapping() {
    use oxur_lang::{Expander, Parser};

    let source = "(deffn main () 42)";
    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).unwrap();
    let source_map = expander.source_map().clone();

    // Lower with the source map
    let mut lowerer = Lowerer::new(source_map);
    let result = lowerer.lower(core_forms);
    assert!(result.is_ok());

    let (_file, source_map) = result.unwrap();
    let stats = source_map.stats();

    // Should have surface nodes from Stage 1.7
    assert!(stats.surface_nodes > 0);

    // Should have lowering mappings from Stage 1.8
    assert!(stats.lowerings > 0);
}

#[test]
fn test_source_map_preserved_through_lowering() {
    use oxur_lang::{Expander, Parser};

    let source = r#"(deffn main ()
  (println! "Hello, world!"))"#;

    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).unwrap();

    // Get the Core NodeId for verification
    let core_id = if let oxur_lang::CoreForm::DefineFunc { id, .. } = &core_forms[0] {
        *id
    } else {
        panic!("Expected DefineFunc");
    };

    let source_map = expander.source_map().clone();
    let surface_pos = source_map.get_surface_position(&core_id);
    assert!(surface_pos.is_some(), "Core node should have surface position");

    // Lower and verify lowering mapping added
    let mut lowerer = Lowerer::new(source_map);
    let result = lowerer.lower(core_forms);
    assert!(result.is_ok());

    let (_file, final_source_map) = result.unwrap();

    // Should still have the surface position
    let surface_pos_after = final_source_map.get_surface_position(&core_id);
    assert!(surface_pos_after.is_some(), "Surface position should be preserved");

    // Should have lowering mapping
    let stats = final_source_map.stats();
    assert!(stats.lowerings > 0, "Should have lowering mappings");
}

#[test]
fn test_source_map_frozen_after_lowering() {
    use oxur_lang::{Expander, Parser};

    let source = "(deffn main () 42)";
    let mut parser = Parser::new(source.to_string());
    let surface_forms = parser.parse().unwrap();

    let mut expander = Expander::new();
    let core_forms = expander.expand(surface_forms).unwrap();
    let source_map = expander.source_map().clone();

    let mut lowerer = Lowerer::new(source_map);
    let result = lowerer.lower(core_forms);
    assert!(result.is_ok());

    let (_file, source_map) = result.unwrap();
    assert!(source_map.is_frozen(), "SourceMap should be frozen after lowering");
}
```

### File 9: `crates/oxur-comp/Cargo.toml` - Verify Dependencies

**Check that oxur-smap is a dependency:**
```toml
[dependencies]
oxur-lang = { path = "../oxur-lang" }
oxur-smap = { path = "../oxur-smap" }
# ...
```

---

## Implementation Steps

1. **Update Lowerer structure** (10 min)
   - Add `source_map: oxur_smap::SourceMap` field
   - Update constructor to accept SourceMap parameter
   - Remove `#[allow(dead_code)]` from node_map

2. **Update `lower()` signature** (10 min)
   - Change return type to `Result<(syn::File, SourceMap)>`
   - Freeze source_map before returning
   - Clone source_map in return tuple

3. **Update `lower_function()` to record mappings** (20 min)
   - Extract Core NodeId parameter (already present)
   - Generate virtual Rust NodeId
   - Call `record_lowering(core_id, rust_id)`

4. **Update `lower_to_stmt()` to record mappings** (15 min)
   - Extract id from CoreForm::List
   - Generate virtual Rust NodeId
   - Call `record_lowering()`

5. **Update `lower_macro_args()` to record mappings** (15 min)
   - Extract id from CoreForm::String
   - Generate virtual Rust NodeId
   - Call `record_lowering()`

6. **Update all existing tests** (30 min)
   - Update constructors to pass SourceMap
   - Update assertions for tuple return type
   - Verify tests still pass

7. **Add 3 new source map tests** (45 min)
   - test_source_map_function_mapping
   - test_source_map_preserved_through_lowering
   - test_source_map_frozen_after_lowering

8. **Update compiler.rs and main.rs** (20 min)
   - Update calls to Lowerer to pass/receive SourceMap
   - Verify compilation works

9. **Run full test suite** (20 min)
   - Run `cargo test --package oxur-comp`
   - Check coverage with `cargo llvm-cov`
   - Fix any issues

10. **Code quality checks** (10 min)
    - Run `cargo clippy --package oxur-comp`
    - Run `cargo fmt --package oxur-comp`

11. **Documentation and commit** (15 min)
    - Write completion document
    - Create detailed commit message
    - Tag as Stage 1.8 complete

**Total Estimated Time:** 210 minutes (~3.5 hours)

---

## Success Criteria

✅ Lowerer accepts SourceMap from Expander
✅ All Core → syn transformations recorded in SourceMap
✅ SourceMap returned from `lower()` with both Surface and Lowering mappings
✅ SourceMap frozen after lowering completes
✅ 3 new tests verify mapping chain preservation
✅ All existing tests updated and passing
✅ Coverage ≥ 80% for lowering.rs
✅ Clippy clean, formatting correct
✅ End-to-end test: Parse → Expand → Lower preserves full mapping chain

---

## Notes

**Temporary Solution:**
- This implementation is temporary until Phase 2 integrates Oxur AST buffer zone
- Virtual NodeIds represent syn nodes conceptually
- In Phase 2, Oxur AST S-expressions will have real NodeIds

**SourceMap Chain:**
After Stage 1.8:
```
Surface Forms (Span)
    ↓ Parser records: Span info
Core Forms (NodeId)
    ↓ Expander records: Core NodeId → Surface SourcePos (Stage 1.7)
syn AST (virtual NodeId)
    ↓ Lowerer records: Core NodeId → virtual Rust NodeId (Stage 1.8)
```

**Next Stage:**
After Stage 1.8, proceed to **Stage 1.9: rustc diagnostic parser** (Parse rustc JSON output)

---

## Related Documents

- Stage breakdown: `crates/design/dev/0012-pipeline-chain-completion-stages.md`
- Main plan: `crates/design/dev/0011-pipeline-chain-completion.md`
- Previous stage: Stage 1.7 (Surface → Core mapping)
- Next stage: Stage 1.9 (rustc diagnostic parser)

---

**Plan Created:** 2026-01-12
**Estimated Completion:** Stage 1.8 implementation
