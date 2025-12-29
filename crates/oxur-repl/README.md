# oxur-repl

REPL implementation with three-tier execution for optimal performance.

## Architecture

### Three-Tier Execution

1. **Tier 1 - Interpreter** (<1ms)
   - Direct interpretation for simple expressions
   - Fast startup, good for interactive exploration

2. **Tier 2 - Cached** (~0ms)
   - Previously compiled functions
   - Just function call overhead

3. **Tier 3 - JIT** (50-200ms first time, cached after)
   - Full compilation for complex code
   - Native performance after first run

## Protocol

The REPL uses a simple request/response protocol:

- `Eval` - Evaluate an expression
- `Load` - Load a file
- `Reset` - Clear REPL state
- `Status` - Get execution statistics
- `Shutdown` - Stop the server

## Usage

```rust
use oxur_repl::ReplServer;

let mut server = ReplServer::new();
let response = server.handle(ReplRequest::Eval {
    source: "(+ 1 2)".to_string()
});
```
