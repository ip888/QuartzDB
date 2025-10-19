# Development Guide

## Architecture Overview

QuartzDB is built on a distributed architecture with the following key components:

### Core Components

- Query Engine: Processes and optimizes queries
- Storage Engine: LSM-tree based storage with WAL
- Consensus Module: Implements Raft for distributed consensus
- Edge Manager: Handles edge node coordination

### Key Features

- Distributed storage with automatic sharding
- Edge-first architecture with local caching
- Multi-region consistency guarantees
- Real-time analytics support
- AI/ML workload optimizations

## Development Setup

1. Install Dependencies:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-edit cargo-watch cargo-audit
```

2. Setup Development Environment:

```bash
# Clone the repository
git clone https://github.com/ip888/QuartzDB.git
cd QuartzDB

# Install git hooks and check dependencies
./scripts/setup-dev.sh
```

3. Build the Project:

```bash
cargo build
```

4. Run Tests:

```bash
cargo test
```

## Pre-Push Validation

Before pushing code, **always** run the validation script:

```bash
./scripts/pre-push-check.sh
```

This automatically runs when you push (via git hook) and validates:

- ✅ **Code Formatting** - Ensures code follows Rust style guide
- ✅ **Clippy Lints** - Catches common mistakes and anti-patterns  
- ✅ **Build** - Confirms project compiles (debug + release)
- ✅ **Tests** - Runs all unit and integration tests
- ✅ **Documentation** - Validates rustdoc builds
- ✅ **Deployment** - Checks Docker and CI configurations
- ✅ **Security** - Scans for hardcoded secrets and vulnerabilities

**Skip the hook** (not recommended):

```bash
git push --no-verify
```

## Code Style

- Follow Rust standard formatting (rustfmt)
- Use meaningful variable names
- Document public APIs
- Write unit tests for new features

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## Testing Strategy

- Unit tests for individual components
- Integration tests for system behavior
- Performance benchmarks
- Distributed system tests
