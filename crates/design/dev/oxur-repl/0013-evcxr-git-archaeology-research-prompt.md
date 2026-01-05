# evcxr Architecture Archaeology - Research Prompt

## Mission

Conduct a deep investigation into evcxr's design decisions, implementation history, and evolution. We're building a Rust REPL (Oxur) and want to understand:

1. **Why** evcxr made certain architectural choices
2. **When** they encountered unexpected challenges
3. **How** they adapted their approach over time
4. **What** they might do differently in hindsight

This is architectural archaeology - we want to learn from their journey!

---

## Key Questions to Investigate

### 1. Subprocess vs In-Process Execution

**Primary Question:** Why did evcxr choose subprocess execution instead of in-process?

**What to look for:**
- Initial commit introducing subprocess model
- Any discussion/issues about execution safety
- Were crashes or hangs mentioned as motivation?
- Did they try in-process first and switch?
- Any regrets or "we should have..." comments?
- Performance trade-offs discussed?

**Search terms:** "subprocess", "isolation", "crash", "panic", "in-process", "execution model"

### 2. IPC Mechanism Choice

**Primary Question:** Why stdin/stdout instead of sockets, pipes, or other IPC?

**What to look for:**
- Discussion of communication protocols
- Why text-based (EVCXR_EXECUTION_COMPLETE) vs binary?
- Were other IPC mechanisms considered?
- Any issues with stdout mixing or protocol fragility?
- Plans to change IPC in future?

**Search terms:** "stdin", "stdout", "protocol", "IPC", "communication", "pipe"

### 3. Compilation Strategy Evolution

**Primary Question:** How did their compilation approach evolve?

**What to look for:**
- Initial implementation (full recompile every time?)
- When was incremental compilation added?
- Cache strategy changes
- Performance optimizations over time
- Cargo integration decisions
- Any rustc API usage (vs shelling out to cargo)?

**Search terms:** "incremental", "cache", "compilation", "rustc", "cargo", "performance"

### 4. Variable Persistence

**Primary Question:** How do they handle variable persistence across evaluations?

**What to look for:**
- Initial approach to variable storage
- Type erasure strategy (Box<dyn Any>)
- Serialization attempts or discussions
- Memory management challenges
- Changes to the variable store over time

**Search terms:** "variable", "persistence", "storage", "state", "Any", "serialization"

### 5. Source Mapping / Error Translation

**Primary Question:** How do they translate rustc errors back to user code?

**What to look for:**
- When was error translation added?
- How do they track source positions?
- Any source mapping strategy discussions?
- Challenges with multi-stage compilation
- User complaints about error messages

**Search terms:** "error", "source map", "position", "translation", "line number", "diagnostic"

### 6. Challenges and Surprises

**Primary Question:** What unexpected problems did they encounter?

**What to look for:**
- Issues labeled as bugs or unexpected behavior
- Workarounds for Rust/cargo limitations
- Platform-specific problems (Windows, macOS, Linux)
- Breaking changes in Rust/cargo they had to adapt to
- Performance regressions and fixes

**Search terms:** "unexpected", "workaround", "hack", "limitation", "broke", "regression"

### 7. Jupyter Integration

**Primary Question:** How did Jupyter requirements influence architecture?

**What to look for:**
- Was Jupyter support from the start or added later?
- Kernel protocol requirements
- Long-running process requirements
- State management for notebooks
- Any Jupyter-specific compromises

**Search terms:** "jupyter", "notebook", "kernel", "zmq", "ipython"

### 8. Alternative Approaches Considered

**Primary Question:** What other designs did they consider and reject?

**What to look for:**
- RFCs or design documents
- Comparison to other Rust REPLs (rusti, papyrus)
- Discussion of trade-offs
- Rejected features or approaches
- "We decided not to..." comments

**Search terms:** "alternative", "considered", "decided against", "trade-off", "design"

---

## Sources to Search

### Primary Sources (Must Check)

1. **GitHub Repository**
   - URL: https://github.com/evcxr/evcxr
   - Focus on: Commit messages, PR descriptions, issue discussions
   - Time range: All history (2018-present)

2. **Issue Tracker**
   - Especially issues with labels: architecture, design, performance, bug
   - Look for discussions with multiple participants
   - Check closed issues (resolved problems tell a story!)

3. **Pull Requests**
   - Major refactors or rewrites
   - Performance improvements
   - Architecture changes
   - Controversial/heavily discussed PRs

4. **Documentation & README**
   - Historical changes to docs (via git blame/log)
   - Architecture explanations
   - Known limitations section

### Secondary Sources (If Available)

5. **Blog Posts / Articles**
   - Author blog posts about evcxr
   - Technical write-ups
   - Conference talks or slides

6. **Reddit / HN Discussions**
   - Search "evcxr" on r/rust, Hacker News
   - User feedback and developer responses

7. **Rust Forums / Discord**
   - Any discussions about REPL design
   - Author participation in design discussions

---

## Research Method

### Step 1: Timeline Construction

Build a timeline of major architectural decisions:

```
2018-XX: Initial commit - what was the architecture?
2018-XX: Subprocess model introduced (or was it initial?)
2019-XX: Incremental compilation added
2020-XX: [Major change discovered]
...
```

### Step 2: Decision Point Analysis

For each major decision, document:

**Decision:** [What they decided]  
**Context:** [What problem were they solving?]  
**Alternatives:** [What else did they consider?]  
**Rationale:** [Why this choice?]  
**Outcome:** [Did it work? Any regrets?]  
**Lessons:** [What can we learn?]

### Step 3: Challenge Catalog

Create a list of unexpected challenges:

**Challenge:** [What went wrong?]  
**When:** [What commit/issue?]  
**Root Cause:** [Why did this happen?]  
**Solution:** [How did they fix it?]  
**Takeaway:** [How can we avoid this?]

### Step 4: Pattern Recognition

Look for recurring themes:
- "We had to..." (forced compromises)
- "Unfortunately..." (limitations accepted)
- "In hindsight..." (lessons learned)
- "This is a hack..." (technical debt)

---

## Key Commits to Find

Help me locate these important commits:

1. **Initial architecture commit** - What was the first working version?
2. **Subprocess introduction** - When and why?
3. **Incremental compilation** - Performance game-changer
4. **Variable store refactor** - Type erasure approach
5. **Error message improvements** - Better user experience
6. **Major breaking changes** - Architectural pivots
7. **Performance optimizations** - What was slow?

---

## Specific Files of Interest

Within the evcxr repository, pay special attention to:

- `evcxr/src/runtime.rs` - Variable store, execution
- `evcxr/src/eval_context.rs` - Compilation coordination  
- `evcxr/src/child_process.rs` - Subprocess management
- `evcxr_repl/src/main.rs` - REPL implementation
- `CHANGELOG.md` - Feature additions and fixes
- `README.md` history - How explanations evolved

---

## Output Format

Please provide your findings as:

### 1. Executive Summary
- 3-5 key insights we should apply to Oxur
- Biggest surprises or lessons learned

### 2. Architectural Decisions
- Timeline of major decisions
- Rationale and outcomes for each

### 3. Challenges Encountered
- Technical problems they faced
- How they solved them
- What we can learn

### 4. Design Patterns
- What worked well
- What they'd change in hindsight
- Recommended approaches

### 5. Specific Recommendations for Oxur
- Should we use subprocess? Why/why not?
- IPC mechanism suggestions
- Compilation strategy advice
- Variable persistence approach
- Source mapping strategy

### 6. Open Questions
- Things we still need to research
- Ambiguities in their approach
- Areas where we need to make our own decisions

---

## Context: Why This Matters

We're designing Oxur's REPL architecture and have tentatively decided:

- **Server-side compilation** (all stages on server)
- **Subprocess execution** (but questioning if needed)
- **Unix sockets + protocol reuse** (instead of stdin/stdout)
- **oxur-smap** (dedicated source mapping crate)
- **Multiple Executor trait** (support both in-process and subprocess)

We want to validate these decisions against evcxr's real-world experience.

**Critical Questions:**
1. Is subprocess isolation actually necessary for a development REPL?
2. What's the performance cost of subprocess vs in-process?
3. How important is crash recovery in practice?
4. What problems did they face that we haven't anticipated?

---

## Constraints

- Focus on **architectural decisions** not implementation details
- Prioritize **lessons learned** over feature lists
- Look for **design evolution** not just current state
- Capture **context** around decisions (why, not just what)
- Note **regrets or "would do differently"** comments

---

## Success Criteria

This research is successful if we can answer:

✅ Why did evcxr choose their architecture?  
✅ What problems forced them to adapt?  
✅ What would they do differently?  
✅ Should we follow their approach or diverge?  
✅ What can we learn to avoid similar challenges?

---

## Timeline

This is a deep research task - take your time! We're looking for quality insights, not quick answers. Expect this to take 30-60 minutes of thorough investigation.

---

## Thank You!

This research will help us make informed decisions about Oxur's architecture. Your archaeological work will save us from repeating mistakes and help us build on evcxr's hard-won lessons! 🔍🏛️
