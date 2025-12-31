# Unsafe Code and FFI

> Patterns for writing correct unsafe code and interfacing with C/foreign code.

---

## US-01: Minimize Unsafe Scope

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

## US-02: Document Safety Requirements

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

## US-03: Use Safe Abstractions from Crates

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

## US-04: FFI String Handling

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

## US-05: FFI Function Declarations

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

## US-06: Callback Patterns for FFI

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

## US-07: Error Handling Across FFI

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

## US-08: Panic Safety Across FFI

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

## US-09: Memory Management Across FFI

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

## US-10: Safe Wrapper Pattern

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

## US-11: Soundness Requirements

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

## Summary: Unsafe Checklist

**Before writing unsafe**:
- [ ] Can this be done safely? (Check crates like `bytemuck`, `zerocopy`)
- [ ] Is the unsafe block as small as possible?
- [ ] Are all safety requirements documented?

**Unsafe code must**:
- [ ] Have `// SAFETY:` comments explaining why it's safe
- [ ] Be wrapped in safe abstractions when possible
- [ ] Not allow safe code to cause UB

**FFI must**:
- [ ] Use `#[repr(C)]` for structs passed to/from C
- [ ] Handle null pointers explicitly
- [ ] Not let panics cross the FFI boundary
- [ ] Have clear ownership semantics (who allocates, who frees)
- [ ] Convert strings properly (`CStr`/`CString`)

---

*See also: [05-type-design.md](05-type-design.md#td-09) for PhantomData patterns.*
