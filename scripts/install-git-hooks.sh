#!/usr/bin/env bash
# Install git hooks for the http-client project

set -e

HOOK_DIR=".git/hooks"
HOOK_FILE="$HOOK_DIR/pre-commit"

echo "Installing git hooks..."

# Create hooks directory if it doesn't exist
mkdir -p "$HOOK_DIR"

# Create pre-commit hook
cat > "$HOOK_FILE" << 'EOF'
#!/usr/bin/env bash
# Pre-commit hook for http-client project
# Runs cargo fmt and clippy before allowing commit

set -e

echo "Running pre-commit checks..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust."
    exit 1
fi

# Run cargo fmt
echo "→ Running cargo fmt..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting check failed!"
    echo "   Run 'cargo fmt --all' to fix formatting issues."
    exit 1
fi

# Run clippy
echo "→ Running cargo clippy..."
if ! cargo clippy --workspace --all-targets -- -D warnings; then
    echo "❌ Clippy check failed!"
    echo "   Fix the warnings above before committing."
    exit 1
fi

echo "✅ All pre-commit checks passed!"
EOF

# Make the hook executable
chmod +x "$HOOK_FILE"

echo "✅ Git hooks installed successfully!"
echo ""
echo "The pre-commit hook will now run 'cargo fmt' and 'cargo clippy' before each commit."
echo "To bypass the hook (not recommended), use: git commit --no-verify"
