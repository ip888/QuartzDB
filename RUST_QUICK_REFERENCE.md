# 🦀 Rust Development Quick Reference

## Essential VS Code Commands

### Build & Run
- `Ctrl+Shift+B` - Build project (default task)
- `F5` - Start debugging
- `Ctrl+Shift+P` → "Tasks: Run Task" - Run any task

### Code Navigation
- `F12` - Go to definition
- `Alt+F12` - Peek definition
- `Shift+F12` - Find all references
- `Ctrl+T` - Go to symbol in workspace
- `Ctrl+Shift+O` - Go to symbol in file

### Code Actions
- `Alt+Enter` or `Ctrl+.` - Quick fix / Show code actions
- `F2` - Rename symbol
- `Shift+Alt+F` - Format document
- `Ctrl+/` - Toggle line comment

### Testing
- Click "Run Test" / "Debug Test" above test functions
- `Ctrl+Shift+P` → "Test: Run All Tests"

## Common Cargo Commands

```bash
# Development
cargo check              # Fast compilation check
cargo build              # Compile debug build
cargo build --release    # Compile optimized build
cargo run                # Build and run
cargo test               # Run tests
cargo bench              # Run benchmarks

# Code Quality
cargo fmt                # Format code
cargo clippy             # Lint code
cargo fix                # Auto-fix warnings

# Dependencies
cargo add tokio          # Add dependency
cargo add serde -F derive # Add with features
cargo rm tokio           # Remove dependency
cargo update             # Update dependencies
cargo outdated           # Check for updates
cargo upgrade            # Upgrade to latest

# Documentation
cargo doc --open         # Build and open docs
rustup doc               # Open Rust docs

# Advanced
cargo expand             # Show macro expansions
cargo nextest run        # Better test runner
cargo audit              # Security audit
cargo watch -x check     # Auto-recompile on save

# Maintenance
cargo clean              # Remove build artifacts
cargo install-update -a  # Update installed tools
```

## Rust-Analyzer Features

### Inlay Hints
Shows types and parameter names inline. Toggle with `Ctrl+Shift+P` → "Toggle Inlay Hints"

### Code Lens
Click "Run" / "Debug" / "references" above functions

### Completions
- Auto-imports
- Trait completions
- Macro completions

## Debugging

### Set Breakpoints
Click in the gutter (left of line numbers) or press `F9`

### Debug Console
Execute expressions while debugging

### Variables Panel
Inspect and watch variables

### Call Stack
Navigate the execution stack

## Keyboard Shortcuts Summary

| Action | Shortcut |
|--------|----------|
| Build | `Ctrl+Shift+B` |
| Debug | `F5` |
| Format | `Shift+Alt+F` |
| Go to Definition | `F12` |
| Find References | `Shift+F12` |
| Rename | `F2` |
| Quick Fix | `Ctrl+.` |
| Command Palette | `Ctrl+Shift+P` |
| Terminal | `` Ctrl+` `` |

## Environment Variables

```bash
# Enable backtraces
export RUST_BACKTRACE=1        # Basic
export RUST_BACKTRACE=full     # Detailed

# Logging
export RUST_LOG=debug          # Debug level
export RUST_LOG=trace          # Trace level
export RUST_LOG=myapp=debug    # Per-crate

# Compilation
export RUSTFLAGS="-C target-cpu=native"  # Optimize for your CPU
```

## Project Structure Best Practices

```
project/
├── Cargo.toml           # Main manifest
├── Cargo.lock           # Dependency lock (commit this)
├── src/
│   ├── main.rs         # Binary entry point
│   ├── lib.rs          # Library entry point
│   └── bin/            # Additional binaries
├── tests/              # Integration tests
├── benches/            # Benchmarks
├── examples/           # Example code
└── .cargo/
    └── config.toml     # Project cargo config
```

## Common Patterns

### Error Handling
```rust
use anyhow::{Result, Context};

fn example() -> Result<()> {
    let file = std::fs::read_to_string("file.txt")
        .context("Failed to read file")?;
    Ok(())
}
```

### Async/Await
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let result = async_function().await?;
    Ok(())
}
```

### Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn test_async() {
        let result = async_fn().await;
        assert!(result.is_ok());
    }
}
```

## Tips & Tricks

1. **Fast Iteration**: Use `cargo watch -x check` while developing
2. **Better Errors**: Use `cargo-nextest` for cleaner test output
3. **Security**: Run `cargo audit` regularly
4. **Documentation**: Comment with `///` for doc comments
5. **Performance**: Use `cargo build --release` for production
6. **Debugging**: Enable `RUST_BACKTRACE=1` for stack traces

## Learning Resources

- **The Rust Book**: https://doc.rust-lang.org/book/
- **Rust By Example**: https://doc.rust-lang.org/rust-by-example/
- **Rustlings**: Interactive exercises
- **Crates.io**: https://crates.io/ - Find libraries
- **Docs.rs**: https://docs.rs/ - Documentation

## Getting Help

```bash
# Command help
cargo help
cargo help build

# Rust documentation
rustup doc
rustup doc --book
rustup doc --std

# In VS Code
# Hover over any item for docs
# Ctrl+Shift+P → "Rust Analyzer: Show Documentation"
```

---
**Setup Location**: `/home/igor/projects/db/QuartzDB`
**Full Documentation**: See `RUST_DEV_SETUP.md`
