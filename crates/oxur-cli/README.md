# oxur-cli

Main CLI tool for working with Oxur.

## Commands

### Compile
```bash
oxur compile input.ox -o output
```

Compile an Oxur file to a native binary.

### Run
```bash
oxur run input.ox -- arg1 arg2
```

Compile and run an Oxur file with arguments.

### REPL
```bash
oxur repl
```

Start the interactive REPL.

### New
```bash
oxur new my-project
```

Create a new Oxur project with standard structure.

### Build
```bash
oxur build
oxur build --release
```

Build the current project.

### Test
```bash
oxur test
```

Run tests in the current project.

## Future Features

- Package management integration
- Dependency resolution
- Project templates
- IDE tool support
