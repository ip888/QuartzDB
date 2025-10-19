#!/bin/bash

# QuartzDB Pre-Push Validation Script
# This script runs all checks locally to ensure GitHub CI will pass

set -e  # Exit on first error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track overall status
FAILED_CHECKS=0

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  QuartzDB Pre-Push Validation${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

# Function to print section header
print_header() {
    echo -e "\n${BLUE}▶ $1${NC}"
    echo -e "${BLUE}────────────────────────────────────────────────────────────${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}✗ $1${NC}"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Function to print info
print_info() {
    echo -e "  $1"
}

# =============================================================================
# 1. Environment Check
# =============================================================================
print_header "1. Checking Development Environment"

# Check Rust
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    print_success "Rust installed: $RUST_VERSION"
else
    print_error "Rust not installed"
fi

# Check Cargo
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    print_success "Cargo installed: $CARGO_VERSION"
else
    print_error "Cargo not installed"
fi

# Check Python3
if command -v python3 &> /dev/null; then
    PYTHON_VERSION=$(python3 --version)
    print_success "Python3 installed: $PYTHON_VERSION"
else
    print_warning "Python3 not installed (needed for demo tests)"
fi

# Check Git
if command -v git &> /dev/null; then
    GIT_VERSION=$(git --version)
    print_success "Git installed: $GIT_VERSION"
else
    print_error "Git not installed"
fi

# =============================================================================
# 2. Git Status Check
# =============================================================================
print_header "2. Checking Git Status"

# Check for uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
    print_warning "Uncommitted changes detected:"
    git status --short
    echo ""
    read -p "Continue anyway? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}Aborted by user${NC}"
        exit 1
    fi
else
    print_success "Working directory clean"
fi

# Check current branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
print_info "Current branch: $CURRENT_BRANCH"

# =============================================================================
# 3. Rust Format Check
# =============================================================================
print_header "3. Checking Rust Code Formatting"

if cargo fmt -- --check &> /dev/null; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting issues detected"
    print_info "Run: cargo fmt"
fi

# =============================================================================
# 4. Clippy Lints
# =============================================================================
print_header "4. Running Clippy Lints"

if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/clippy.log; then
    print_success "Clippy checks passed"
else
    print_error "Clippy found issues"
    print_info "Check /tmp/clippy.log for details"
fi

# =============================================================================
# 5. Build Check
# =============================================================================
print_header "5. Building Project (Debug)"

if cargo build 2>&1 | tee /tmp/build-debug.log; then
    print_success "Debug build successful"
else
    print_error "Debug build failed"
    print_info "Check /tmp/build-debug.log for details"
fi

print_header "5b. Building Project (Release)"

if cargo build --release 2>&1 | tee /tmp/build-release.log; then
    print_success "Release build successful"
else
    print_error "Release build failed"
    print_info "Check /tmp/build-release.log for details"
fi

# =============================================================================
# 6. Run Tests
# =============================================================================
print_header "6. Running Unit and Integration Tests"

if cargo test --all 2>&1 | tee /tmp/test.log; then
    print_success "All tests passed"
else
    print_error "Tests failed"
    print_info "Check /tmp/test.log for details"
fi

# =============================================================================
# 7. Check Documentation
# =============================================================================
print_header "7. Building Documentation"

if cargo doc --no-deps --all-features 2>&1 | tee /tmp/doc.log; then
    print_success "Documentation built successfully"
else
    print_error "Documentation build failed"
    print_info "Check /tmp/doc.log for details"
fi

# =============================================================================
# 8. Check Cargo.toml Files
# =============================================================================
print_header "8. Validating Cargo.toml Files"

CARGO_TOMLS=$(find . -name "Cargo.toml" -not -path "./target/*")
for toml in $CARGO_TOMLS; do
    if cargo metadata --manifest-path "$toml" &> /dev/null; then
        print_success "Valid: $toml"
    else
        print_error "Invalid: $toml"
    fi
done

# =============================================================================
# 9. Check for Common Issues
# =============================================================================
print_header "9. Checking for Common Issues"

# Check for println! in production code (should use logging)
PRINTLN_COUNT=$(grep -r "println!" --include="*.rs" quartz-*/src/ 2>/dev/null | grep -v "test" | wc -l | tr -d ' ')
if [ "$PRINTLN_COUNT" -gt 0 ]; then
    print_warning "Found $PRINTLN_COUNT println! statements in production code"
    print_info "Consider using tracing/log macros instead"
else
    print_success "No println! in production code"
fi

# Check for unwrap() in production code
UNWRAP_COUNT=$(grep -r "\.unwrap()" --include="*.rs" quartz-*/src/ 2>/dev/null | grep -v "test" | wc -l | tr -d ' ')
if [ "$UNWRAP_COUNT" -gt 0 ]; then
    print_warning "Found $UNWRAP_COUNT .unwrap() calls in production code"
    print_info "Consider proper error handling"
else
    print_success "No unwrap() in production code"
fi

# Check for TODO comments
TODO_COUNT=$(grep -r "TODO\|FIXME\|XXX\|HACK" --include="*.rs" quartz-*/src/ 2>/dev/null | wc -l | tr -d ' ')
if [ "$TODO_COUNT" -gt 0 ]; then
    print_warning "Found $TODO_COUNT TODO/FIXME comments"
    print_info "Consider addressing before release"
else
    print_success "No TODO/FIXME comments"
fi

# =============================================================================
# 10. Check Deployment Files
# =============================================================================
print_header "10. Validating Deployment Configuration"

# Check Dockerfile exists
if [ -f "Dockerfile" ]; then
    print_success "Dockerfile exists"
    
    # Validate Dockerfile syntax (basic check)
    if grep -q "FROM" Dockerfile && grep -q "COPY" Dockerfile; then
        print_success "Dockerfile appears valid"
    else
        print_error "Dockerfile might be malformed"
    fi
else
    print_error "Dockerfile not found"
fi

# Check docker-compose.yml
if [ -f "docker-compose.yml" ]; then
    print_success "docker-compose.yml exists"
else
    print_warning "docker-compose.yml not found"
fi

# Check .dockerignore
if [ -f ".dockerignore" ]; then
    print_success ".dockerignore exists"
else
    print_warning ".dockerignore not found"
fi

# Check GitHub Actions workflows
if [ -d ".github/workflows" ]; then
    WORKFLOW_COUNT=$(find .github/workflows -name "*.yml" | wc -l | tr -d ' ')
    print_success "Found $WORKFLOW_COUNT GitHub Actions workflow(s)"
    
    # List workflows
    for workflow in .github/workflows/*.yml; do
        print_info "  - $(basename $workflow)"
    done
else
    print_error ".github/workflows directory not found"
fi

# Check DigitalOcean config
if [ -f ".do/app.yaml" ]; then
    print_success "DigitalOcean app.yaml exists"
    
    # Check if placeholder username is still there
    if grep -q "YOUR_USERNAME" .do/app.yaml; then
        print_warning "Replace YOUR_USERNAME in .do/app.yaml"
    fi
else
    print_warning ".do/app.yaml not found"
fi

# Check deploy script
if [ -f "scripts/deploy-do.sh" ]; then
    print_success "Deployment script exists"
    
    if [ -x "scripts/deploy-do.sh" ]; then
        print_success "Deployment script is executable"
    else
        print_warning "Deployment script is not executable"
        print_info "Run: chmod +x scripts/deploy-do.sh"
    fi
else
    print_error "Deployment script not found"
fi

# =============================================================================
# 11. Check Documentation Files
# =============================================================================
print_header "11. Checking Documentation"

REQUIRED_DOCS=(
    "README.md"
    "CONTRIBUTING.md"
    "LICENSE"
    "DEPLOYMENT.md"
    "TECHNICAL_OVERVIEW.md"
)

for doc in "${REQUIRED_DOCS[@]}"; do
    if [ -f "$doc" ]; then
        print_success "$doc exists"
    else
        print_error "$doc not found"
    fi
done

# =============================================================================
# 12. Check Example Files
# =============================================================================
print_header "12. Checking Examples"

if [ -d "quartz-server/examples" ]; then
    EXAMPLE_COUNT=$(find quartz-server/examples -name "*.py" -o -name "*.rs" | wc -l | tr -d ' ')
    print_success "Found $EXAMPLE_COUNT example file(s)"
else
    print_warning "No examples directory found"
fi

# =============================================================================
# 13. Security Checks
# =============================================================================
print_header "13. Security Checks"

# Check for secrets in code
if grep -r -i "password\s*=\s*[\"']" --include="*.rs" --include="*.toml" quartz-*/ 2>/dev/null; then
    print_error "Potential hardcoded password found"
else
    print_success "No hardcoded passwords detected"
fi

if grep -r -i "api_key\s*=\s*[\"']" --include="*.rs" --include="*.toml" quartz-*/ 2>/dev/null; then
    print_error "Potential hardcoded API key found"
else
    print_success "No hardcoded API keys detected"
fi

# Check dependencies for known vulnerabilities (requires cargo-audit)
if command -v cargo-audit &> /dev/null; then
    print_info "Running cargo audit..."
    if cargo audit 2>&1 | tee /tmp/audit.log; then
        print_success "No known vulnerabilities in dependencies"
    else
        print_warning "Vulnerabilities found (check /tmp/audit.log)"
    fi
else
    print_warning "cargo-audit not installed (run: cargo install cargo-audit)"
fi

# =============================================================================
# 14. File Size Checks
# =============================================================================
print_header "14. Checking File Sizes"

# Check for large files that shouldn't be committed
LARGE_FILES=$(find . -type f -size +1M -not -path "./target/*" -not -path "./.git/*" -not -path "./data/*" 2>/dev/null)

if [ -n "$LARGE_FILES" ]; then
    print_warning "Large files detected (>1MB):"
    echo "$LARGE_FILES" | while read file; do
        SIZE=$(du -h "$file" | cut -f1)
        print_info "  $file ($SIZE)"
    done
else
    print_success "No unexpectedly large files"
fi

# =============================================================================
# 15. CI Simulation
# =============================================================================
print_header "15. Simulating GitHub Actions CI"

print_info "This simulates what GitHub Actions will run..."

# Simulate the CI workflow steps
print_info "[1/4] Format check..."
if cargo fmt -- --check &> /dev/null; then
    print_success "  Format check passed"
else
    print_error "  Format check failed (GitHub CI will fail)"
fi

print_info "[2/4] Clippy..."
if cargo clippy --all-targets --all-features -- -D warnings &> /dev/null; then
    print_success "  Clippy passed"
else
    print_error "  Clippy failed (GitHub CI will fail)"
fi

print_info "[3/4] Build..."
if cargo build &> /dev/null; then
    print_success "  Build passed"
else
    print_error "  Build failed (GitHub CI will fail)"
fi

print_info "[4/4] Tests..."
if cargo test --all &> /dev/null; then
    print_success "  Tests passed"
else
    print_error "  Tests failed (GitHub CI will fail)"
fi

# =============================================================================
# Summary
# =============================================================================
echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Validation Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed! Safe to push.${NC}\n"
    echo -e "${GREEN}You can now run:${NC}"
    echo -e "${GREEN}  git push${NC}\n"
    exit 0
else
    echo -e "${RED}✗ $FAILED_CHECKS check(s) failed!${NC}\n"
    echo -e "${RED}Please fix the issues before pushing.${NC}\n"
    echo -e "${YELLOW}Logs saved to /tmp/*.log${NC}\n"
    exit 1
fi
