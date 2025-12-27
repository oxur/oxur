# Oxur Lisp Project Resources Analysis

*Updated December 25, 2025*

This document provides an updated assessment of the resources collected ~5 years ago for the Oxur Lisp project. Each resource has been evaluated for:

- **Link Status**: Whether the URL still works
- **Relevance**: How useful it is for building a Lisp-on-Rust with AST interop
- **Activity Level**: Recent development activity (High/Medium/Low/Archived)
- **Overall Rating**: ★★★★★ (5 stars = excellent, 1 star = poor/dead)

---

## Lisp / S-Expressions

### ★★★☆☆ ketos - <https://github.com/murarth/ketos>

**Status**: INACTIVE (last commit ~2019, ~6 years ago)
**Relevance**: High - Complete Lisp implementation
**Activity**: None (766 stars, author on indefinite hiatus)

A mature Lisp dialect designed specifically as a scripting/extension language for Rust programs. Features:

- Bytecode compilation
- REPL and file execution
- Good documentation
- v0.11 on crates.io (last updated 2019)
- **Author is on indefinite hiatus from all programming work**

**Recommendation**: Still useful as a reference implementation for understanding how to build a production-quality Lisp in Rust, but it's unmaintained and won't receive updates. Code quality is good but won't work with latest Rust features. Consider it an educational resource rather than a dependency.

---

### ★★★☆☆ lexpr-rs - <https://github.com/rotty/lexpr-rs>

**Status**: Active
**Relevance**: High - S-expression parsing library
**Activity**: Medium (172 stars, maintained)

Comprehensive S-expression parser and serializer with Serde integration. Features:

- Supports multiple Lisp dialects (Scheme R6RS/R7RS, Emacs Lisp)
- Excellent for parsing S-expressions as data
- Good for DSL surface syntax
- Version 0.2.7 on crates.io
- 247,391 total downloads

**Recommendation**: Ideal for just the parsing/serialization layer if you want to build your own evaluator. Not a full language implementation.

---

### ★★☆☆☆ macro-lisp - <https://github.com/JunSuzukiJapan/macro-lisp>

**Status**: Link works
**Relevance**: Low - Educational only
**Activity**: Low (appears inactive)

**Recommendation**: Limited information available. Appears to be a small educational project. Better alternatives exist.

---

### ★★☆☆☆ schemers - <https://github.com/mgattozzi/schemers>

**Status**: Blog posts still accessible
**Relevance**: Medium - Educational series
**Activity**: Archived/Inactive

A blog post series about building a Scheme interpreter. The repository appears to be from ~2017-2018.

**Recommendation**: Good educational resource for understanding the process, but not a production library. The blog posts are well-written and still valuable for learning.

---

### ★★☆☆☆ scheme.rs - <https://github.com/isamert/scheme.rs>

**Status**: Link works
**Relevance**: Low
**Activity**: Low/Inactive

**Recommendation**: Appears to be a small/abandoned project.

---

### ★★☆☆☆ oxischeme - <https://github.com/fitzgen/oxischeme>

**Status**: Link works
**Relevance**: Low
**Activity**: Inactive (last activity ~2015)

**Recommendation**: Old, abandoned project. Historical interest only.

---

### ★☆☆☆☆ rusty_scheme - <https://github.com/kenpratt/rusty_scheme>

**Status**: Link works
**Relevance**: Low
**Activity**: Inactive

**Recommendation**: Abandoned. Skip.

---

### ★★★☆☆ risp - <https://github.com/stopachka/risp>

**Status**: Active (still accessible)
**Relevance**: Medium-High - Educational
**Activity**: Low (67 stars, appears complete)

Accompanies the excellent blog post "Risp (in (Rust) (Lisp))" by Stepan Parunashvili. A small but complete Lisp implementation following Peter Norvig's approach.

**Blog post**: <https://stopa.io/post/222> (highly recommended reading)

**Recommendation**: Excellent educational resource for understanding how to build a Lisp interpreter in Rust. Not production-ready, but very clear code. Great starting point for learning.

---

### ★★☆☆☆ blispr - <https://github.com/deciduously/blispr>

**Status**: Link works
**Relevance**: Medium - Educational
**Activity**: Low

Accompanies the blog post "Rust Your Own Lisp". Another educational implementation.

**Recommendation**: Similar to risp - good for learning, not for production.

---

## Other Languages in Rust

### ★★★★☆ Interpreterbook - <https://interpreterbook.com/>

**Status**: Active, book available for purchase
**Relevance**: High - General interpreter design
**Activity**: Book is complete and maintained

The book "Writing an Interpreter in Go" with Rust implementations available:

- <https://github.com/shuhei/cymbal> (Rust implementation)
- Related blog post still accessible

**Recommendation**: Excellent general resource for interpreter design patterns, though not Lisp-specific.

---

### ★★★★☆ Cloudflare Fast Interpreters - <https://blog.cloudflare.com/building-fast-interpreters-in-rust/>

**Status**: Blog post still live
**Relevance**: High - Performance optimization
**Activity**: N/A (blog post)

**Recommendation**: Excellent technical blog post about interpreter performance in Rust. Still highly relevant for optimization strategies.

---

## Compiler / AST / Code Formatting

### ★★★★★ syn - <https://github.com/dtolnay/syn>

**Status**: Extremely Active
**Relevance**: VERY HIGH - Essential for Rust AST manipulation
**Activity**: Very High (2.3k stars, 1.2B+ downloads!)

The de-facto standard for parsing Rust code. Version 2.x is current. Absolutely essential for AST interop.

**Recommendation**: CRITICAL for your Oxur project. This is how you'll interact with Rust's AST. Extremely well-maintained by dtolnay (one of Rust's most prolific library authors).

---

### ★★★☆☆ Rust AST - <https://github.com/rust-lang/rust/blob/master/src/libsyntax/ast.rs>

**Status**: Link has moved
**Relevance**: Medium
**Activity**: Part of rustc

**Note**: The Rust compiler internals have reorganized since 2019. AST code is now in different locations. For most use cases, use `syn` instead.

**Recommendation**: Unless you're writing compiler plugins, use `syn` instead. The compiler's internal AST is unstable.

---

### ★★☆☆☆ kamalmarhubi AST examples

**Status**: Blog post accessible
**Relevance**: Low (outdated)
**Activity**: Historical

**Recommendation**: Outdated. Use modern `syn` documentation instead.

---

### ★☆☆☆☆ syntex_syntax - <https://crates.io/crates/syntex_syntax>

**Status**: Explicitly abandoned
**Relevance**: None
**Activity**: Dead

**Recommendation**: Skip. This was abandoned years ago. Use `syn`.

---

### ★★★★☆ rustfmt - <https://github.com/rust-lang/rustfmt>

**Status**: Very Active
**Relevance**: Medium - Code generation
**Activity**: Very High (official Rust tool)

**Recommendation**: Useful if you're generating Rust code from Lisp and want to format it nicely.

---

### ★☆☆☆☆ Rust plugin system - Deprecated

**Status**: Dead link
**Relevance**: None
**Activity**: Removed from Rust

**Recommendation**: This feature was removed from Rust. Use proc-macros instead.

---

## REPL / Jupyter

### ★★★★★ evcxr - <https://github.com/evcxr/evcxr>

**Status**: Very Active (moved to evcxr org from Google)
**Relevance**: Very High - Production REPL
**Activity**: Very High (actively maintained, latest release 2024)

Excellent Rust REPL and Jupyter kernel. Now maintained under the `evcxr` GitHub organization.

Features:

- Full REPL for Rust (evcxr_repl)
- Jupyter kernel (evcxr_jupyter)
- 138,089 downloads
- Regular releases
- Moved to Apache2/MIT dual license

**Recommendation**: The gold standard for Rust REPL. Highly relevant for understanding how to build interactive environments. Could be useful for an interactive Oxur REPL.

---

### ★☆☆☆☆ rusti - <https://github.com/murarth/rusti>

**Status**: Archived/old
**Relevance**: Low
**Activity**: Dead (explicitly noted as old)

**Recommendation**: Skip. Use evcxr instead.

---

## CLI / getopts / signals

### ★★★★★ clap - <https://github.com/clap-rs/clap>

**Status**: Extremely Active
**Relevance**: High - CLI parsing
**Activity**: Very High (v4.x current, massive ecosystem)

The dominant CLI argument parsing library for Rust. v4.x is current with derive macros.

**Downloads**: 1.2+ BILLION (one of the most-used Rust crates)

**Recommendation**: Essential for any CLI tool. Use this for Oxur's command-line interface. Very well documented and supported.

---

### ★★☆☆☆ getopts - <https://docs.rs/getopts/>

**Status**: Maintained but less popular
**Relevance**: Low
**Activity**: Low

**Recommendation**: Superseded by `clap`. No reason to use this for new projects.

---

### ★★★★☆ Signal handling - <https://rust-cli.github.io/book/in-depth/signals.html>

**Status**: Active (part of Rust CLI book)
**Relevance**: Medium
**Activity**: Maintained

**Recommendation**: Good resource for proper signal handling in CLI applications.

---

### ★★★☆☆ signal-hook - <https://crates.io/crates/signal-hook>

**Status**: Active
**Relevance**: Medium
**Activity**: Maintained

**Recommendation**: Good library for Unix signal handling if you need it.

---

## Config

### ★★★★☆ config-rs - <https://github.com/mehcode/config-rs>

**Status**: Active
**Relevance**: Medium
**Activity**: Maintained

**Recommendation**: Useful if you want configuration file support for Oxur. Not essential for MVP.

---

## Logging

### ★★★★★ log - <https://docs.rs/log/>

**Status**: Very Active
**Relevance**: Medium-High
**Activity**: Official ecosystem crate

**Recommendation**: Standard logging facade for Rust. Use this for any logging needs.

---

## Summary & Recommendations

### 🔥 CRITICAL RESOURCES (Use These)

1. **syn** - Absolutely essential for Rust AST interop
2. **clap** - Best CLI argument parsing
3. **evcxr** - Study this for REPL implementation ideas

### 📚 HIGHLY RECOMMENDED

1. **ketos** - Good reference for Lisp implementation patterns (unmaintained but well-designed)
2. **lexpr-rs** - If you need just S-expression parsing
3. **risp + blog post** - Excellent educational resource
4. **Interpreterbook** - General interpreter patterns
5. **Cloudflare blog post** - Performance optimization

### ⚠️ SKIP THESE

- syntex_syntax (abandoned)
- rusti (old)
- getopts (superseded by clap)
- Rust plugin system (removed from language)
- Most small/abandoned Scheme implementations

---

## Overall State Assessment

**Good News**: The Rust ecosystem has matured significantly in 5 years! The core tools you need (syn, clap, evcxr) are better than ever and very actively maintained.

**Reality Check**: Most of the Lisp-in-Rust projects from 2019-2020 are now unmaintained or abandoned, including ketos (the most complete one). This is actually normal for the ecosystem - many were educational projects or side projects whose authors moved on.

**Key Changes**:

- Most Lisp projects from 2019-2020 are now inactive (including ketos)
- Production tools (syn, clap) are thriving
- The `syn` crate has become the undisputed standard for Rust AST manipulation
- evcxr has become the standard Rust REPL
- No actively maintained production-ready Lisp-in-Rust currently exists

**For Oxur Lisp Project**:

- Start with `syn` for Rust AST interop (non-negotiable)
- Study `ketos` source code for implementation patterns (even though unmaintained, the code is well-designed)
- Use `risp` + blog for learning/prototyping
- Use `clap` for CLI
- Consider `lexpr-rs` if you want to focus on S-expression parsing separately
- **You'll likely need to build most of the Lisp implementation yourself** - there's no actively maintained production option to fork/extend

**Architecture Recommendation**:
Given modern tools, consider a two-layer approach:

1. Use `lexpr-rs` for S-expression parsing
2. Use `syn` for Rust AST generation/manipulation
3. Study `ketos` for runtime/evaluation patterns
4. Use `evcxr` patterns for REPL (if desired)

This gives you production-quality components while focusing your efforts on the unique Lisp↔Rust interop layer.
