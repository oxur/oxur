# Rust AI Guidelines - Complete Index

Quick reference index for all patterns and topics.

## Files by Category

### 🎯 Start Here
- **README.md** - Overview, quick start, usage guide
- **11-anti-patterns.md** - **CRITICAL** - What NOT to do (essential for AI)

### 📚 Foundation
- **01-core-idioms.md** - Essential patterns every Rust dev should know
- **02-api-design.md** - Designing clean, ergonomic public APIs
- **03-error-handling.md** - Result, custom errors, backtraces

### 🔧 Type System
- **04-ownership-borrowing.md** - Send/Sync, borrowing, mockable I/O
- **05-type-design.md** - Newtypes, traits, strong typing

### ⚡ Performance & Safety
- **08-performance.md** - Profiling, optimization, throughput
- **09-unsafe-ffi.md** - Unsafe code, FFI, soundness

### 📦 Project Organization
- **12-project-structure.md** - Crates, features, building
- **13-documentation.md** - Rustdoc, module docs

## Quick Pattern Lookup

### API Design Patterns
- Builder pattern (02)
- Cascaded initialization (02)
- Accept impl AsRef<> (02)
- Accept impl RangeBounds<> (02)
- Sans-IO pattern (02)
- Services are Clone (02)
- Essential methods inherent (02)

### Error Handling Patterns
- Canonical error structs (03)
- Backtrace capture (03)
- Display/Debug implementation (03)
- Application errors with anyhow (03)

### Type Design Patterns
- Newtype pattern (05)
- Strong types over primitives (04, 05)
- Debug for public types (05)
- Display for user-facing types (05)

### Ownership Patterns
- Send/Sync requirements (04)
- Mockable I/O (04)
- Proper type families (04)
- Don't leak external types (04)

### Performance Patterns
- Batch processing (08)
- Yield points in async (08)
- Profile hot paths (08)
- Use mimalloc for apps (08)

### Safety Patterns
- Document unsafe code (09)
- Test with Miri (09)
- FFI escape hatches (09)
- Sound abstractions (09)

### Project Patterns
- Split into small crates (12)
- Features are additive (12)
- Libraries work OOBE (12)
- Static verification (12)

### Documentation Patterns
- First sentence ~15 words (13)
- Canonical sections (13)
- Module documentation (13)
- doc(inline) for re-exports (13)

## Common Anti-Patterns (Critical)

### Type System
- ❌ String for everything (11)
- ❌ Smart pointers in APIs (11)
- ❌ Deep generic nesting (11)

### Error Handling
- ❌ Panic for errors (11)
- ❌ Result for bugs (11)
- ❌ Public ErrorKind (11)

### Ownership
- ❌ Clone instead of borrow (11)
- ❌ unsafe to bypass borrow checker (11)
- ❌ Rc in async code (11)

### API Design
- ❌ Associated fn for everything (11)
- ❌ Builder for simple types (11)
- ❌ Glob re-exports (11)

### Performance
- ❌ Premature optimization (11)
- ❌ Allocate in hot loops (11)

### Safety
- ❌ Unsound code (11)
- ❌ unsafe without docs (11)

## Strength Indicators

| Indicator | Meaning | When to Deviate |
|-----------|---------|-----------------|
| **MUST** | Always required | Only with strong architectural justification |
| **SHOULD** | Strong recommendation | When you have documented good reason |
| **CONSIDER** | Suggested approach | Situational; evaluate trade-offs |
| **AVOID** | Anti-pattern | Don't do unless no alternative exists |

## Quick Access by Need

### "I'm writing a library"
1. Read: 01, 02, 03, 04, 05
2. Check: 11 (anti-patterns)
3. Setup: 12 (project structure)
4. Document: 13

### "I'm writing an application"
1. Read: 01, 03, 08
2. Check: 11 (anti-patterns)
3. Consider: Performance (08), error handling (03)

### "I'm doing FFI"
1. Read: 09 (unsafe-ffi)
2. Check: 11 (anti-patterns)
3. Review: 04 (ownership)

### "I'm optimizing performance"
1. Read: 08 (performance)
2. Check: 11 (anti-patterns)
3. Review: 02 (API design for batching)

### "I'm reviewing code"
1. Start with: 11 (anti-patterns)
2. Check patterns in relevant topic files
3. Verify strength indicators (MUST vs SHOULD)

## File Statistics

| File | Patterns | Examples | Pages |
|------|----------|----------|-------|
| 01-core-idioms.md | 7 | 10+ | ~10 |
| 02-api-design.md | 11 | 15+ | ~18 |
| 03-error-handling.md | 9 | 12+ | ~18 |
| 04-ownership-borrowing.md | 7 | 10+ | ~14 |
| 05-type-design.md | 6 | 8+ | ~13 |
| 08-performance.md | 6 | 10+ | ~11 |
| 09-unsafe-ffi.md | 6 | 8+ | ~12 |
| 11-anti-patterns.md | 15 | 20+ | ~17 |
| 12-project-structure.md | 7 | 8+ | ~11 |
| 13-documentation.md | 6 | 8+ | ~12 |

**Total**: 80+ patterns, 100+ examples

## External References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Pragmatic Rust Guidelines](https://microsoft.github.io/rust-guidelines/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- [Rustonomicon](https://doc.rust-lang.org/nomicon/)

## Version

Last Updated: 2025-01-05
Source: Pragmatic Rust Guidelines (Microsoft)
Coverage: ~85% of source material
