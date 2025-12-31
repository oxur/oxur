# Task: Create an AI-Optimized Rust Best Practices Reference

## Context

I have several PDF files containing Rust best practices, guidelines, and patterns. I want you to:

1. Extract and synthesize the content from these PDFs
2. Create a modular collection of Markdown files optimized for AI/LLM consumption when coding or reviewing Rust

## Source PDFs (in priority order)

- RustDesignPatterns.pdf
- RustAPIGuidelines.pdf
- ClippyDocumentation.pdf
- TheRustStyleGuide.pdf
- PragmaticRustGuidelines.pdf
- AsynchronousProgrammingInRust.pdf
- TheLittleBookOfRustMacros.pdf

## Output Structure

Create a directory called `rust-ai-guidelines/` with:

````
rust-ai-guidelines/
├── README.md                    # Index and overview
├── 01-core-idioms.md            # Essential Rust idioms
├── 02-api-design.md             # Public API design guidelines
├── 03-error-handling.md         # Result, Option, error types
├── 04-ownership-borrowing.md    # Lifetime patterns, borrow checker strategies
├── 05-type-design.md            # Structs, enums, newtypes, generics
├── 06-traits.md                 # Trait design, impl patterns, trait objects
├── 07-concurrency-async.md      # Async patterns, Send/Sync, threading
├── 08-performance.md            # Allocation, cloning, iterators
├── 09-unsafe-ffi.md             # Unsafe guidelines, FFI patterns
├── 10-macros.md                 # Declarative and procedural macros
├── 11-anti-patterns.md          # What NOT to do (critical for AI)
├── 12-project-structure.md      # Crate organization, modules, visibility
└── 13-documentation.md          # Doc comments, examples, rustdoc
````

## Content Guidelines

### Format for AI consumption

- Use clear section headers
- Include concrete code examples for EVERY pattern
- Use "Prefer X over Y" format where applicable
- Include brief rationale (1-2 sentences) for each guideline
- Mark guidelines by strength: MUST, SHOULD, CONSIDER, AVOID
- Keep prose dense but readable — no filler

### For each pattern/guideline include

````markdown
### Pattern Name

**Strength**: MUST | SHOULD | CONSIDER | AVOID

**Summary**: One sentence description.

**Example**:
```rust
// Good
fn example_good() { ... }

// Bad (if applicable)
fn example_bad() { ... }
```

**Rationale**: Why this matters (1-2 sentences).

**See also**: Related patterns, Clippy lints, etc.
````

### Anti-patterns section is critical

AI models frequently generate these mistakes. For each anti-pattern:

- Show the problematic code
- Explain WHY it's wrong
- Show the correct alternative

## Extraction Instructions

1. First, install PDF extraction tools:

````bash
   pip install pymupdf --break-system-packages
````

1. Extract text from each PDF:

````python
   import fitz  # PyMuPDF

   def extract_pdf(path):
       doc = fitz.open(path)
       text = ""
       for page in doc:
           text += page.get_text()
       return text
````

1. Process PDFs in priority order, building up each output file

2. Cross-reference between sources — if multiple sources discuss the same topic, synthesize the best guidance

3. When sources conflict, prefer:
   - Rust API Guidelines (most authoritative for public APIs)
   - Clippy (enforced by tooling)
   - Design Patterns book (community consensus)

## Quality Checks

After generating, verify:

- [ ] All code examples compile (or are clearly marked as pseudocode)
- [ ] No section is just bullet points — include examples
- [ ] Anti-patterns section has at least 15 entries
- [ ] Each file is self-contained but cross-references related files

Begin by extracting the first PDF and showing me the proposed structure for `01-core-idioms.md`.
