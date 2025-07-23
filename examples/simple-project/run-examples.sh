#!/bin/bash

# 🎯 Simple Example Runner for Beginners
# This script demonstrates basic code-insight usage

echo "🚀 Welcome to code-insight examples!"
echo "===================================="
echo

# Make sure we're in the right directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Check if code-insight is available
echo "📋 Checking if code-insight is available..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first:"
    echo "   https://rustup.rs/"
    exit 1
fi

echo "✅ Cargo found! Building code-insight..."

# Build the tool (only needed once)
cd ../..
cargo build --quiet
cd examples/simple-project

echo
echo "📁 Our simple project structure:"
echo "├── pom.xml                    (Maven configuration)"
echo "└── src/main/java/com/example/"
echo "    ├── User.java              (Entity class)"
echo "    ├── UserRepository.java    (Interface for database)"
echo "    └── UserService.java       (Business logic)"
echo

# Example 1: Parse the Maven project
echo "🔍 Example 1: Parse Maven project"
echo "--------------------------------"
cargo run --manifest-path ../../Cargo.toml -- parse --verbose --project-root .
echo

# Example 2: Build search index
echo "📚 Example 2: Build search index"
echo "--------------------------------"
cargo run --manifest-path ../../Cargo.toml -- index --project-root . --index-path ./index --force
echo

# Example 3: Search for classes
echo "🔎 Example 3: Search for classes"
echo "--------------------------------"
echo "Searching for classes..."
cargo run --manifest-path ../../Cargo.toml -- search --query User --kind exact --index-path ./index
echo

# Example 4: Search for @Service classes
echo "🔍 Example 4: Search for @Service classes"
echo "-----------------------------------------"
echo "Searching for classes with @Service annotation..."
cargo run --manifest-path ../../Cargo.toml -- search --query "" --kind exact --filter-kind class --index-path ./index
echo

# Example 5: Create dependency graph
echo "📊 Example 5: Create dependency graph"
echo "-------------------------------------"
echo "Creating Mermaid graph..."
cargo run --manifest-path ../../Cargo.toml -- graph --project-root . --index-path ./index --output deps.mmd --format mermaid
echo
echo "Graph saved to deps.mmd!"
echo "You can view it at: https://mermaid.live"
echo

# Example 6: Export for AI tools
echo "🤖 Example 6: Export for AI tools"
echo "--------------------------------"
echo "Exporting all classes as JSON..."
cargo run --manifest-path ../../Cargo.toml -- export --project-root . --index-path ./index --output export.json --format json --kind class
echo
echo "Export saved to export.json!"
echo

echo "🎉 All examples completed!"
echo
echo "📋 What you learned:"
echo "1. How to parse a Maven project"
echo "2. How to build a searchable index"
echo "3. How to search for specific code"
echo "4. How to create dependency graphs"
echo "5. How to export data for AI tools"
echo
echo "🚀 Next steps:"
echo "- Try running these commands on your own Java project!"
echo "- Check out the other examples in ../step-by-step/"
echo "- Experiment with different search queries and filters" 

# Clean up
echo
echo "🧹 Cleaning up..."
rm -rf index export.json deps.mmd 2>/dev/null || true
echo "✅ Done!" 

chmod +x "$SCRIPT_DIR/run-examples.sh" 2>/dev/null || true
echo "💡 To run this again: ./run-examples.sh" 

exit 0