#!/bin/bash
set -e

echo "=== QuartzDB Development Environment Setup ==="

# Display versions
echo ""
echo "📦 Installed Versions:"
echo "  Rust:    $(rustc --version)"
echo "  Cargo:   $(cargo --version)"
echo "  Python:  $(python3 --version)"
echo "  Node.js: $(node --version)"
echo "  npm:     $(npm --version)"
echo "  Docker:  $(docker --version)"
echo "  Git:     $(git --version)"
echo ""

# Install Python packages
echo "🐍 Installing Python packages..."
pip3 install --user --no-cache-dir \
    requests \
    numpy \
    sentence-transformers \
    black \
    pylint \
    pytest

# Install global Node.js packages
echo "📦 Installing Node.js packages..."
npm install -g \
    typescript \
    @types/node \
    prettier \
    eslint

# Verify Rust components
echo "🦀 Verifying Rust components..."
rustup component list | grep -E "(rustfmt|clippy)" | grep installed

echo ""
echo "✅ Development environment setup complete!"
echo "🚀 Ready to build QuartzDB!"
