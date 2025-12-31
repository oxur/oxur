# Unsafe & Safety Guidelines

Guidelines for writing sound unsafe code, FFI bindings, and understanding undefined behavior.


## US-01: All Code Must Be Sound

**Strength**: MUST

**Summary**: No safe code may cause undefined behavior under any circumstances, including "weird" or "theoretical" scenarios.

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

**See also**: M-UNSOUND, Unsafe/Unsound/Undefined

---

## US-02: Callback Patterns for FFI

**Strength**: SHOULD

**Summary**: Handle callbacks between Rust and C safely.

```rust
use std::os::raw::c_void;

// C callback type
type CCallback = extern "C" fn(data: c_int, user_data: *mut c_void);

extern "C" {
    fn register_callback(cb: CCallback, user_data: *mut c_void);
}

// ✅ CORRECT: Rust function as C callback
extern "C" fn my_callback(data: c_int, user_data: *mut c_void) {
    // SAFETY: We registered this with a Box<Context> pointer
    let context = unsafe { &mut *(user_data as *mut Context) };
    context.handle(data);
}

struct Context {
    counter: i32,
}

impl Context {
    fn handle(&mut self, data: c_int) {
        self.counter += data;
    }
}

fn setup_callback() {
    let context = Box::new(Context { counter: 0 });
    let context_ptr = Box::into_raw(context);
    
    unsafe {
        register_callback(my_callback, context_ptr as *mut c_void);
    }
    
    // Remember to free context_ptr when done!
}

// ✅ Cleanup
fn cleanup(context_ptr: *mut Context) {
    unsafe {
        let _ = Box::from_raw(context_ptr);  // Drops the Context
    }
}
```

---

## US-03: Document All Safety Requirements

**Strength**: MUST

**Summary**: Every unsafe function and unsafe block must have a Safety section documenting requirements.

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

**See also**: M-UNSAFE, undocumented_unsafe_blocks

---

## US-04: Document Safety Requirements

**Strength**: MUST

**Summary**: Every `unsafe` block and function must have a `// SAFETY:` comment.

```rust
// ✅ GOOD: Documented unsafe block
let value = unsafe {
    // SAFETY: We checked that index < len on line 42,
    // so this access is within bounds.
    *slice.get_unchecked(index)
};

// ✅ GOOD: Documented unsafe function
/// Reads a value from the pointer.
///
/// # Safety
///
/// - `ptr` must be valid for reads of `T`
/// - `ptr` must be properly aligned
/// - `ptr` must point to an initialized `T`
/// - The memory must not be mutated while this reference exists
pub unsafe fn read_ptr<T>(ptr: *const T) -> T {
    // SAFETY: Caller is responsible for upholding the contract
    // documented above.
    ptr.read()
}

// ✅ GOOD: unsafe trait with documented requirements
/// A type that can be safely zeroed.
///
/// # Safety
///
/// Implementors must ensure that a value consisting of all zero bytes
/// is a valid instance of the type.
pub unsafe trait Zeroable {
    fn zeroed() -> Self;
}

// Safe because all-zeros is a valid u32
unsafe impl Zeroable for u32 {
    fn zeroed() -> Self { 0 }
}
```

---

## US-05: Error Handling Across FFI

**Strength**: SHOULD

**Summary**: Convert between Rust errors and C-style error codes.

```rust
use std::os::raw::c_int;

// C-style error codes
const SUCCESS: c_int = 0;
const ERR_INVALID_ARG: c_int = -1;
const ERR_OUT_OF_MEMORY: c_int = -2;
const ERR_IO: c_int = -3;

// ✅ Rust error to C error code
#[derive(Debug)]
enum MyError {
    InvalidArg,
    OutOfMemory,
    Io(std::io::Error),
}

impl MyError {
    fn to_c_error(&self) -> c_int {
        match self {
            MyError::InvalidArg => ERR_INVALID_ARG,
            MyError::OutOfMemory => ERR_OUT_OF_MEMORY,
            MyError::Io(_) => ERR_IO,
        }
    }
}

// ✅ Exported function with C error handling
#[no_mangle]
pub extern "C" fn my_function(arg: c_int) -> c_int {
    match do_work(arg) {
        Ok(()) => SUCCESS,
        Err(e) => e.to_c_error(),
    }
}

fn do_work(arg: c_int) -> Result<(), MyError> {
    if arg < 0 {
        return Err(MyError::InvalidArg);
    }
    Ok(())
}

// ✅ Thread-local last error (like errno)
use std::cell::RefCell;

thread_local! {
    static LAST_ERROR: RefCell<Option<MyError>> = RefCell::new(None);
}

#[no_mangle]
pub extern "C" fn get_last_error() -> c_int {
    LAST_ERROR.with(|e| {
        e.borrow().as_ref().map_or(SUCCESS, |e| e.to_c_error())
    })
}

fn set_last_error(error: MyError) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(error);
    });
}
```

---

## US-06: FFI Function Declarations

**Strength**: MUST

**Summary**: Declare extern functions with correct types and ABI.

```rust
use std::os::raw::{c_int, c_char, c_void};

// ✅ CORRECT: Explicit ABI and correct types
#[link(name = "mylib")]
extern "C" {
    fn my_c_function(arg: c_int) -> c_int;
    fn process_buffer(ptr: *const c_void, len: usize);
    fn get_string() -> *const c_char;
    
    // Variadic function
    fn printf(format: *const c_char, ...) -> c_int;
}

// ✅ CORRECT: Opaque types for C structs
#[repr(C)]
pub struct OpaqueHandle {
    _private: [u8; 0],  // Zero-sized, can't be constructed in Rust
}

extern "C" {
    fn create_handle() -> *mut OpaqueHandle;
    fn destroy_handle(handle: *mut OpaqueHandle);
}

// ✅ CORRECT: Struct with C layout
#[repr(C)]
pub struct Point {
    x: f64,
    y: f64,
}

extern "C" {
    fn process_point(point: *const Point);
}

// ❌ WRONG: Missing repr(C)
struct BadPoint {  // Rust may reorder fields!
    x: f64,
    y: f64,
}
```

---

## US-07: FFI String Handling

**Strength**: MUST

**Summary**: Convert between Rust strings and C strings correctly.

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// ✅ Passing Rust string TO C
fn call_c_function(s: &str) {
    // CString adds null terminator, ensures no interior nulls
    let c_string = CString::new(s).expect("string contains null byte");
    unsafe {
        c_function(c_string.as_ptr());
    }
    // c_string lives until end of scope - pointer remains valid
}

// ❌ BAD: Dangling pointer!
fn bad_call(s: &str) {
    unsafe {
        // CString is dropped immediately, pointer is dangling!
        c_function(CString::new(s).unwrap().as_ptr());
    }
}

// ✅ Receiving C string IN Rust
unsafe fn handle_c_string(ptr: *const c_char) -> String {
    // SAFETY: Caller guarantees ptr is valid, null-terminated C string
    let c_str = CStr::from_ptr(ptr);
    
    // Lossy: replaces invalid UTF-8 with replacement character
    c_str.to_string_lossy().into_owned()
    
    // Or strict: fails on invalid UTF-8
    // c_str.to_str().unwrap().to_string()
}

// ✅ Passing string to C that will store it
extern "C" fn c_stores_string(ptr: *const c_char);

fn pass_string_to_c(s: &str) {
    let c_string = CString::new(s).unwrap();
    unsafe {
        // Transfer ownership - C is now responsible for freeing
        c_stores_string(c_string.into_raw());
    }
    // Don't use c_string after into_raw()!
}

// ✅ Receiving back a string C allocated
unsafe fn take_back_string(ptr: *mut c_char) -> CString {
    // Retake ownership of string we gave to C
    CString::from_raw(ptr)
}
```

---

## US-08: Invalid Pointer Dereference

**Strength**: SHOULD

**Summary**: 

```rust
// UB - dereferencing null
let ptr: *const i32 = std::ptr::null();
let value = unsafe { *ptr };  // UB!

// OK - check for null
if !ptr.is_null() {
    let value = unsafe { *ptr };
}
```

---

## US-09: Isolate DLL State Between Libraries

**Strength**: MUST

**Summary**: When loading multiple Rust DLLs, only share portable (FFI-safe, stateless) types between them.

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

**See also**: M-ISOLATE-DLL-STATE

---

## US-10: Memory Management Across FFI

**Strength**: MUST

**Summary**: Be explicit about ownership when crossing FFI boundaries.

```rust
use std::os::raw::c_void;

// ✅ Pattern 1: Rust allocates, Rust frees
#[no_mangle]
pub extern "C" fn create_data() -> *mut Data {
    Box::into_raw(Box::new(Data::new()))
}

#[no_mangle]
pub extern "C" fn destroy_data(ptr: *mut Data) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

// ✅ Pattern 2: C allocates, C frees (Rust only borrows)
#[no_mangle]
pub extern "C" fn process_data(ptr: *const Data) {
    if ptr.is_null() {
        return;
    }
    let data = unsafe { &*ptr };
    // Use data, but don't free it
}

// ✅ Pattern 3: Explicit allocator parameter
extern "C" {
    fn c_malloc(size: usize) -> *mut c_void;
    fn c_free(ptr: *mut c_void);
}

#[no_mangle]
pub extern "C" fn create_with_c_allocator() -> *mut Data {
    unsafe {
        let ptr = c_malloc(std::mem::size_of::<Data>()) as *mut Data;
        if !ptr.is_null() {
            ptr.write(Data::new());
        }
        ptr
    }
}
```

---

## US-11: Minimize Unsafe Scope

**Strength**: MUST

**Summary**: Keep `unsafe` blocks as small as possible; wrap in safe abstractions.

```rust
// ❌ BAD: Large unsafe block
unsafe fn process_buffer(ptr: *const u8, len: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let byte = *ptr.add(i);  // Unsafe
        result.push(byte);       // Safe, but inside unsafe block
        validate(byte);          // Safe, but inside unsafe block
    }
    result
}

// ✅ GOOD: Minimal unsafe, safe wrapper
fn process_buffer(ptr: *const u8, len: usize) -> Vec<u8> {
    // Convert to safe slice first
    let slice = unsafe {
        // SAFETY: Caller guarantees ptr is valid for len bytes,
        // properly aligned, and the memory won't be mutated.
        std::slice::from_raw_parts(ptr, len)
    };
    
    // Rest is safe!
    let mut result = Vec::with_capacity(len);
    for &byte in slice {
        result.push(byte);
        validate(byte);
    }
    result
}

// ✅ BEST: Safe public API wrapping unsafe internals
pub struct Buffer {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

impl Buffer {
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            // SAFETY: Buffer maintains invariant that ptr is valid
            // for len bytes and properly aligned.
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }
    
    // All unsafe is encapsulated; public API is safe
}
```

---

## US-12: Native Escape Hatches

**Strength**: SHOULD

**Summary**: Types wrapping native handles should provide unsafe conversion methods for interop.

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

## US-13: Panic Safety Across FFI

**Strength**: MUST

**Summary**: Never let panics unwind across FFI boundaries.

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};

// ❌ BAD: Panic can cross FFI boundary (undefined behavior!)
#[no_mangle]
pub extern "C" fn dangerous_function() {
    panic!("This is UB!");  // Panic unwinds into C code
}

// ✅ GOOD: Catch panics at FFI boundary
#[no_mangle]
pub extern "C" fn safe_function() -> c_int {
    let result = catch_unwind(AssertUnwindSafe(|| {
        do_work()
    }));
    
    match result {
        Ok(Ok(())) => SUCCESS,
        Ok(Err(e)) => e.to_c_error(),
        Err(_panic) => {
            // Panic occurred - return error code
            // Optionally log the panic
            ERR_PANIC
        }
    }
}

// ✅ Alternative: abort on panic (set in Cargo.toml)
// [profile.release]
// panic = "abort"
```

---

## US-14: Safe Wrapper Pattern

**Strength**: SHOULD

**Summary**: Create a safe Rust wrapper around unsafe FFI.

```rust
// Raw FFI bindings (usually in a separate -sys crate)
mod ffi {
    use std::os::raw::{c_int, c_void};
    
    extern "C" {
        pub fn lib_create() -> *mut c_void;
        pub fn lib_destroy(handle: *mut c_void);
        pub fn lib_process(handle: *mut c_void, value: c_int) -> c_int;
    }
}

// ✅ Safe wrapper
pub struct Handle {
    ptr: *mut std::ffi::c_void,
}

impl Handle {
    pub fn new() -> Option<Self> {
        let ptr = unsafe { ffi::lib_create() };
        if ptr.is_null() {
            None
        } else {
            Some(Handle { ptr })
        }
    }
    
    pub fn process(&mut self, value: i32) -> Result<i32, Error> {
        let result = unsafe { ffi::lib_process(self.ptr, value) };
        if result < 0 {
            Err(Error::from_code(result))
        } else {
            Ok(result)
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            ffi::lib_destroy(self.ptr);
        }
    }
}

// Handle is now safe to use:
// - Can't be null (checked in constructor)
// - Automatically cleaned up (Drop)
// - Errors are propagated properly
```

---

## US-15: Soundness Requirements

**Strength**: MUST

**Summary**: Understand what makes unsafe code sound.

```rust
// SOUNDNESS: Safe code cannot cause undefined behavior,
// no matter how it's used.

// ❌ UNSOUND: Safe function can cause UB
pub fn unsound(slice: &[u8], index: usize) -> u8 {
    unsafe {
        *slice.get_unchecked(index)  // UB if index >= slice.len()
    }
}

// ✅ SOUND: Unsafe is contained, invariants checked
pub fn sound(slice: &[u8], index: usize) -> Option<u8> {
    if index < slice.len() {
        Some(unsafe { *slice.get_unchecked(index) })
    } else {
        None
    }
}

// ✅ SOUND: Unsafe internals, safe interface
pub struct SafeBuffer {
    ptr: *mut u8,
    len: usize,
}

impl SafeBuffer {
    // Constructor ensures invariants
    pub fn new(len: usize) -> Self {
        let ptr = unsafe { alloc(len) };
        Self { ptr, len }
    }
    
    // Safe method maintains invariants
    pub fn get(&self, index: usize) -> Option<u8> {
        if index < self.len {
            Some(unsafe { *self.ptr.add(index) })
        } else {
            None
        }
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.len) }
    }
}
```

---

## US-16: Test Unsafe Code with Miri

**Strength**: SHOULD

**Summary**: Run Miri on unsafe code to detect undefined behavior.

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

**See also**: M-UNSAFE, Miri documentation

---

## US-17: Transmuting to Wrong Size

**Strength**: SHOULD

**Summary**: 

```rust
// UB - size mismatch
let x: u32 = 42;
let y: u64 = unsafe { std::mem::transmute(x) };  // UB!

// OK - explicit conversion
let y: u64 = x as u64;
```

---

## US-18: Unsafe Code Checklist

**Strength**: SHOULD

**Summary**: 

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

---

## US-19: Unsafe Implies Undefined Behavior

**Strength**: MUST

**Summary**: The `unsafe` keyword must only mark functions where misuse risks undefined behavior, not just "dangerous" operations.

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

## US-20: Unsafe Needs Reason, Should Be Avoided

**Strength**: MUST

**Summary**: Unsafe code must have a valid reason and be documented; prefer safe alternatives when possible.

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

**See also**: M-UNSAFE

---

## US-21: Use Safe Abstractions from Crates

**Strength**: SHOULD

**Summary**: Prefer battle-tested safe abstractions over writing unsafe code.

```rust
// ❌ RISKY: Rolling your own unsafe
fn transmute_slice<T, U>(slice: &[T]) -> &[U] {
    unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const U,
            slice.len() * std::mem::size_of::<T>() / std::mem::size_of::<U>()
        )
    }
}

// ✅ SAFER: Use bytemuck crate
use bytemuck::{cast_slice, Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

let vertices: &[Vertex] = &[/* ... */];
let bytes: &[u8] = cast_slice(vertices);  // Safe, checked at compile time

// Other safe abstraction crates:
// - zerocopy: Safe transmutation
// - memoffset: Safe offset_of!
// - pin-utils: Safe pinning helpers
```

---

## US-22: Violating Aliasing Rules

**Strength**: SHOULD

**Summary**: 

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