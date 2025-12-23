#!/bin/bash
# Code coverage script for local development
# Generates HTML report and opens it in browser

set -e

echo "🔍 Running tests with coverage instrumentation..."
cargo llvm-cov --all-features --workspace --html

echo ""
echo "✅ Coverage report generated!"
echo "📊 Opening report in browser..."
echo ""
echo "   Report location: target/llvm-cov/html/index.html"
echo ""

# Try to open in browser (works on most systems)
if command -v xdg-open &> /dev/null; then
    xdg-open target/llvm-cov/html/index.html
elif command -v open &> /dev/null; then
    open target/llvm-cov/html/index.html
elif command -v wslview &> /dev/null; then
    wslview target/llvm-cov/html/index.html
else
    echo "ℹ️  Open target/llvm-cov/html/index.html manually to view the report"
fi
