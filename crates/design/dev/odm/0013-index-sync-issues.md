# Index Sync Issues Found During Testing

**Date:** 2025-12-26
**Context:** Testing the fix for automatic index updates after transition command

## Issues Discovered

### 1. Duplicate Document Entries

**Description:** Document 0001 exists in multiple locations:
- `crates/design/docs/05-active/0001-oxur-letter-of-intent.md` (frontmatter: `state: Active`)
- `crates/design/docs/.dustbin/overwritten/0001-oxur-letter-of-intent-655f4b6e.md` (old copy)

**Impact:**
- Index.md shows conflicting information
- Document appears in multiple state sections simultaneously
- Unclear which is the "canonical" version

**Example from index.md:**
```markdown
### Overwritten
- [0001 - Oxur: A Letter of Intent](.dustbin/overwritten/0001-oxur-letter-of-intent-655f4b6e.md)

### Draft
- [0001 - Oxur: A Letter of Intent](01-draft/0001-oxur-letter-of-intent.md)
```

### 2. Index Table Shows Wrong State

**Description:** The "All Documents by Number" table shows document 0001 with state "Overwritten":
```markdown
| Number | Title | State | Updated |
|--------|-------|-------|----------|
| 0001 | Oxur: A Letter of Intent | Overwritten | 2025-12-26 |
```

But the actual document on disk is in `05-active/` with `state: Active` in frontmatter.

**Root Cause:** Likely reading state from the dustbin copy instead of the active copy.

### 3. Missing Active Section in Index

**Description:** After transitioning document to Active state, the index update ran successfully:
```
✓ Transitioned 0001-oxur-letter-of-intent.md from Draft
  to Active

Synchronizing index with documents...
Changes:
  ✓ Remove from Draft: 01-draft/0001-oxur-letter-of-intent.md
Summary: 2 change(s) applied to index
```

But the index.md file has no "### Active" section for the document.

**Expected:** Should have:
```markdown
### Active
- [0001 - Oxur: A Letter of Intent](05-active/0001-oxur-letter-of-intent.md)
```

**Actual:** The Active section doesn't exist in the index.

### 4. Document 0002 Missing from Index Table

**Description:** Document 0002 appears in the "Draft" section but not in the "All Documents by Number" table.

**Index shows:**
```markdown
| Number | Title | State | Updated |
|--------|-------|-------|----------|
| 0001 | Oxur: A Letter of Intent | Overwritten | 2025-12-26 |
```

But Draft section has:
```markdown
### Draft
- [0001 - Oxur: A Letter of Intent](01-draft/0001-oxur-letter-of-intent.md)
- [0002 - Test Document](01-draft/0002-test-document.md)
```

Document 0002 is missing from the table entirely.

### 5. Duplicate Frontmatter in Document

**Description:** The active document has duplicate/corrupted frontmatter:
```markdown
---
number: 1
title: "Oxur: A Letter of Intent"
author: "Duncan McGreggor"
created: 2025-12-25
updated: 2025-12-26
state: Active
supersedes: null
superseded-by: null
---


---
number: 0001
title: "Oxur: A Letter of Intent"
...
```

There are TWO frontmatter blocks (lines 1-9 and starting at line 13).

## Recommendations

### Immediate Actions

1. **Clean up duplicate documents:**
   - Decide canonical location for each document
   - Remove or properly archive duplicates
   - Update state manager to track moves correctly

2. **Fix index update logic:**
   - Ensure it scans all state directories (including Active)
   - Prioritize "live" documents over dustbin copies
   - Handle missing sections properly (create Active section if needed)

3. **Fix frontmatter duplication:**
   - Investigate why transition creates duplicate frontmatter
   - Add validation to prevent/detect duplicate headers

### Long-term Improvements

1. **State consistency validation:**
   - Add `oxd validate` check for documents in multiple locations
   - Warn about state mismatches between frontmatter and directory location
   - Detect and report duplicate frontmatter

2. **Index generation robustness:**
   - Always create sections for all possible states (even if empty)
   - Add clear precedence rules when multiple copies exist
   - Log warnings when skipping/ignoring documents

3. **Testing:**
   - Add integration test for full transition cycle
   - Test index updates after each state transition
   - Verify no duplicates are created

## Related Files

- `crates/design/src/commands/transition.rs` - Transition command (now updates index)
- `crates/design/src/commands/update_index.rs` - Index update logic
- `crates/design/src/index_sync.rs` - Index synchronization implementation
- `crates/design/docs/index.md` - The index file being updated

## Test Commands

```bash
# Check current state
./bin/oxd list --state all

# Validate documents
./bin/oxd validate

# Manually update index
./bin/oxd update-index

# Check for duplicates
find crates/design/docs -name "0001-*"
```
