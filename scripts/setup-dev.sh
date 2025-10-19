#!/bin/bash

# Setup script for QuartzDB development environment
# Installs git hooks and checks dependencies

set -e

echo "🔧 Setting up QuartzDB development environment..."
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must be run from QuartzDB root directory"
    exit 1
fi

# Install git hooks
echo "📦 Installing git hooks..."
if [ -f "scripts/pre-push-check.sh" ]; then
    # Create pre-push hook
    cat > .git/hooks/pre-push << 'EOF'
#!/bin/bash

# Git pre-push hook
# Automatically runs validation checks before allowing push

echo "Running pre-push validation..."
echo ""

# Run the validation script
./scripts/pre-push-check.sh

# Capture exit code
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "❌ Pre-push validation failed!"
    echo ""
    echo "To skip this check (NOT RECOMMENDED), use:"
    echo "  git push --no-verify"
    echo ""
    exit 1
fi

exit 0
EOF
    
    chmod +x .git/hooks/pre-push
    echo "✅ Git pre-push hook installed"
else
    echo "⚠️  Warning: pre-push-check.sh not found"
fi

# Check dependencies
echo ""
echo "🔍 Checking dependencies..."

# Rust
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✅ Rust: $RUST_VERSION"
else
    echo "❌ Rust not installed"
    echo "   Install from: https://rustup.rs/"
fi

# Cargo
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    echo "✅ Cargo: $CARGO_VERSION"
else
    echo "❌ Cargo not installed"
fi

# Python3
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version)
    echo "✅ Python3: $PYTHON_VERSION"
else
    echo "⚠️  Python3 not installed (optional, for demos)"
fi

# cargo-audit (optional)
if command -v cargo-audit &> /dev/null; then
    echo "✅ cargo-audit installed"
else
    echo "⚠️  cargo-audit not installed (optional)"
    echo "   Install with: cargo install cargo-audit"
fi

# cargo-watch (optional)
if command -v cargo-watch &> /dev/null; then
    echo "✅ cargo-watch installed"
else
    echo "⚠️  cargo-watch not installed (optional, for auto-rebuild)"
    echo "   Install with: cargo install cargo-watch"
fi

echo ""
echo "✨ Setup complete!"
echo ""
echo "Next steps:"
echo "  1. cargo build           # Build the project"
echo "  2. cargo test            # Run tests"
echo "  3. cargo run -p quartz-server  # Start server"
echo ""
echo "Before pushing:"
echo "  ./scripts/pre-push-check.sh  # Run validation"
echo ""
