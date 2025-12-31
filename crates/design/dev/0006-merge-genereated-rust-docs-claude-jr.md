# Task: Merge Multiple Versions of Rust Guidelines Documents

## Context

I have multiple versions of Rust guidelines documents in a directory. Each document has a number prefix (01-13) and multiple versions exist from different source materials. I need you to merge all versions of each numbered document into a single comprehensive file.

## Input Location

All input files are in: `./rust-guidelines-sources/`

The files follow naming patterns like:

- `01-core-idioms.md`, `01-core-idioms-clippy.md`, `01-core-idioms-v2.md`
- `02-api-design.md`, `02-api-design-api-guidelines.md`
- etc.

Files with the same number prefix should be merged together.

## Output Location

Write merged files to: `./rust-ai-guidelines/`

## Merge Strategy

For each numbered document (01-13):

### 1. Identify all versions

```bash
# Find all files starting with the same number
ls ./rust-guidelines-sources/01-*.md
```

### 2. Read and parse all versions

Extract patterns from each file. Patterns follow this structure:

```markdown
## XX-NN: Pattern Name

**Strength**: MUST | SHOULD | CONSIDER | AVOID

**Summary**: One-line description.

[Code examples in ```rust blocks]

**Rationale**: Explanation.

**See also**: References
```

### 3. Merge patterns using these rules

**Combining unique patterns:**

- If pattern exists in only one version → include as-is
- If same pattern ID exists in multiple versions → merge (see below)
- Collect all patterns, then renumber sequentially (XX-01, XX-02, ...)

**Merging duplicate patterns (same concept, possibly different IDs):**

- Use the most descriptive title
- Keep strongest strength indicator: MUST > SHOULD > CONSIDER > AVOID
- Combine summaries if they capture different aspects
- Include ALL unique code examples (both good and bad)
- Merge rationale sections, removing redundancy
- Preserve all Clippy lint references (format: `clippy::lint_name`)
- Combine "See also" references, deduplicate

**Detecting duplicates:**

- Same pattern ID (e.g., both have "ID-03")
- Very similar titles (e.g., "Use Borrowed Types" vs "Prefer Borrowed Types for Parameters")
- Same core code example

### 4. Preserve document structure

Each merged document must have:

```markdown
# Document Title

> Summary quote describing the document's scope.

---

## XX-01: First Pattern

...

## XX-02: Second Pattern

...

---

## Summary: [Topic] Checklist

- [ ] Checklist item 1
- [ ] Checklist item 2

---

*See also: [cross-references to other documents]*
```

### 5. Update pattern counts and cross-references

After merging:

- Renumber all patterns sequentially (XX-01, XX-02, ...)
- Update any internal cross-references to use new pattern IDs
- Fix cross-references to other documents if pattern IDs changed

## Processing Order

Process files in this order:

1. `01-core-idioms` → prefix: ID
2. `02-api-design` → prefix: API
3. `03-error-handling` → prefix: EH
4. `04-ownership-borrowing` → prefix: OB
5. `05-type-design` → prefix: TD
6. `06-traits` → prefix: TR
7. `07-concurrency-async` → prefix: CA
8. `08-performance` → prefix: PF
9. `09-unsafe-ffi` → prefix: US
10. `10-macros` → prefix: MC
11. `11-anti-patterns` → prefix: AP
12. `12-project-structure` → prefix: PS
13. `13-documentation` → prefix: DC

## Final Steps

After merging all documents:

1. **Create/update README.md** with:
   - Accurate pattern counts per document
   - Updated total pattern count
   - Current date

2. **Validation report** — Print summary:

```
   Merge Summary:
   - 01-core-idioms.md: X patterns (merged from N files)
   - 02-api-design.md: X patterns (merged from N files)
   ...
   Total: XXX patterns across 13 documents
```

1. **List any conflicts** that required judgment calls

## Example Workflow

```bash
# 1. List all source files grouped by number
for i in $(seq -w 1 13); do
  echo "=== ${i} ==="
  ls ./rust-guidelines-sources/${i}-*.md 2>/dev/null
done

# 2. For each group, read all files and merge
# 3. Write merged output
# 4. Generate summary
```

## Quality Checks

Before finalizing each merged document, verify:

- [ ] No duplicate patterns (same concept twice)
- [ ] All pattern IDs are sequential with no gaps
- [ ] All code blocks have `rust` language tag
- [ ] All cross-references are valid
- [ ] Document has summary/checklist section
- [ ] Consistent formatting (headers, spacing)

## Output Files

```
./rust-ai-guidelines/
├── README.md
├── 01-core-idioms.md
├── 02-api-design.md
├── 03-error-handling.md
├── 04-ownership-borrowing.md
├── 05-type-design.md
├── 06-traits.md
├── 07-concurrency-async.md
├── 08-performance.md
├── 09-unsafe-ffi.md
├── 10-macros.md
├── 11-anti-patterns.md
├── 12-project-structure.md
└── 13-documentation.md
```

Begin by listing all files in the source directory to understand what needs to be merged.
