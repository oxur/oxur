# oxur-lang

Language processing for Oxur: parsing, macro expansion, and Core Forms IR.

## Stages

### Stage 1: Parse
Converts raw Oxur source into Surface Forms (S-expression AST).

### Stage 2: Expand
Transforms Surface Forms into Core Forms through macro expansion and desugaring.

## Core Forms

Core Forms are the canonical intermediate representation that serves as the stable
contract between the Oxur frontend and the Rust backend. All syntactic sugar and
macros are expanded into these forms.

## Source Maps

The source map tracks every transformation from original source through to Rust AST,
enabling accurate error reporting that points back to the original Oxur code.
