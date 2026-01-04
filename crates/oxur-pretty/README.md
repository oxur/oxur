# oxur-pretty

Pretty-printer for S-expression formatted data with rustfmt-style CLI.

## Overview

`oxur-pretty` provides tools for formatting S-expressions in a human-readable way, with support for various formatting strategies and a command-line tool that follows rustfmt conventions.

Key features:

- **Zero dependencies on oxur-ast** - Avoids cyclic dependencies
- **Idempotent formatting** - `format(format(x)) === format(x)`
- **Multiple strategies** - Automatic selection between inline, compact, and multiline formats
- **CLI tool** - `oxur-fmt` command-line tool matching rustfmt interface
- **Check mode** - CI/CD integration with `--check` flag
- **Configurable** - Control line width, indentation, and formatting rules
- **Atomic writes** - Safe in-place file modification with permission preservation

## CLI Tool: `oxur-fmt`

The `oxur-fmt` command-line tool formats S-expression files following rustfmt conventions.

### Installation

```bash
# From workspace root
make build

# Or directly
cargo build --release -p oxur-pretty
```

The binary will be at `./bin/oxur-fmt` (via Makefile) or `target/release/oxur-fmt`.

### Quick Start

```bash
# Format file in-place
oxur-fmt file.sexp

# Check if formatted (CI/CD)
oxur-fmt --check file.sexp

# Format to stdout
oxur-fmt --emit stdout file.sexp

# Stdin to stdout
cat file.sexp | oxur-fmt
echo "(Span :lo 0 :hi 10)" | oxur-fmt

# With custom config
oxur-fmt --config max_width=120,tab_spaces=4 file.sexp
```

### Commands and Flags

**Basic Usage:**

```bash
oxur-fmt [OPTIONS] <FILE>...
```

**Options:**

| Flag | Description |
|------|-------------|
| `--check` | Check if files are formatted (exit 1 if not) |
| `--emit <MODE>` | What data to emit: `files` (default) or `stdout` |
| `--backup` | Backup modified files (creates `.bk` files) |
| `--config <key=val,...>` | Set config from command line |
| `--color <MODE>` | Use colored output: `always`, `never`, or `auto` |
| `-l, --files-with-diff` | Print names of files needing formatting |
| `-v, --verbose` | Print verbose output |
| `-q, --quiet` | Print less output |

**Configuration Keys:**

- `max_width=<N>` - Maximum line width (default: 100)
- `tab_spaces=<N>` - Spaces per indent level (default: 2)
- `align_keywords=<bool>` - Align keyword-value pairs (default: true)
- `compact_simple_values=<bool>` - Keep simple values compact (default: true)
- `max_inline_items=<N>` - Max items for inline formatting (default: 3)

### Examples

#### Format Files In-Place

```bash
# Single file
oxur-fmt my-ast.sexp

# Multiple files
oxur-fmt file1.sexp file2.sexp file3.sexp

# With glob expansion
oxur-fmt **/*.sexp
```

#### Check Mode (CI/CD)

```bash
# Check if files are formatted
oxur-fmt --check src/*.sexp

# Exit code 0 if formatted, 1 if not, 2 on error
if oxur-fmt --check file.sexp; then
    echo "Formatted correctly"
else
    echo "Needs formatting"
fi

# List files needing formatting
oxur-fmt --check -l **/*.sexp
```

#### Stdin/Stdout Usage

```bash
# Read from stdin, write to stdout
cat input.sexp | oxur-fmt > output.sexp

# Use '-' explicitly
oxur-fmt - < input.sexp > output.sexp

# Format and pipe
echo "(Item :id 0 :name value)" | oxur-fmt | less

# Format to stdout even with file input
oxur-fmt --emit stdout my-ast.sexp > formatted.sexp
```

#### Backup Files

```bash
# Create backup before formatting
oxur-fmt --backup important.sexp

# Creates important.sexp.bk before modifying important.sexp
```

#### Custom Configuration

```bash
# Single option
oxur-fmt --config max_width=120 file.sexp

# Multiple options
oxur-fmt --config max_width=80,tab_spaces=4,align_keywords=false file.sexp

# Disable keyword alignment
oxur-fmt --config align_keywords=false file.sexp
```

#### Verbose Output

```bash
# Show what's being formatted
oxur-fmt -v file1.sexp file2.sexp

# Quiet mode (errors only)
oxur-fmt -q **/*.sexp
```

### Formatting Strategies

`oxur-fmt` automatically selects the best formatting strategy based on content:

#### Inline Format

Simple expressions that fit on one line:

```lisp
(a b c)
(PathSegment :ident foo :id 0)
```

#### Compact Struct Format

Type-like structures with keyword-value pairs that fit the max width:

```lisp
(Span :lo 0 :hi 10)
(Ident :name "use" :span (Span :lo 0 :hi 3))
```

#### Keyword-Aligned Format

Multiline with aligned keywords for readability:

```lisp
(Item
  :attrs ()
  :id 0
  :span (Span :lo 0 :hi 0)
  :vis (Inherited)
  :ident (Ident :name "main" :span (Span :lo 0 :hi 0))
  :kind (Fn ...))
```

#### Smart Multiline Format

General multiline with proper indentation:

```lisp
(Crate
  :attrs ()
  :items ((Item ...)
          (Item ...))
  :spans (ModSpans
           :inner-span (Span :lo 0 :hi 0)
           :inject-use-span (Span :lo 0 :hi 0)))
```

### Exit Codes

- **0** - Success (all files formatted correctly)
- **1** - Files need formatting (only in `--check` mode)
- **2** - Error occurred (parse error, I/O error, etc.)

## Library API

Use `oxur-pretty` as a library for programmatic formatting.

### Basic Usage

```rust
use oxur_pretty::format_sexp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = "(Span :lo 0 :hi 10)";
    let formatted = format_sexp(input)?;

    println!("{}", formatted);
    // Output: (Span :lo 0 :hi 10)

    Ok(())
}
```

### Custom Configuration

```rust
use oxur_pretty::{format_sexp_with_config, FormatConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = "(VeryLongTypeName :key1 value1 :key2 value2 :key3 value3)";

    let config = FormatConfig::default()
        .with_max_width(40)?
        .with_tab_spaces(4)?
        .with_align_keywords(true);

    let formatted = format_sexp_with_config(input, config)?;

    println!("{}", formatted);
    // Output will be multiline because it exceeds max_width

    Ok(())
}
```

### Configuration Builder

```rust
use oxur_pretty::FormatConfig;

// Default configuration
let config = FormatConfig::default();
// max_width: 100
// tab_spaces: 2
// align_keywords: true
// compact_simple_values: true
// max_inline_items: 3

// Custom configuration
let config = FormatConfig::default()
    .with_max_width(120)?
    .with_tab_spaces(4)?
    .with_align_keywords(false)
    .with_compact_simple_values(false)
    .with_max_inline_items(5);
```

### Error Handling

```rust
use oxur_pretty::{format_sexp, FormatterError};

match format_sexp("(unclosed") {
    Ok(formatted) => println!("{}", formatted),
    Err(FormatterError::UnmatchedOpen { pos }) => {
        eprintln!("Unmatched opening parenthesis at position {}", pos);
    }
    Err(FormatterError::UnterminatedString { pos }) => {
        eprintln!("Unterminated string at position {}", pos);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Working with Parser and Formatter Directly

```rust
use oxur_pretty::{parse, Formatter, FormatConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse S-expression
    let input = r#"(Ident :name "use" :span (Span :lo 0 :hi 3))"#;
    let expr = parse(input)?;

    // Format with custom config
    let config = FormatConfig::default().with_max_width(80)?;
    let formatter = Formatter::new(config);
    let output = formatter.format(&expr);

    println!("{}", output);

    Ok(())
}
```

## Integration with oxur-ast

`oxur-pretty` is designed to work seamlessly with `oxur-ast`:

```rust
use oxur_ast::sexp::print_sexp;  // Built-in S-expression printer
use oxur_pretty::format_sexp;     // oxur-pretty formatter

// oxur-ast generates S-expressions (compact, no specific formatting)
let sexp = generate_sexp_from_ast(&ast);
let compact = print_sexp(&sexp);

// oxur-pretty formats for human readability
let formatted = format_sexp(&compact)?;
```

**When to use which:**

- **`oxur-ast`** - For AST operations, round-trip preservation, programmatic S-expression generation
- **`oxur-pretty`** - For human-readable output, code formatting, documentation examples

## Testing

Run all tests:

```bash
cargo test -p oxur-pretty
```

Run specific test suites:

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test cli_integration
```

### Test Coverage

The crate includes comprehensive tests:

- **Parser tests** - All S-expression syntax (atoms, lists, strings, escapes, comments)
- **Formatter tests** - Each formatting strategy, edge cases, idempotency
- **Config tests** - Validation, builder pattern
- **Rules tests** - Heuristics, decision logic
- **CLI tests** - All flags, exit codes, file operations
- **Error tests** - All error paths

Coverage target: 95%+

## Design Documentation

See design document ODD-0037 for implementation details:

```bash
# From workspace root
./bin/oxd show 37
```

## Formatting Philosophy

`oxur-pretty` follows these principles:

1. **Idempotency** - Running the formatter multiple times produces the same result
2. **Readability** - Optimizes for human readers, not machines
3. **Consistency** - Same input always produces same output
4. **Preservation** - Maintains semantic meaning, only changes whitespace
5. **Intelligence** - Adapts formatting strategy based on content

## Performance

The formatter is designed for speed:

- **Simple caching** - Avoids redundant calculations
- **Efficient string building** - Minimizes allocations
- **Lazy evaluation** - Only calculates what's needed

Run benchmarks:

```bash
cargo bench -p oxur-pretty
```

## Examples in the Wild

```bash
# Format generated AST files
./bin/aster to-ast examples/hello.rs | oxur-fmt

# Format test fixtures
oxur-fmt test-data/**/*.sexp

# Check formatting in CI
oxur-fmt --check src/**/*.sexp || exit 1

# Format with project-specific config
oxur-fmt --config max_width=120,tab_spaces=4 src/*.sexp
```

## Comparison with Other Tools

| Feature | oxur-pretty | oxur-ast printer | rustfmt |
|---------|-------------|------------------|---------|
| **S-expressions** | ✓ Optimized | ✓ Basic | ✗ |
| **Rust code** | ✗ | ✗ | ✓ Optimized |
| **Idempotent** | ✓ | ✓ | ✓ |
| **Check mode** | ✓ | ✗ | ✓ |
| **Config file** | ✗ CLI only | ✗ | ✓ |
| **Stdin/stdout** | ✓ | ✓ | ✓ |
| **Backup files** | ✓ | ✗ | ✓ |
| **Multiple strategies** | ✓ 4 strategies | ✗ Simple | ✓ Many |

## Troubleshooting

### "Parse error: Unmatched opening parenthesis"

Check that all parentheses are balanced:

```bash
# Use verbose mode to see where parsing fails
oxur-fmt -v file.sexp
```

### "Invalid config: max_width must be greater than 0"

Ensure config values are valid:

```bash
# Wrong
oxur-fmt --config max_width=0 file.sexp

# Correct
oxur-fmt --config max_width=80 file.sexp
```

### File Not Modified

If using `--emit stdout`, output goes to stdout, not file:

```bash
# This DOES NOT modify file.sexp
oxur-fmt --emit stdout file.sexp

# This modifies file.sexp in-place
oxur-fmt file.sexp
```

### Permission Denied

Ensure write permissions:

```bash
chmod u+w file.sexp
oxur-fmt file.sexp
```

## License

See the [main repository](../../) for license information.

## Contributing

Contributions are welcome! Please see the [main repository](../../) for guidelines.
