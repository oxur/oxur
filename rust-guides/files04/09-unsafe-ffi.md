# Unsafe & Safety Guidelines

Guidelines for writing sound unsafe code, FFI bindings, and understanding undefined behavior.

## Table of Contents

- [Unsafe Code](#unsafe-code)
- [FFI Patterns](#ffi-patterns)
- [Soundness](#soundness)

---

## Unsafe Code

### Unsafe Needs Reason, Should Be Avoided

**Strength**: MUST

**Summary**: Unsafe code must have a valid reason and be documented; prefer safe alternatives when possible.

**Example**:
```rust
// WRONG - using unsafe unnecessarily
fn sum_slice(slice: &[i32]) -> i32 {
    let mut total = 0;
    unsafe {
        for i in 0..slice.len() {
            total += *slice.get_unchecked(i);
        }
    }
    total
}

// CORRECT - safe version works fine
fn sum_slice(slice: &[i32]) -> i32 {
    slice.iter().sum()
}

// Unsafe is OK for performance if:
// 1. Benchmarked and shows real improvement
// 2. Properly documented
// 3. Safety invariants are clear

/// Sum slice elements.
///
/// # Safety
///
/// This is unsafe for performance. Benchmarks show 15% improvement
/// over iterator version for slices > 1000 elements.
fn sum_slice_fast(slice: &[i32]) -> i32 {
    let mut total = 0;
    unsafe {
        // SAFETY: Loop bounds ensure index is always in range
        for i in 0..slice.len() {
            total += *slice.get_unchecked(i);
        }
    }
    total
}
```

**Valid reasons for unsafe**:
1. **Novel abstractions** - New smart pointer, allocator, etc.
2. **Performance** - Documented, benchmarked improvement
3. **FFI** - Calling into C or operating system

**Invalid reasons**:
- Shortening safe Rust code
- Bypassing Send/Sync bounds
- Bypassing lifetime requirements
- Working around borrow checker

**See also**: M-UNSAFE

---

### Unsafe Implies Undefined Behavior

**Strength**: MUST

**Summary**: The `unsafe` keyword must only mark functions where misuse risks undefined behavior, not just "dangerous" operations.

**Example**:
```rust
// WRONG - dangerous but not UB
unsafe fn delete_database() {
    std::fs::remove_dir_all("/var/lib/database").unwrap();
}

// CORRECT - dangerous operation is safe
fn delete_database() {
    std::fs::remove_dir_all("/var/lib/database").unwrap();
}

// CORRECT - misuse causes UB
/// Dereferences a raw pointer.
///
/// # Safety
///
/// The caller must ensure `ptr` is non-null, properly aligned, and points
/// to a valid `T`.
pub unsafe fn deref_ptr<T>(ptr: *const T) -> &'static T {
    &*ptr
}

// CORRECT - breaking contract causes UB
/// Creates a vector from raw parts.
///
/// # Safety
///
/// - `ptr` must be allocated by the global allocator
/// - `ptr` must point to `len` consecutive initialized values
/// - The memory must not be accessed after calling this function
pub unsafe fn from_raw_parts<T>(ptr: *mut T, len: usize) -> Vec<T> {
    Vec::from_raw_parts(ptr, len, len)
}
```

**Rationale**: `unsafe` is a technical marker indicating UB risk. Don't use it to mark functions that are merely dangerous or irreversible.

**See also**: M-UNSAFE-IMPLIES-UB

---

### Document All Safety Requirements

**Strength**: MUST

**Summary**: Every unsafe function and unsafe block must have a Safety section documenting requirements.

**Example**:
```rust
/// Reads a value from a raw pointer.
///
/// # Safety
///
/// The caller must ensure:
/// - `ptr` is non-null
/// - `ptr` is properly aligned for type `T`
/// - `ptr` points to a valid, initialized `T`
/// - The pointed-to `T` is not accessed elsewhere during this call
pub unsafe fn read_ptr<T>(ptr: *const T) -> T {
    // SAFETY: Caller guarantees all requirements above
    unsafe { ptr.read() }
}

// In implementation, document each unsafe operation
fn complex_operation() {
    let data = vec![1, 2, 3, 4, 5];
    
    unsafe {
        // SAFETY: We know data.len() is 5, so index 2 is in bounds
        let third = *data.get_unchecked(2);
        
        // SAFETY: Pointer is derived from valid slice, index is in bounds
        let ptr = data.as_ptr().add(3);
        let fourth = *ptr;
    }
}
```

**Required for unsafe functions**:
- `# Safety` section in documentation
- List all invariants caller must uphold
- Be specific about requirements

**Required for unsafe blocks**:
- `// SAFETY:` comment before block
- Explain why operation is safe in this context

**See also**: M-UNSAFE, undocumented_unsafe_blocks

---

### All Code Must Be Sound

**Strength**: MUST (no exceptions)

**Summary**: No safe code may cause undefined behavior under any circumstances, including "weird" or "theoretical" scenarios.

**Example**:
```rust
// WRONG - unsound
pub fn as_u128<T>(x: &T) -> &u128 {
    unsafe { std::mem::transmute(x) }
    // UB if T is not exactly 16 bytes!
}

// WRONG - unsound Send impl
struct HasPointer {
    ptr: *const u8,
}

unsafe impl Send for HasPointer { }
// Breaking Send means data races = UB

// CORRECT - sound alternative
pub fn as_u128(x: &[u8; 16]) -> &u128 {
    // Type system enforces correct size
    unsafe {
        // SAFETY: [u8; 16] has same layout as u128
        &*(x.as_ptr() as *const u128)
    }
}

// WRONG - looks safe but is unsound
pub fn safe_deref<T>(x: Option<&T>) -> T 
where
    T: Copy
{
    match x {
        Some(r) => *r,
        None => unsafe {
            // UNSOUND: dereferencing null is UB
            *std::ptr::null()
        }
    }
}
```

**Meaning of "Safe"**:
- Function signature doesn't mark it `unsafe`
- Can be called from any safe code
- Cannot cause UB regardless of inputs

**Meaning of "Unsound"**:
- Appears safe but can cause UB
- Even in "weird" or "theoretical" scenarios
- Even if "unlikely" or "requires weird code"

**No exceptions**: Unsound code is never acceptable, even if breaking soundness seems unlikely.

**See also**: M-UNSOUND, Unsafe/Unsound/Undefined

---

## FFI Patterns

### Native Escape Hatches

**Strength**: SHOULD

**Summary**: Types wrapping native handles should provide unsafe conversion methods for interop.

**Example**:
```rust
pub struct Handle(*mut c_void);

impl Handle {
    /// Creates a handle from native API.
    pub fn new() -> Result<Self, Error> {
        // Safe creation via API calls
    }
    
    /// Creates a handle from a raw native handle.
    ///
    /// # Safety
    ///
    /// The caller must ensure:
    /// - `raw` is a valid handle obtained from the native API
    /// - The handle has not been closed or invalidated
    /// - The handle will not be used after passing to this function
    pub unsafe fn from_raw(raw: *mut c_void) -> Self {
        Self(raw)
    }
    
    /// Extracts the raw native handle.
    ///
    /// The caller becomes responsible for closing the handle.
    pub fn into_raw(self) -> *mut c_void {
        let raw = self.0;
        std::mem::forget(self);  // Don't run Drop
        raw
    }
    
    /// Borrows the raw native handle.
    pub fn as_raw(&self) -> *mut c_void {
        self.0
    }
}
```

**Rationale**: FFI scenarios may require passing handles between libraries or language boundaries. Unsafe conversions enable this while maintaining encapsulation.

**See also**: M-ESCAPE-HATCHES

---

### Isolate DLL State Between Libraries

**Strength**: MUST (for DLL FFI)

**Summary**: When loading multiple Rust DLLs, only share portable (FFI-safe, stateless) types between them.

**Example**:
```rust
// WRONG - sharing Rust types between DLLs
#[repr(C)]
pub struct Data {
    items: Vec<u8>,  // Vec layout may differ between DLLs!
}

#[no_mangle]
pub extern "C" fn process(data: Data) {
    // UB if Data came from different DLL
}

// CORRECT - only FFI-safe types
#[repr(C)]
pub struct Data {
    ptr: *const u8,
    len: usize,
}

#[no_mangle]
pub extern "C" fn process(data: Data) {
    // Safe - only uses raw pointers and primitives
    let slice = unsafe {
        std::slice::from_raw_parts(data.ptr, data.len)
    };
}
```

**Portable types** (safe to share between DLLs):
- `#[repr(C)]` types
- No interaction with `static` or `thread_local`
- No `TypeId` usage
- No `Vec`, `String`, `Box`, etc.
- No references to non-portable data

**Problems with non-portable sharing**:
- Each DLL has its own statics
- Type layouts may differ
- `TypeId` differs between DLLs
- Allocators differ

**See also**: M-ISOLATE-DLL-STATE

---

## Soundness

### Test Unsafe Code with Miri

**Strength**: SHOULD

**Summary**: Run Miri on unsafe code to detect undefined behavior.

**Example**:
```bash
# Install Miri
rustup +nightly component add miri

# Run Miri on tests
cargo +nightly miri test

# Run on specific test
cargo +nightly miri test test_unsafe_operation
```

```rust
#[test]
fn test_unsafe_slice_operation() {
    let data = vec![1, 2, 3, 4, 5];
    
    unsafe {
        // Miri will detect if this is UB
        let ptr = data.as_ptr();
        let value = *ptr.add(2);
        assert_eq!(value, 3);
    }
}

#[test]
fn test_custom_smart_pointer() {
    let ptr = CustomPtr::new(42);
    
    // Miri checks Drop, aliasing, etc.
    drop(ptr);
}
```

**What Miri detects**:
- Use-after-free
- Double-free
- Invalid pointer dereference
- Unaligned access
- Data races (with `-Zmiri-tree-borrows`)

**See also**: M-UNSAFE, Miri documentation

---

## Summary Table

| Pattern | Strength | Key Principle |
|---------|----------|---------------|
| Avoid unsafe when possible | MUST | Prefer safe alternatives |
| Unsafe = UB risk | MUST | Not just "dangerous" |
| Document safety requirements | MUST | # Safety in docs |
| All code must be sound | MUST | No exceptions |
| Provide FFI escape hatches | SHOULD | from_raw/into_raw/as_raw |
| Isolate DLL state | MUST | Only share FFI-safe types |
| Test with Miri | SHOULD | Detect UB in tests |

## Unsafe Code Checklist

When writing unsafe code:

```rust
// 1. Is unsafe necessary?
// - ✅ FFI, new abstraction, proven performance gain
// - ❌ Shortcut, avoiding borrow checker, bypassing bounds

// 2. Document requirements
/// # Safety
///
/// The caller must ensure:
/// - (specific requirement 1)
/// - (specific requirement 2)
pub unsafe fn operation() { }

// 3. Document each unsafe block
unsafe {
    // SAFETY: Explain why this specific operation is safe
}

// 4. Test with Miri
#[test]
fn test_unsafe_operation() {
    // Test all code paths
}

// 5. Handle adversarial cases
impl Drop for MyType {
    fn drop(&mut self) {
        // Must be panic-safe!
    }
}

// Assume user traits can:
// - Panic
// - Return wrong results
// - Have weird implementations
```

## Common UB Sources

### Invalid Pointer Dereference

```rust
// UB - dereferencing null
let ptr: *const i32 = std::ptr::null();
let value = unsafe { *ptr };  // UB!

// OK - check for null
if !ptr.is_null() {
    let value = unsafe { *ptr };
}
```

### Violating Aliasing Rules

```rust
// UB - multiple mutable references to same data
let mut x = 42;
let ptr = &mut x as *mut i32;
let r1 = unsafe { &mut *ptr };
let r2 = unsafe { &mut *ptr };  // UB! Aliasing violation
*r1 = 1;
*r2 = 2;

// OK - use safe split
let mut arr = [1, 2, 3];
let (left, right) = arr.split_at_mut(1);
```

### Transmuting to Wrong Size

```rust
// UB - size mismatch
let x: u32 = 42;
let y: u64 = unsafe { std::mem::transmute(x) };  // UB!

// OK - explicit conversion
let y: u64 = x as u64;
```

## Related Guidelines

- **Core Idioms**: See `01-core-idioms.md` for panic vs UB
- **Anti-Patterns**: See `11-anti-patterns.md` for unsafe misuse

## External References

- [Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Unsafe Code Guidelines](https://rust-lang.github.io/unsafe-code-guidelines/)
- [Miri](https://github.com/rust-lang/miri)
- [Rust Reference - UB](https://doc.rust-lang.org/reference/behavior-considered-undefined.html)
- Pragmatic Rust: M-UNSAFE, M-UNSOUND, M-UNSAFE-IMPLIES-UB
