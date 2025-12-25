# Oxur Design Documents

This directory contains all design documents, architectural decisions, and technical specifications for the Oxur project.

## Document States

- **Draft** - Work in progress, open for discussion
- **Under Review** - Complete but awaiting approval
- **Final** - Approved and implemented
- **Superseded** - Replaced by a newer document

## Document Index

| Number | Title | State | Updated |
|--------|-------|-------|---------|
| 0001 | Oxur: A Letter of Intent | Draft | 2025-12-25 |

## Adding New Documents

Use the design CLI tool:

```bash
cargo run -p design -- new "Your Document Title"
```

This will create a new document in `01-drafts/` with the next available number.

## Document Template

All design documents follow a standard template with YAML frontmatter. See `templates/design-doc-template.md`.
