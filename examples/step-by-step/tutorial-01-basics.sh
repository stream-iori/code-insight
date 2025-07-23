#!/bin/bash

# 🎯 Tutorial 1: Basics for Absolute Beginners
# This is your first step into code analysis!

echo "🎓 Tutorial 1: Getting Started with Code Analysis"
echo "================================================="
echo

# Welcome message
echo "👋 Hello! Let's learn about code analysis together."
echo
echo "📚 What we'll learn:"
echo "1. What a Maven project looks like"
echo "2. How to find classes in your code"
echo "3. How to see dependencies"
echo "4. How to search for specific things"
echo

# Navigate to correct directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Build the tool
echo "🔨 Building code-insight..."
cd ../..
cargo build --quiet
cd examples/step-by-step

echo "✅ Tool built successfully!"
echo

# Create a tiny project to practice on
echo "📁 Creating practice project..."
mkdir -p practice-project/src/main/java/com/tutorial

cat > practice-project/pom.xml << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.tutorial</groupId>
    <artifactId>my-first-app</artifactId>
    <version>1.0.0</version>
    
    <properties>
        <maven.compiler.source>11</maven.compiler.source>
        <maven.compiler.target>11</maven.compiler.target>
    </properties>
</project>
EOF

cat > practice-project/src/main/java/com/tutorial/HelloWorld.java << 'EOF'
package com.tutorial;

/**
 * My first Java class!
 * This is what a simple class looks like
 */
public class HelloWorld {
    private String message = "Hello, World!";
    
    public String getMessage() {
        return message;
    }
}
EOF

cat > practice-project/src/main/java/com/tutorial/Calculator.java << 'EOF'
package com.tutorial;

/**
 * A simple calculator class
 * Shows basic methods and fields
 */
public class Calculator {
    private int result = 0;
    
    public int add(int a, int b) {
        return a + b;
    }
    
    public int multiply(int a, int b) {
        return a * b;
    }
}
EOF

echo "✅ Practice project created!"
echo
echo "📁 Project structure:"
echo "practice-project/"
echo "├── pom.xml                    (Maven config)"
echo "└── src/main/java/com/tutorial/"
echo "    ├── HelloWorld.java"
echo "    └── Calculator.java"
echo

# Step 1: Parse the project
echo "🔍 Step 1: Parse the project"
echo "-----------------------------"
echo "Let's see what code-insight finds in our project..."
echo
cargo run --manifest-path ../../Cargo.toml -- parse --project-root practice-project --verbose
echo
echo "💡 What you just saw:"
echo "- The parser found our project info"
echo "- It counted our Java files"
echo "- It read the pom.xml dependencies"
echo

# Step 2: Build an index
echo "📚 Step 2: Build a search index"
echo "--------------------------------"
echo "Now we'll build a searchable database of our code..."
echo
cargo run --manifest-path ../../Cargo.toml -- index --project-root practice-project --index-path ./practice-index --force
echo
echo "💡 What just happened:"
echo "- Created an index directory"
echo "- Analyzed all Java code"
echo "- Built a searchable database"
echo

# Step 3: Search for classes
echo "🔎 Step 3: Search for classes"
echo "------------------------------"
echo "Let's find all classes in our project..."
echo
cargo run --manifest-path ../../Cargo.toml -- search --query "" --kind exact --filter-kind class --index-path ./practice-index
echo
echo "💡 What you see:"
echo "- Found 2 classes: HelloWorld and Calculator"
echo "- Each shows file location and line numbers"
echo

# Step 4: Search by name
echo "🔍 Step 4: Search by specific name"
echo "----------------------------------"
echo "Let's search for the Calculator class..."
echo
cargo run --manifest-path ../../Cargo.toml -- search --query Calculator --kind exact --index-path ./practice-index
echo
echo "💡 This shows:"
echo "- Only the Calculator class"
echo "- Its full signature"
echo "- Where it's defined"
echo

# Step 5: Create a simple graph
echo "📊 Step 5: Create a dependency graph"
echo "------------------------------------"
echo "Let's visualize our classes..."
echo
cargo run --manifest-path ../../Cargo.toml -- graph --project-root practice-project --index-path ./practice-index --output practice-graph.mmd --format mermaid
echo
echo "💡 Graph created! You can:"
echo "- Open practice-graph.mmd in https://mermaid.live"
echo "- See how classes relate to each other"
echo

# Interactive section
echo "🎯 Interactive Practice"
echo "======================"
echo "Try these commands yourself:"
echo
echo "1. Search for HelloWorld:"
echo "   cargo run -- search --query HelloWorld --kind exact --index-path ./practice-index"
echo
echo "2. Find all methods:"
echo "   cargo run -- search --query "" --kind exact --filter-kind method --index-path ./practice-index"
echo
echo "3. Export as JSON:"
echo "   cargo run -- export --project-root practice-project --index-path ./practice-index --output my-classes.json --format json --kind class"
echo

# Summary
echo "🎓 Tutorial Complete!"
echo "===================="
echo "What you learned:"
echo "✅ How to parse a Maven project"
echo "✅ How to build a searchable index"
echo "✅ How to search for specific classes"
echo "✅ How to create dependency graphs"
echo "✅ How to export data"
echo
echo "🚀 Next steps:"
echo "- Try these commands on your own Java project"
echo "- Experiment with different search queries"
echo "- Check out tutorial-02-advanced.sh"
echo
echo "💡 Practice project ready at: practice-project/"

# Cleanup option
echo
echo "🧹 Press ENTER to clean up, or Ctrl+C to keep the practice files"
read -r
rm -rf practice-project practice-index practice-graph.mmd
echo "✅ Cleaned up! Files saved to memory."

echo
exit 0