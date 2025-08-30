#!/bin/bash
# Format all Rust code and check for issues

echo "🔧 Formatting Rust code..."
cargo fmt --all

echo "🔍 Checking for any remaining formatting issues..."
if cargo fmt --all -- --check; then
    echo "✅ All code is properly formatted!"
else
    echo "❌ Some formatting issues remain"
    exit 1
fi

echo "🔍 Running clippy for additional checks..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "✅ Clippy checks passed!"
else
    echo "⚠️  Clippy found some issues"
fi