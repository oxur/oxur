# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2025-12-25

### Added
- Initial release
- Document listing with state filtering
- Document creation from template
- Full document addition workflow with `add` command
- State transitions with git history preservation
- YAML frontmatter management
- Automatic index generation and synchronization
- Comprehensive validation with auto-fix
- Git integration for metadata extraction
- Command aliases for common operations
- Colored output with consistent theme
- Dry-run support for previewing changes

### Features
- 10 document states (draft through superseded)
- Automatic document numbering
- Git-based author and date extraction
- Index table and state sections
- Supersedes/superseded-by tracking
- State/directory consistency checking
- Flexible state name parsing (hyphens, spaces, case-insensitive)

### Commands
- `list` (alias: `ls`) - List all documents
- `show` - Show specific document
- `new` - Create new document
- `add` - Add document with full processing
- `add-headers` (alias: `headers`) - Add/update YAML headers
- `transition` (alias: `mv`) - Change document state
- `sync-location` (alias: `sync`) - Fix directory/header mismatch
- `validate` (alias: `check`) - Validate all documents
- `update-index` (alias: `sync-index`) - Sync index with filesystem
- `index` (alias: `gen-index`) - Generate index file
