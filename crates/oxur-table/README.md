# oxur-table

Styled table rendering for Oxur tools with the warm orange "Oxur" theme.

## Overview

`oxur-table` provides a simple, ergonomic API for creating beautifully styled terminal tables. It uses the [tabled](https://crates.io/crates/tabled) crate for core table rendering and includes an embedded default theme with warm orange sunset colors that match the Oxur brand.

## Features

- **Zero Configuration** - Works out-of-box with embedded default theme
- **Type-Safe** - Generic over any type implementing `Tabled`
- **Colored Output** - Full support for `ColoredString` in table cells
- **Hex Colors** - Theme supports both ANSI color names and hex colors (`#RRGGBB`)
- **Row Styling** - Configurable header, title, data rows, and footer
- **Flexible Theming** - TOML-based theme system

## Quick Start

```rust
use oxur_table::{OxurTable, Tabled};
use colored::*;

#[derive(Tabled)]
struct Employee {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Age")]
    age: u32,
    #[tabled(rename = "Role")]
    role: ColoredString,
}

let employees = vec![
    Employee {
        name: "Alice".into(),
        age: 30,
        role: "Engineer".green()
    },
    Employee {
        name: "Bob".into(),
        age: 25,
        role: "Designer".cyan()
    },
];

let table = OxurTable::new(employees).render();
println!("{}", table);
```

## Default Theme

The default Oxur theme features:

- **Header**: Dark orange background (#D45500) with dark brown text
- **Data Rows**: Alternating warm peach/coral text on dark brown background
- **Colors**: Warm orange sunset palette
- **Padding**: Minimal (0px) for compact display
- **Title/Footer**: Optional, styled to match theme

## Usage in oxd

This crate is currently used by the `oxd` design document management tool for displaying formatted tables of document information. See `crates/design/src/commands/list.rs` for example usage.

## API

### `OxurTable::new(data: Vec<T>) -> Self`

Creates a new table with the default Oxur theme.

### `.render() -> String`

Renders the table as a styled string for terminal output.

## Dependencies

- `tabled = "0.17"` - Core table rendering
- `serde` - TOML deserialization
- `toml` - Theme parsing
- `colored` - Color support

## License

MIT OR Apache-2.0
