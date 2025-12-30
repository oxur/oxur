# Changelog

All notable changes to oxur-cli will be documented in this file.

## [0.1.0] - 2025-12-30

### Added

**Library Features:**
- Common I/O utilities module (`common::io`)
  - `read_input()` - Read from stdin or file
  - `write_output()` - Write to stdout or file
  - `write_stderr()` - Write to stderr
- Colored output module (`common::output`)
  - `success()`, `error()`, `warning()`, `info()`
  - `error_with_context()` - Error with helpful context
  - `step()` and `step_done()` - Numbered step indicators
- Progress tracking module (`common::progress`)
  - `ProgressTracker` - Multi-step operation tracking with verbose mode
- Comprehensive unit tests for all modules
- Usage documentation and examples

**Binary Features:**
- Unified `oxur` command-line tool with compile, run, repl, new, build, and test commands
- Uses library utilities for consistent output and error handling

### Migration Notes

- `aster` CLI now uses `oxur-cli` for I/O, output, and progress tracking
- `oxd` CLI partially migrated (using output utilities for info/warning messages)

## [Unreleased]

### Planned

- Full `oxd` integration with progress tracking
- Configuration file loading utilities
- Interactive prompt helpers
