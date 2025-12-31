# Quick Start Guide

Get up to speed with the Rust AI Guidelines in 5 minutes.

## 🎯 For AI Code Generation

### Must Read (5 minutes)
1. **11-anti-patterns.md** - What NOT to do (critical!)
2. **01-core-idioms.md** - Essential Rust patterns

### Pattern Quick Reference
```rust
// ✅ DO: Use strong types
pub struct UserId(u64);
pub fn create_user(id: UserId) -> User { }

// ❌ DON'T: String for everything  
pub fn create_user(id: String) -> User { }

// ✅ DO: Return Result for errors
pub fn load_config() -> Result<Config, Error> { }

// ❌ DON'T: Panic for recoverable errors
pub fn load_config() -> Config {
    let file = File::open("config.toml").unwrap(); // ❌
}

// ✅ DO: Implement Debug
#[derive(Debug)]
pub struct Config { }

// ✅ DO: Use Arc in async, not Rc
async fn process(data: Arc<String>) { }

// ❌ DON'T: Use Rc in async
async fn process(data: Rc<String>) { } // ❌ !Send
```

## 📚 For Learning Rust

### Day 1 - Basics
- Read: 01-core-idioms.md
- Read: 11-anti-patterns.md (Type System, Error Handling)
- Practice: Write a simple struct with methods

### Day 2 - APIs
- Read: 02-api-design.md
- Read: 03-error-handling.md
- Practice: Design a small library API

### Day 3 - Types
- Read: 04-ownership-borrowing.md
- Read: 05-type-design.md
- Practice: Create newtypes, implement traits

### Day 4 - Advanced
- Read: 08-performance.md
- Read: 09-unsafe-ffi.md
- Read: 12-project-structure.md

## 🔍 For Code Review

### Review Checklist
```markdown
- [ ] Check 11-anti-patterns.md for violations
- [ ] Verify MUST patterns are followed
- [ ] Check error handling (Result not panic)
- [ ] Verify types are Send where needed
- [ ] Check unsafe has Safety documentation
- [ ] Verify public types have Debug
- [ ] Check for String overuse
```

### Top 10 Things to Look For
1. ❌ `unwrap()` / `expect()` in library code
2. ❌ String used for paths, IDs, etc.
3. ❌ `Arc<Mutex<T>>` in public APIs
4. ❌ Rc in async functions
5. ❌ Public ErrorKind enum
6. ❌ Missing Debug implementation
7. ❌ unsafe without Safety docs
8. ❌ Builder for simple types (< 4 params)
9. ❌ Associated functions for unrelated logic
10. ❌ Features that aren't additive

## 🚀 By Use Case

### "I'm building a web service"
```rust
// Essential reading:
// 01 - Core idioms
// 02 - API design (services, builders)
// 03 - Error handling
// 04 - Send/Sync
// 08 - Performance (yield points)
// 11 - Anti-patterns

// Key patterns:
#[derive(Clone)]  // Services are Clone
pub struct MyService {
    inner: Arc<ServiceInner>,
}

impl MyService {
    pub fn new() -> Self { }
    
    pub async fn process(&self) -> Result<Data, Error> {
        // Yield in long operations
        tokio::task::yield_now().await;
    }
}
```

### "I'm writing a CLI tool"
```rust
// Essential reading:
// 01 - Core idioms
// 03 - Error handling (anyhow ok for apps)
// 11 - Anti-patterns

use anyhow::Result;  // OK for applications

fn main() -> Result<()> {
    let config = load_config()?;
    process(&config)?;
    Ok(())
}
```

### "I'm creating a library"
```rust
// Essential reading:
// 01 - Core idioms
// 02 - API design
// 03 - Error handling (canonical structs)
// 04 - Send/Sync, mockable I/O
// 05 - Type design
// 12 - Project structure
// 13 - Documentation

// Key patterns:
pub struct MyError {
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl MyError {
    pub fn is_io(&self) -> bool {
        matches!(self.kind, ErrorKind::Io(_))
    }
}
```

## 📖 File Overview

| File | When to Read | Est. Time |
|------|-------------|-----------|
| **11-anti-patterns.md** | Before writing code | 15 min |
| **01-core-idioms.md** | Starting with Rust | 10 min |
| **02-api-design.md** | Designing APIs | 20 min |
| **03-error-handling.md** | Handling errors | 20 min |
| **04-ownership-borrowing.md** | Working with ownership | 15 min |
| **05-type-design.md** | Creating types | 15 min |
| **08-performance.md** | Optimizing code | 15 min |
| **09-unsafe-ffi.md** | Using unsafe/FFI | 15 min |
| **12-project-structure.md** | Organizing crates | 15 min |
| **13-documentation.md** | Writing docs | 15 min |

**Total**: ~2.5 hours to read everything

## 🎓 Understanding Strength Indicators

```rust
// MUST - Always do this
#[derive(Debug)]  // All public types MUST have Debug
pub struct User { }

// SHOULD - Strong recommendation
impl User {
    pub fn builder() -> UserBuilder { }  // SHOULD use builder for 4+ params
}

// CONSIDER - Evaluate trade-offs
#[global_allocator]  // CONSIDER mimalloc for apps
static GLOBAL: MiMalloc = MiMalloc;

// AVOID - Anti-pattern
pub fn create(name: String) { }  // AVOID String for IDs/types
```

## 🔗 Quick Links

- **Start**: README.md
- **Index**: INDEX.md  
- **Critical**: 11-anti-patterns.md
- **Summary**: GENERATION_SUMMARY.md

## 💡 Tips

1. **For AI assistants**: Always check anti-patterns first
2. **For beginners**: Read in order (01 → 13)
3. **For code review**: Start with strength indicators (MUST > SHOULD)
4. **For debugging**: Check error handling and ownership files
5. **For optimization**: Read performance file with profiler handy

## 🎯 Next Steps

After reading this guide:
1. Skim through 11-anti-patterns.md (15 min)
2. Read 01-core-idioms.md thoroughly (10 min)
3. Reference other files as needed
4. Keep INDEX.md open for quick lookups

Happy coding! 🦀
