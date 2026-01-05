# Evcxr Architectural Archaeology: Critical Lessons for Oxur

**Evcxr's subprocess model exists primarily because Rust threads cannot be forcibly interrupted.** After analyzing GitHub issues, maintainer interviews, and release notes, the most important insight for Oxur is this: subprocess isolation isn't just about crash recovery—it's the *only* way to support Ctrl-C interruption in a Rust REPL. This single constraint drove evcxr's entire execution architecture. The project also reveals that its stdin/stdout IPC protocol, while simple, has inherent fragility that a socket-based approach would solve. Evcxr's variable persistence through `Box<dyn Any + 'static>` type erasure works but imposes strict limitations—variables cannot reference other variables. These findings validate several of Oxur's tentative decisions while suggesting refinements.

## Subprocess isolation solves the unsolvable Rust interruption problem

The case for subprocess execution is stronger than initially apparent. From the HOW_IT_WORKS.md documentation, David Lattimore explicitly lists four reasons for subprocess execution:

1. **Crash recovery**: "It allows us to restart everything if the subprocess segfaults due to some bad unsafe code"
2. **Simpler I/O management**: "We can use our stdout/stderr for printing stuff, since we didn't redirect them"
3. **Multi-context isolation**: "It keeps things isolated if running multiple EvaluationContexts at once"
4. **Portability**: "It's probably easier to port since we don't need to capture our own stdout/stderr"

However, the **most critical reason** emerges from release notes discussing Jupyter interruption support: "Don't ask Jupyter to 'interrupt kernel', it won't work. Rust threads can't be interrupted." The Jupyter README explicitly states this limitation. When evcxr added Ctrl-C support, it worked by **terminating the subprocess**—something fundamentally impossible with in-process execution since Rust has no mechanism for forcibly terminating a running thread.

This is a decisive architectural constraint. If Oxur requires interactive interruption (which any development REPL should), subprocess execution becomes **mandatory**, not optional. There's no evidence evcxr ever tried in-process execution—the design was subprocess-based from inception based on David Lattimore's prior experience with linkers and dynamic library loading.

The tradeoff is explicit: "Support interrupting execution in Jupyter notebook. **Process gets terminated, so variables are lost**, but other state is preserved." Additionally, COMMON.md warns: "If your code segfaults (e.g. due to buggy unsafe code), aborts, exits etc, the process in which the code runs will be restarted. **All variables will be lost.**"

## Stdin/stdout protocol works but has known fragility issues

Evcxr uses a text-based, line-oriented protocol over stdin/stdout with special marker strings:

| Marker | Purpose |
|--------|---------|
| `EVCXR_EXECUTION_COMPLETE` | Signals end of execution |
| `EVCXR_BEGIN_CONTENT {mime-type}` | Starts a mime-typed output block |
| `EVCXR_END_CONTENT` | Ends a mime-typed output block |
| `EVCXR_VARIABLE_CHANGED_TYPE:{name}` | Variable type change notification |
| `EVCXR_ERROR_OCCURRED` | User error via `?` operator |

The protocol has **three inherent weaknesses**:

**Stdout mixing**: User code can print arbitrary content to stdout, potentially colliding with protocol markers. The documentation explicitly advises: "It's probably a good idea to either print the whole block at once, or to lock stdout then print the block. This should ensure that nothing else prints to stdout at the same time." Release notes mention "Reduced interleaving of stdout and stderr" as a recurring fix.

**No framing or escaping**: The protocol relies on exact string matching of marker lines with no length-prefix framing. If user code prints a line exactly matching `EVCXR_EXECUTION_COMPLETE`, parsing breaks.

**Binary content overhead**: Binary data (images) must be base64 encoded, adding **~33% size overhead** and encoding/decoding cost.

No evidence exists of sockets, named pipes, or binary protocols ever being considered. The choice appears pragmatic: stdin/stdout works identically across Windows, macOS, and Linux with zero platform-specific code.

## Variable persistence uses type erasure with significant limitations

Evcxr's variable persistence mechanism stores all variables in a `HashMap<String, Box<dyn Any + 'static>>` within the `evcxr_internal_runtime` crate. The approach works as follows:

1. Parse user code with rust-analyzer to identify variable declarations
2. Generate wrapper code that stores variables into the HashMap after execution
3. Compile everything to a shared object (.so/.dll)
4. Load and execute via dlopen/dlsym in the subprocess
5. Restore variables from HashMap before next execution

The type inference story reveals **significant technical debt and evolution**. Originally, evcxr used a clever hack: intentionally declare all variables as `String` type, then extract the *actual* types from rustc's error messages. The author acknowledged: "this ends up less hacky than it sounds (although it's still obviously not ideal)."

This approach **broke catastrophically** with Rust 1.48 (Issue #138) when rustc changed error message formatting to omit fully-qualified type paths. David Lattimore's solution was integrating rust-analyzer as a library for type inference. By 0.13.0, the error-message-parsing code was completely removed: "The last bit of code that tried to use rustc error messages to determine variable types has been deleted. Now if rust-analyzer can't determine the type of a variable, we ask the user to add an explicit type annotation."

The **fundamental limitation** of this approach: **Variables cannot reference other variables.** From COMMON.md:

```rust
let all_values = vec![10, 20, 30, 40, 50];
let some_values = &all_values[2..3];  // ERROR: Cannot persist references
```

This is inherent to type erasure with `'static` lifetime requirements. The documented workaround using `Box::leak` exists but leaks memory. Notably, **evcxr does NOT use serialization**—the HashMap lives entirely in subprocess memory. No serde, no serialization overhead, but no persistence across process restarts.

## Maintainer regrets and lessons from five years of development

David Lattimore's 2021 interview reveals key retrospective insights:

**On rust-analyzer integration**: "Probably my biggest complaint would be that it's quite big and so it takes a while to compile. Certainly my continuous integration slowed down quite a lot when I pulled in rust-analyzer because now every time it runs it has to build all of rust-analyzer which takes a while. An alternative way that I could have integrated that I didn't consider at the time was to actually pull in the rust-analyzer binary and talk to it using the language server protocol, instead of using it as a library. Obviously, that would have been better from a compilation time perspective."

**On API stability**: "The API of the library is very *not* stable. I make breaking changes to the API of that library moderately often."

**On why rusti died**: "I think the issue there was that they were using some compiler internals, and those compiler internals got removed, so there was no way they could move forward... It worked completely differently to the way Evcxr works." This validates avoiding deep compiler integration.

The **preserve_vars_on_panic** feature went through significant evolution—initially enabled, then disabled when "it was significantly slowing down compilation," then re-enabled once the implementation improved. This suggests panic recovery is harder than it appears and has performance implications.

Dynamic linking proved problematic: version 0.17.0 "Reverted to static linking by default as we had prior to 0.16.0. You can still get dynamic linking by setting :allow_static_linking 0 which is recommended if it works for you. Forcing dynamic linking was breaking in hard-to-debug ways for several people on both Mac and Linux."

## Problems Oxur might not have anticipated

**Async is fundamentally challenging**: The release notes document ongoing issues: "stuck on pre-1.0 tokio due to rustc changes to dynamic libraries." Async support was eventually added but required building and managing a Tokio runtime, adding significant complexity.

**Struct redefinition is impossible**: A Hacker News user noted: "while you can redefine values and functions when you're iterating, you cannot do so with structs." This is a fundamental limitation of evcxr's compilation model—types are fixed once defined.

**Performance requires caching**: Speed concerns recur in community feedback. IRust developers describe "evcxr is painfully slow." The solution in 0.17.0 was a "New built-in caching mechanism. Enable a 500MiB cache by adding :cache 500 to your init.evcxr." Caching isn't optional—it's essential for acceptable performance.

**Variable shadowing differs from Rust**: Issue #109 documents that variable shadowing in evcxr doesn't match Rust semantics, causing user confusion.

**Mutable statics require dynamic linking**: "Now compiles dependencies as dylibs. This means that mutable static variables in dependencies are now preserved between executions." But dynamic linking causes other problems (see above).

## Specific recommendations for Oxur's decisions

**Subprocess execution: Keep it.** The Rust thread interruption limitation is fatal for in-process execution in any interactive context. The `Executor` trait abstraction is still valuable—it enables testing with mock executors and potentially future optimizations for specific use cases where interruption isn't needed.

**Unix sockets + protocol reuse: Strong choice.** This solves evcxr's three IPC weaknesses:
- Dedicated control channel separates protocol from user stdout
- Binary-capable framing eliminates marker collision risk
- No base64 overhead for binary content

Consider adding a platform abstraction for Windows (named pipes or TCP localhost fallback). A length-prefixed binary protocol with JSON-RPC or similar structure would be more robust than evcxr's line-based markers.

**Server-side compilation: Validated.** Evcxr's approach of compiling to shared objects and dynamic loading works. Avoid compiler internals (rusti's fatal mistake). Consider rust-analyzer via LSP rather than as a library to reduce build times—this is David Lattimore's stated regret.

**Variable persistence strategy**: 
- `Box<dyn Any + 'static>` type erasure is proven to work
- Accept the "no inter-variable references" limitation or document workarounds
- Consider serialization for crash recovery (evcxr loses all state on subprocess death)
- rust-analyzer integration for type inference is now mature and necessary

**Caching is mandatory**: Build caching infrastructure from day one. Evcxr's experience shows this is essential for acceptable iteration speed.

## Open questions requiring Oxur-specific decisions

**Should Oxur support struct redefinition?** Evcxr cannot. This would require a fundamentally different approach—possibly code versioning or namespacing of user types. Unclear if the complexity is worthwhile.

**What is the serialization strategy for crash recovery?** Evcxr accepts losing all variables on crash/interrupt. Is this acceptable for Oxur's use cases? Serialization (serde) could enable recovery but limits supported types.

**How will Oxur handle async?** Evcxr's async support required significant runtime management. Will Oxur build a Tokio runtime? Support multiple executors?

**What's the Windows story?** Unix sockets don't exist on Windows. Evcxr's stdin/stdout approach is maximally portable. Oxur needs a fallback strategy—named pipes, TCP localhost, or Windows-specific code.

**Should Oxur support multiple evaluation contexts?** Evcxr's subprocess model explicitly enables running "multiple EvaluationContexts at once" with isolation. Is this a requirement?

**What's the source mapping approach?** Evcxr's "dedicated source mapping crate" approach (as mentioned in Oxur's tentative decisions) isn't explicitly documented in evcxr's architecture. This appears to be novel work for Oxur.

## Conclusion

Evcxr's five-year journey validates subprocess execution as **necessary, not just desirable**, due to Rust's thread interruption limitations. Its stdin/stdout protocol works but has inherent fragility that Oxur's socket-based approach should solve. Variable persistence through type erasure is proven but fundamentally limited—references between variables are impossible. The critical lessons are: avoid compiler internals (stay alive when rustc changes), integrate rust-analyzer (preferably via LSP), build robust caching from day one, and accept that some Rust semantics (struct redefinition, variable borrowing) will not translate to REPL context. Oxur's tentative architecture aligns well with evcxr's evolved wisdom while offering opportunities to improve on its protocol fragility and crash recovery limitations.