# oxur-comp

Oxur compiler library and binary.

## Compiler Binary (`oxurc`)

```bash
# Compile an Oxur file
oxurc input.ox -o output

# Keep generated Rust source
oxurc input.ox --emit-rust

# Verbose output
oxurc input.ox -v
```

## Library

The `oxur-comp` library provides:

- **Lowering**: Core Forms → Rust AST (Stage 3)
- **Code Generation**: Rust AST → Rust source (Stage 4)
- **Compilation**: Rust source → Binary via rustc (Stage 5)

## Architecture

The compiler takes Core Forms (from `oxur-lang`) and:

1. Lowers them to Rust AST using the `syn` crate
2. Generates formatted Rust source using `prettyplease`
3. Invokes `rustc` to produce the final binary
