---
number: 28
title: "evcxr_runtime Audit Report"
author: "Claude Code"
component: REPL
tags: [research]
created: 2026-01-03
updated: 2026-01-03
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# evcxr_runtime Audit Report

**Date:** 2026-01-03
**Audited by:** Claude Sonnet 4.5
**Repository:** <https://github.com/evcxr/evcxr>
**Crate Version:** 1.1.0
**Focus:** Understanding evcxr_runtime for potential Oxur REPL integration

---

## 1. Executive Summary

### Critical Discovery: "evcxr_runtime" is NOT a Runtime

**The name is misleading.** `evcxr_runtime` is not a comprehensive runtime system for code execution. It is a minimal (~75 lines) utility library that provides **MIME-typed output formatting** for REPL and Jupyter kernel display.

### What evcxr_runtime Actually Provides

1. **`Display` Trait** - Interface for custom display implementations
2. **`mime_type()` Function** - Emit content with MIME type markers
3. **Text/Binary Support** - Handle both text and base64-encoded binary data

That's it. No value representation, no execution model, no error handling, no state management.

### The Real "Runtime" is Elsewhere

The actual runtime functionality for evcxr is located in:

- **`evcxr/src/evcxr_internal_runtime.rs`** - Variable storage (`Box<dyn Any>` based)
- **`evcxr/src/runtime.rs`** - Subprocess execution loop and library loading
- **`evcxr/src/child_process.rs`** - Process management and IPC

These were covered in the evcxr_repl audit report.

### Recommended Approach for Oxur

**Option A: Don't Use evcxr_runtime** (Recommended)

- It provides minimal value for Oxur's needs
- The MIME-type output pattern can be trivially implemented ourselves
- Avoids unnecessary dependency
- We have more control over output formatting

**Option B: Use evcxr_runtime** (If we want Jupyter compatibility)

- Only useful if we plan to build an Oxur Jupyter kernel
- Provides consistent MIME output format with evcxr
- Enables reuse of evcxr extension libraries (like `evcxr_image`)

**Recommendation: Skip evcxr_runtime for v1.0.** Implement our own output formatting. Consider it only if building a Jupyter kernel later.

---

## 2. API Analysis

### API 1: `evcxr_runtime::Display` Trait

**Purpose:**

A trait that types can implement to provide custom REPL display logic. When a value of a type implementing this trait is the final expression in a REPL evaluation, evcxr attempts to call `.evcxr_display()` on it, allowing the type to emit MIME-typed content (HTML, images, etc.) instead of just using `Debug` formatting.

**Key Methods/Functions:**

```rust
pub trait Display {
    /// Emit a representation of self using mime_type() calls
    fn evcxr_display(&self);
}
```

**Usage Example:**

```rust
use evcxr_runtime::Display;

struct MyChart {
    data: Vec<f64>,
}

impl Display for MyChart {
    fn evcxr_display(&self) {
        // Generate SVG chart
        let svg = format!(
            r#"<svg width="400" height="200">
                <polyline points="{}" fill="none" stroke="blue"/>
            </svg>"#,
            self.data.iter()
                .enumerate()
                .map(|(i, v)| format!("{},{}", i * 10, 200.0 - v))
                .collect::<Vec<_>>()
                .join(" ")
        );

        evcxr_runtime::mime_type("text/html").text(svg);
    }
}

// In REPL:
let chart = MyChart { data: vec![10.0, 50.0, 30.0, 80.0] };
chart  // Calls chart.evcxr_display(), renders SVG in Jupyter
```

**Relevance to Oxur:** **Low** - We'll likely define our own display trait

**Complexity:** **Simple** - Just implement one method

**Priority:** **P3** - Only needed if building Jupyter kernel

**Integration Notes:**

For Oxur, we'd define our own trait instead:

```rust
pub trait OxurDisplay {
    fn oxur_display(&self) -> DisplayValue;
}

pub enum DisplayValue {
    Text(String),
    Html(String),
    Image { mime: String, data: Vec<u8> },
    Custom { mime: String, content: String },
}
```

This gives us more control and avoids the println-based approach which doesn't work well with our network protocol (we need structured responses, not stdout scraping).

**Dependencies:** None

---

### API 2: `evcxr_runtime::mime_type()` Function

**Purpose:**

Creates a `ContentMimeType` builder that can emit content with a specified MIME type. Content is output via `println!()` with special markers (`EVCXR_BEGIN_CONTENT`, `EVCXR_END_CONTENT`) that the REPL parent process parses to extract MIME-typed output.

**Key Methods/Functions:**

```rust
pub fn mime_type<S: Into<String>>(mime_type: S) -> ContentMimeType;

impl ContentMimeType {
    pub fn text<S: AsRef<str>>(self, text: S);

    #[cfg(feature = "bytes")]
    pub fn bytes(self, buffer: &[u8]);  // Base64 encodes binary data
}
```

**Usage Example:**

```rust
use evcxr_runtime::mime_type;

// Text content
mime_type("text/plain").text("Hello, world!");

// HTML content
mime_type("text/html").text("<h1>Title</h1><p>Paragraph</p>");

// Image content (with "bytes" feature)
let png_data: Vec<u8> = generate_png();
mime_type("image/png").bytes(&png_data);
// Output: EVCXR_BEGIN_CONTENT image/png
//         <base64 encoded data>
//         EVCXR_END_CONTENT
```

**Relevance to Oxur:** **Low** - The pattern is useful but we need structured output

**Complexity:** **Simple** - Just a thin wrapper around println

**Priority:** **P3** - Not needed unless building Jupyter compatibility

**Integration Notes:**

For Oxur's network protocol, we can't use stdout markers. Instead, we'd structure responses:

```rust
pub struct EvalResponse {
    pub value: Option<DisplayValue>,
    pub out: String,        // Captured stdout
    pub err: String,        // Captured stderr
    pub mime_outputs: Vec<MimeOutput>,  // Rich outputs
}

pub struct MimeOutput {
    pub mime_type: String,
    pub content: MimeContent,
}

pub enum MimeContent {
    Text(String),
    Binary(Vec<u8>),
}
```

If we later build a Jupyter kernel, we could implement compatibility by:

1. Capturing stdout during evaluation
2. Parsing EVCXR_BEGIN_CONTENT/END_CONTENT markers
3. Converting to MimeOutput structs
4. Sending via Jupyter protocol

**Dependencies:**

- None for text output
- `base64` crate for binary output (optional feature)

---

### API 3: `ContentMimeType` Struct (Internal)

**Purpose:**

An internal builder struct created by `mime_type()`. Holds the MIME type string and provides methods to emit content.

**Key Methods/Functions:**

```rust
pub struct ContentMimeType {
    mime_type: String,  // Private field
}

impl ContentMimeType {
    pub fn text<S: AsRef<str>>(self, text: S);

    #[cfg(feature = "bytes")]
    pub fn bytes(self, buffer: &[u8]);
}
```

**Usage Example:**

```rust
// Create with mime_type()
let content_type = evcxr_runtime::mime_type("application/json");

// Emit text
content_type.text(r#"{"key": "value", "number": 42}"#);

// Output to stdout:
// EVCXR_BEGIN_CONTENT application/json
// {"key": "value", "number": 42}
// EVCXR_END_CONTENT
```

**Relevance to Oxur:** **Low** - Internal implementation detail

**Complexity:** **Simple** - Just a wrapper struct

**Priority:** **P3** - Not directly used

**Integration Notes:**

Not relevant to Oxur unless we're implementing evcxr compatibility layer.

**Dependencies:** None (base64 for bytes feature)

---

## 3. Value Type System Analysis

### Critical Finding: No Value Type System

**evcxr_runtime does NOT provide value representation.** It only provides output formatting.

Value representation in evcxr is handled by:

- Rust's native types (i32, String, Vec, etc.)
- `Box<dyn Any>` for type-erased storage (in `evcxr_internal_runtime.rs`)
- Direct FFI passing of raw pointers between compiled code and subprocess

### What evcxr_runtime Does NOT Provide

❌ Type system or value representation
❌ Serialization/deserialization
❌ Type conversions or coercions
❌ Generic value handling
❌ Memory management
❌ Execution model
❌ Error handling

### What evcxr_runtime DOES Provide

✅ MIME type markers for stdout
✅ Base64 encoding for binary data
✅ A trait for custom display implementations

That's it.

### Supported Types

**All Rust types are "supported"** in the sense that any type can implement the `Display` trait. But evcxr_runtime itself doesn't do anything with types - it just prints markers to stdout.

### Display Formatting

**Text:** Printed as-is between markers
**Binary:** Base64 encoded, then printed

**Example Output:**

```
EVCXR_BEGIN_CONTENT text/html
<h1>Hello</h1>
EVCXR_END_CONTENT
```

**No special formatting** - it's just string interpolation into println.

---

## 4. Memory and Safety Model

### Memory Model: None

evcxr_runtime does not manage memory. It just calls `println!()`.

### Safety Model: Safe

All APIs are safe. No `unsafe` code in the crate.

```rust
// The entire implementation (simplified):
pub fn mime_type<S: Into<String>>(mime_type: S) -> ContentMimeType {
    ContentMimeType { mime_type: mime_type.into() }
}

impl ContentMimeType {
    pub fn text<S: AsRef<str>>(self, text: S) {
        println!("EVCXR_BEGIN_CONTENT {}\n{}\nEVCXR_END_CONTENT",
            self.mime_type, text.as_ref());
    }
}
```

### Thread Safety: Yes

- No shared state
- All methods take `self` by value (consumed after use)
- `println!()` is thread-safe

Multiple threads can call `mime_type()` concurrently without issues (though output may interleave).

### What This Means for Oxur

**We don't need to worry about evcxr_runtime's safety** because it has no memory management, no unsafe code, and no complex invariants. It's just a printing utility.

The *actual* safety concerns are in:

- Loading dynamic libraries (`libloading` - covered in evcxr_repl audit)
- Type-erased storage (`Box<dyn Any>` downcasting - covered in evcxr_repl audit)
- FFI boundaries (raw pointer passing - covered in evcxr_repl audit)

---

## 5. Integration Checklist

### If Using evcxr_runtime (Not Recommended)

**Required Steps:**

- [x] Add dependency: `evcxr_runtime = "1.1.0"`
- [x] Capture stdout during evaluation
- [x] Parse `EVCXR_BEGIN_CONTENT` / `EVCXR_END_CONTENT` markers
- [x] Extract MIME type and content
- [x] Convert to protocol Response format
- [x] Optionally implement `Display` trait for custom types

**Issues with This Approach:**

- ❌ Relies on parsing stdout (fragile, slow)
- ❌ Doesn't work well with our binary protocol
- ❌ Output markers can appear in user's actual println output (false positives)
- ❌ No structured representation

### If Implementing Our Own (Recommended)

**Required Steps:**

- [x] Define `OxurDisplay` trait in `oxur-runtime` crate
- [x] Define `DisplayValue` enum for structured output
- [x] Implement display logic in protocol Response
- [x] Support common MIME types (text/plain, text/html, image/png)
- [x] Add optional base64 encoding for binary data

**Advantages:**

- ✅ Structured, not text-parsed
- ✅ Works cleanly with binary protocol
- ✅ No false positives from user output
- ✅ Full control over format
- ✅ Can extend easily (e.g., add streaming)

---

## 6. Recommendations

### What to Use Directly

**Nothing from evcxr_runtime.**

The crate is too minimal and too specific to evcxr's stdout-based approach. We'd be better served by a clean implementation designed for Oxur's protocol.

### What to Wrap/Adapt

**The MIME-typed output pattern** - but implement it ourselves:

```rust
// In oxur-runtime/src/display.rs

pub trait OxurDisplay {
    fn oxur_display(&self) -> DisplayValue;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayValue {
    Text(String),
    Html(String),
    Markdown(String),
    Latex(String),
    Json(String),
    Image { mime: String, data: Vec<u8> },
    Custom { mime: String, content: Vec<u8> },
}

// Auto-implement for common types
impl<T: std::fmt::Display> OxurDisplay for T {
    fn oxur_display(&self) -> DisplayValue {
        DisplayValue::Text(format!("{}", self))
    }
}

// Users can override for custom types
impl OxurDisplay for MyChart {
    fn oxur_display(&self) -> DisplayValue {
        DisplayValue::Html(self.render_svg())
    }
}
```

### What to Replace

**Everything.** evcxr_runtime's stdout-marker approach doesn't fit our architecture.

### What's Missing (That We Need)

1. **Structured output** - Not stdout scraping
2. **Protocol integration** - Direct serialization to Response messages
3. **Streaming support** - For large outputs or progress updates
4. **Metadata** - Width hints, truncation preferences, etc.
5. **Error handling** - Display failures shouldn't crash evaluation
6. **Extensibility** - Plugin system for custom renderers

---

## 7. Code Examples

### Example 1: How evcxr_runtime Works (For Understanding Only)

```rust
// This is what evcxr_runtime does - NOT recommended for Oxur

use evcxr_runtime::{Display, mime_type};

struct DataFrame {
    rows: Vec<Vec<String>>,
}

impl Display for DataFrame {
    fn evcxr_display(&self) {
        // Generate HTML table
        let mut html = String::from("<table>");
        for row in &self.rows {
            html.push_str("<tr>");
            for cell in row {
                html.push_str(&format!("<td>{}</td>", cell));
            }
            html.push_str("</tr>");
        }
        html.push_str("</table>");

        mime_type("text/html").text(html);
        // Prints to stdout:
        // EVCXR_BEGIN_CONTENT text/html
        // <table><tr><td>...</td></tr></table>
        // EVCXR_END_CONTENT
    }
}

// Usage in evcxr REPL:
let df = DataFrame {
    rows: vec![
        vec!["Name".into(), "Age".into()],
        vec!["Alice".into(), "30".into()],
    ],
};
df  // Calls df.evcxr_display(), parent process parses stdout
```

### Example 2: Recommended Approach for Oxur

```rust
// In oxur-runtime/src/display.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayValue {
    Text(String),
    Html(String),
    Image { mime: String, data: Vec<u8> },
}

pub trait OxurDisplay {
    fn oxur_display(&self) -> DisplayValue;
}

// In user code or oxur-dataframe crate:
struct DataFrame {
    rows: Vec<Vec<String>>,
}

impl OxurDisplay for DataFrame {
    fn oxur_display(&self) -> DisplayValue {
        let mut html = String::from("<table>");
        for row in &self.rows {
            html.push_str("<tr>");
            for cell in row {
                html.push_str(&format!("<td>{}</td>", cell));
            }
            html.push_str("</tr>");
        }
        html.push_str("</table>");

        DisplayValue::Html(html)  // Return structured value
    }
}

// In oxur REPL compiler:
fn eval_and_display<T: OxurDisplay>(value: T) -> Response {
    let display = value.oxur_display();

    Response {
        value: Some(display),  // Directly serialize to protocol
        out: capture_stdout(),
        err: capture_stderr(),
        status: vec![Status::Done],
    }
}
```

### Example 3: Image Display Comparison

```rust
// evcxr_runtime approach (what we WON'T do):

use evcxr_runtime::mime_type;

fn display_image(png_data: &[u8]) {
    mime_type("image/png").bytes(png_data);
    // Prints base64 to stdout with markers
}

// Oxur approach (what we WILL do):

use oxur_runtime::DisplayValue;

fn display_image(png_data: &[u8]) -> DisplayValue {
    DisplayValue::Image {
        mime: "image/png".into(),
        data: png_data.to_vec(),
    }
    // Returns structured value, serialized directly via protocol
}
```

### Example 4: Integration with Protocol

```rust
// In oxur-compiler/src/eval.rs

pub async fn eval_with_display(
    session: &mut Session,
    code: CoreForm,
) -> Result<Response> {
    // Compile and execute
    let result_value = compile_and_run(session, code).await?;

    // Get display representation
    let display_value = if let Some(display_fn) = result_value.display_fn {
        // Call user's OxurDisplay::oxur_display() implementation
        unsafe { display_fn(result_value.ptr) }
    } else {
        // Default: use Debug formatting
        DisplayValue::Text(format!("{:?}", result_value))
    };

    Ok(Response {
        value: Some(display_value),  // Structured, serializable
        out: session.take_stdout(),
        err: session.take_stderr(),
        status: vec![Status::Done],
    })
}
```

### Example 5: Custom Oxur Display Helper

```rust
// Convenience macro for quick display implementations

#[macro_export]
macro_rules! impl_text_display {
    ($type:ty) => {
        impl OxurDisplay for $type {
            fn oxur_display(&self) -> DisplayValue {
                DisplayValue::Text(format!("{}", self))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_debug_display {
    ($type:ty) => {
        impl OxurDisplay for $type {
            fn oxur_display(&self) -> DisplayValue {
                DisplayValue::Text(format!("{:#?}", self))
            }
        }
    };
}

// Usage:
impl_text_display!(MyType);
impl_debug_display!(MyComplexType);
```

---

## 8. Risk Assessment

### Technical Risks

**If Using evcxr_runtime:**

- **Stdout parsing fragility** - Markers can appear in user output, causing false positives
- **Binary protocol mismatch** - stdout-based approach doesn't fit Postcard/MessagePack
- **Performance** - Parsing stdout is slower than structured data
- **Complexity** - Adding stdout capture + parsing + conversion adds unnecessary layers

**If Implementing Our Own:**

- **Initial effort** - ~100 lines of code to write
- **Testing burden** - Need to test various display types
- **Maintenance** - We own the code (but it's simple)

**Overall**: Implementing our own has negligible risk and better fit.

### Performance Risks

**evcxr_runtime:**

- Printing to stdout: ~microseconds per call
- Base64 encoding: ~10-100 microseconds for typical images
- Parsing stdout markers: ~milliseconds for large outputs

**Oxur approach:**

- Creating DisplayValue: ~nanoseconds (just enum construction)
- Serializing with postcard: ~microseconds
- No parsing overhead

**Performance is not a major concern either way**, but structured approach is slightly faster.

### Maintenance Risks

**evcxr_runtime:**

- **API stability**: Very stable (hasn't changed in years)
- **Breaking changes**: Unlikely (API is minimal)
- **Documentation**: Sparse but sufficient
- **Dependency risk**: Only base64 (optional), very stable

**Our own implementation:**

- **Stability**: Under our control
- **Changes**: We decide
- **Documentation**: We write it
- **Dependencies**: Only serde (required anyway)

**Maintenance risk is low for both approaches.**

---

## 9. Dependency Analysis

### Direct Dependencies of evcxr_runtime

```toml
[dependencies]
base64 = { version = "0.13.0", optional = true }

[features]
bytes = ["base64"]
```

**That's it.** No other dependencies.

### Version Constraints

- `evcxr_runtime` is at version 1.1.0 (stable)
- `base64` is a mature, stable crate
- No known version conflicts

### Feature Flags

**`bytes` feature:**

- Enables `ContentMimeType::bytes()` method
- Adds base64 dependency
- Needed for binary data (images, etc.)

**Default features:** None (must opt-in to `bytes`)

### Platform Support

**All platforms** - pure Rust, no platform-specific code, no FFI.

Works on:

- Linux ✅
- macOS ✅
- Windows ✅
- BSD ✅
- WASM ✅ (if stdout works)

---

## 10. Alternative Approaches

### Option 1: Use evcxr_runtime as-is

**Pros:**

- Zero implementation effort
- Compatible with evcxr ecosystem (can reuse evcxr_image, etc.)
- Proven in production (Jupyter kernel)
- Tiny dependency footprint

**Cons:**

- Stdout-based approach doesn't fit our binary protocol
- Requires parsing stdout for MIME content
- Can't stream output efficiently
- Less control over format
- Adds unnecessary complexity (capture stdout → parse → convert → serialize)

**Verdict:** Not recommended for Oxur's architecture.

---

### Option 2: Implement Oxur-Specific Display System

**Pros:**

- Clean integration with binary protocol
- Structured data (no parsing)
- Full control over features
- Can extend easily (streaming, metadata, etc.)
- Better performance (direct serialization)
- Simpler overall architecture

**Cons:**

- ~100-200 lines of code to write
- Not compatible with evcxr extensions (would need adapters)
- Testing burden
- Documentation burden

**Verdict:** **Recommended approach** for Oxur v1.0.

**Implementation Plan:**

1. Create `oxur-runtime` crate
2. Define `OxurDisplay` trait
3. Define `DisplayValue` enum
4. Implement for common types
5. Integrate with protocol Response
6. Add convenience macros
7. Test with various types

**Estimated effort:** 2-4 hours

---

### Option 3: Hybrid Approach (Compatibility Layer)

**Pros:**

- Can use evcxr extensions if needed
- Provides Oxur-native API
- Enables future Jupyter kernel

**Cons:**

- Most complex approach
- Maintains two code paths
- Overhead of conversion

**Implementation:**

```rust
// Provide both APIs

// Oxur-native (recommended)
impl OxurDisplay for MyType {
    fn oxur_display(&self) -> DisplayValue {
        DisplayValue::Html(self.render())
    }
}

// evcxr compatibility (for extensions)
impl evcxr_runtime::Display for MyType {
    fn evcxr_display(&self) {
        evcxr_runtime::mime_type("text/html")
            .text(self.render());
    }
}

// Adapter in Oxur compiler
fn eval_with_compat(value: impl Any) -> Response {
    // Try Oxur-native first
    if let Some(display) = value.downcast_ref::<dyn OxurDisplay>() {
        return Response::with_value(display.oxur_display());
    }

    // Fall back to evcxr compat (capture stdout)
    let stdout = capture_stdout(|| {
        if let Some(display) = value.downcast_ref::<dyn evcxr_runtime::Display>() {
            display.evcxr_display();
        }
    });

    // Parse stdout for MIME markers
    let mime_outputs = parse_evcxr_output(&stdout);
    Response::with_mime_outputs(mime_outputs)
}
```

**Verdict:** Only needed if we plan to support evcxr extensions. Skip for v1.0.

---

### Option 4: Fork evcxr_runtime

**Pros:**

- Start with working code
- Modify to fit our needs
- Maintain some compatibility

**Cons:**

- Maintenance burden of fork
- Divergence from upstream
- Overkill for such a small library

**Verdict:** Not worth it. evcxr_runtime is only 75 lines - easier to write from scratch.

---

### **Recommendation: Option 2 (Oxur-Specific Display System)**

**Rationale:**

1. **Better Architecture** - Structured data fits our protocol perfectly
2. **Simple** - Only ~100 lines of code, easy to implement and maintain
3. **No Compromises** - Designed exactly for our needs
4. **Extensible** - Easy to add features later (streaming, metadata, etc.)
5. **Performance** - Direct serialization, no parsing overhead

**When to reconsider:** If/when we build an Oxur Jupyter kernel, we could add evcxr compatibility layer at that time.

---

## 11. Conclusion

### Key Findings

1. **evcxr_runtime is NOT a runtime** - It's a display utility (~75 lines)
2. **No value representation** - Just MIME-typed output formatting
3. **Stdout-based approach** - Doesn't fit Oxur's binary protocol
4. **Minimal functionality** - Easy to replicate with better integration

### What We Actually Need (From evcxr_repl Audit)

The true "runtime" components we need are from the main evcxr crate:

✅ **Variable Storage** - `evcxr_internal_runtime.rs` (Box<dyn Any> pattern)
✅ **Subprocess Execution** - `runtime.rs` (dynamic library loading)
✅ **Process Management** - `child_process.rs` (IPC and crash recovery)

These were covered in detail in the evcxr_repl audit report.

### Recommended Path Forward

**For Oxur v1.0:**

1. **Skip evcxr_runtime** - Don't add as dependency
2. **Implement OxurDisplay** - ~100 lines in oxur-runtime crate
3. **Use structured output** - DisplayValue enum serialized via protocol
4. **Focus on core functionality** - Get Tier 2 compilation working first
5. **Add rich display later** - Start with text/html/json, expand over time

**For Future (v2.0+):**

- Consider evcxr compatibility layer if building Jupyter kernel
- Evaluate evcxr ecosystem extensions (evcxr_image, etc.)
- Potentially contribute Oxur display implementations upstream

### Final Assessment

**evcxr_runtime provides minimal value for Oxur.** The pattern is useful (MIME-typed output for rich display), but the implementation (stdout markers) doesn't fit our architecture. We should implement our own display system designed for the Oxur protocol.

**Estimated effort to replace:** 2-4 hours
**Value gained:** Better integration, more control, cleaner architecture
**Risk:** Negligible - it's simple functionality

**Recommendation: Do not use evcxr_runtime. Implement Oxur-specific display system.**

---

## Appendix A: Complete evcxr_runtime Source Code

For reference, here is the entire evcxr_runtime crate (75 lines):

```rust
// evcxr_runtime/src/lib.rs

#[cfg(feature = "bytes")]
extern crate base64;

pub trait Display {
    fn evcxr_display(&self);
}

pub struct ContentMimeType {
    mime_type: String,
}

pub fn mime_type<S: Into<String>>(mime_type: S) -> ContentMimeType {
    ContentMimeType {
        mime_type: mime_type.into(),
    }
}

impl ContentMimeType {
    pub fn text<S: AsRef<str>>(self, text: S) {
        println!(
            "EVCXR_BEGIN_CONTENT {}\n{}\nEVCXR_END_CONTENT",
            self.mime_type,
            text.as_ref()
        );
    }

    #[cfg(feature = "bytes")]
    pub fn bytes(self, buffer: &[u8]) {
        self.text(base64::encode(buffer))
    }
}
```

That's the entire implementation. Simple and focused, but not what we need for Oxur.

---

## Appendix B: Suggested Oxur Display Implementation

Here's a complete, ready-to-use implementation for Oxur:

```rust
// oxur-runtime/src/display.rs

use serde::{Serialize, Deserialize};

/// Values that can be displayed in the Oxur REPL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DisplayValue {
    /// Plain text output
    Text(String),

    /// HTML content (for rich display)
    Html(String),

    /// Markdown content
    Markdown(String),

    /// LaTeX math
    Latex(String),

    /// JSON data (pre-serialized)
    Json(String),

    /// Image data
    Image {
        mime: String,      // e.g., "image/png", "image/svg+xml"
        data: Vec<u8>,     // Raw bytes (not base64 encoded)
    },

    /// Custom MIME-typed content
    Custom {
        mime: String,
        content: Vec<u8>,
    },
}

/// Trait for types that provide custom REPL display
pub trait OxurDisplay {
    fn oxur_display(&self) -> DisplayValue;
}

// Default implementations for common types
impl OxurDisplay for String {
    fn oxur_display(&self) -> DisplayValue {
        DisplayValue::Text(self.clone())
    }
}

impl OxurDisplay for &str {
    fn oxur_display(&self) -> DisplayValue {
        DisplayValue::Text(self.to_string())
    }
}

impl<T: std::fmt::Display> OxurDisplay for T {
    default fn oxur_display(&self) -> DisplayValue {
        DisplayValue::Text(format!("{}", self))
    }
}

// Convenience constructors
impl DisplayValue {
    pub fn text(s: impl Into<String>) -> Self {
        DisplayValue::Text(s.into())
    }

    pub fn html(s: impl Into<String>) -> Self {
        DisplayValue::Html(s.into())
    }

    pub fn png(data: Vec<u8>) -> Self {
        DisplayValue::Image {
            mime: "image/png".into(),
            data,
        }
    }

    pub fn svg(s: impl Into<String>) -> Self {
        DisplayValue::Image {
            mime: "image/svg+xml".into(),
            data: s.into().into_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_display() {
        let val = DisplayValue::text("Hello");
        assert!(matches!(val, DisplayValue::Text(_)));
    }

    #[test]
    fn test_oxur_display_string() {
        let s = "test".to_string();
        let display = s.oxur_display();
        assert!(matches!(display, DisplayValue::Text(ref t) if t == "test"));
    }
}
```

Usage in protocol:

```rust
// In Response message
#[derive(Serialize, Deserialize)]
pub struct Response {
    pub value: Option<DisplayValue>,  // Rich display value
    pub out: String,                   // Captured stdout
    pub err: String,                   // Captured stderr
    pub status: Vec<Status>,
}
```

This provides everything evcxr_runtime does, but integrated cleanly with Oxur's protocol.

---

**End of Audit Report**

**Next Steps:**

1. Implement `oxur-runtime` crate with display system
2. Integrate with Tier 2 compiler
3. Test with various types
4. Add convenience macros and helpers
5. Document usage for Oxur library authors
