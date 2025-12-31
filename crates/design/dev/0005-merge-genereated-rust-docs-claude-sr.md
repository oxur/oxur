# Task: Merge Multiple Versions of Rust Guidelines Documents

I have multiple versions of each Rust guidelines document (numbered 01-13) generated from different source materials. I need you to merge them into single, comprehensive documents.

## Input

I'm uploading multiple text/markdown files. Files with the same number prefix (e.g., `01-core-idioms.md`, `01-core-idioms-v2.md`, `01-core-idioms-clippy.md`) should be merged into a single output file.

## Merge Strategy

For each numbered document:

1. **Combine all unique patterns** — If version A has patterns ID-01 through ID-15 and version B has ID-01 through ID-12 plus ID-16 through ID-20, the merged doc should have ID-01 through ID-20.

2. **Deduplicate intelligently** — If the same pattern appears in multiple versions:
   - Keep the most complete/detailed explanation
   - Merge code examples if they show different aspects
   - Preserve all unique "See also" and Clippy lint references
   - Keep the strongest strength indicator (MUST > SHOULD > CONSIDER)

3. **Preserve structure** — Each merged document should maintain:
   - The standard header format with summary quote
   - Pattern sections with: Strength, Summary, code examples, Rationale, See also
   - Summary/checklist at the end
   - Cross-references to other documents

4. **Resolve conflicts** — If versions disagree:
   - Prefer more restrictive guidance (safer)
   - Prefer guidance with Clippy lint backing
   - Note if there's genuine ambiguity worth preserving

5. **Renumber if needed** — After merging, renumber patterns sequentially (ID-01, ID-02, etc.) within each document.

## Output

Create 13 merged markdown files (or 14 including README):

- `01-core-idioms.md`
- `02-api-design.md`
- `03-error-handling.md`
- `04-ownership-borrowing.md`
- `05-type-design.md`
- `06-traits.md`
- `07-concurrency-async.md`
- `08-performance.md`
- `09-unsafe-ffi.md`
- `10-macros.md`
- `11-anti-patterns.md`
- `12-project-structure.md`
- `13-documentation.md`
- `README.md` (updated with accurate pattern counts)

## Quality Checks

After merging each document:

- No duplicate patterns (same concept explained twice)
- All code examples are syntactically valid Rust
- Cross-references point to correct pattern IDs
- Consistent formatting throughout

## Example Merge

If I upload:

- `07-concurrency-async.md` (base version with CA-01 through CA-11)
- `07-concurrency-async-from-async-book.md` (has CA-01, CA-02 with more detail, plus CA-12 through CA-15)

The merged `07-concurrency-async.md` should have:

- CA-01 and CA-02: Enhanced versions combining both sources
- CA-03 through CA-11: From base version
- CA-12 through CA-15: New patterns from async book version

Please process all uploaded files and generate the merged documents.

```

---

## Tips for the Upload

1. **Rename files clearly** before uploading so it's obvious which go together:
```

   01-core-idioms-base.md
   01-core-idioms-clippy.md
   01-core-idioms-api-guidelines.md
   02-api-design-base.md
   02-api-design-api-guidelines.md
   ...
