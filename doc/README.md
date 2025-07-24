# Code-Insight UML Documentation

This directory contains comprehensive UML diagrams for the Code-Insight project, showing the architecture, type relationships, and data flow.

## 🎯 Available Diagrams

### 1. [uml-architecture.mmd](uml-architecture.mmd)
**High-level Architecture Overview**
- Core types and their relationships
- Main traits and their implementations
- System boundaries and interfaces
- Component interactions

### 2. [detailed-class-structure.mmd](detailed-class-structure.mmd)
**Detailed Type Relationships**
- Complete class hierarchy
- Data flow between components
- Multiplicity and relationships
- Complex type compositions

## 🔍 How to View

### Option 1: Online Mermaid Live Editor
1. Open [Mermaid Live Editor](https://mermaid.live)
2. Copy and paste the `.mmd` file contents (without the backticks)
3. View and interact with the diagrams

### Option 2: VS Code Extension
Install the "Mermaid" extension in VS Code to view diagrams directly in the editor.

### Option 3: GitHub Preview
GitHub automatically renders Mermaid diagrams when viewing `.mmd` files.

### Option 4: Mermaid CLI
```bash
# Install globally
npm install -g @mermaid-js/mermaid-cli

# Generate PNG
mmdc -i doc/uml-architecture.mmd -o doc/architecture.png

# Generate SVG
mmdc -i doc/uml-architecture.mmd -o doc/architecture.svg
```

## 📊 Quick Start

### View Architecture Diagram
1. Open [Mermaid Live Editor](https://mermaid.live)
2. Copy contents from `uml-architecture.mmd`
3. Paste into editor and see the live diagram

### View Class Structure
1. Open [Mermaid Live Editor](https://mermaid.live)
2. Copy contents from `detailed-class-structure.mmd`
3. Paste into editor for detailed type relationships

## 🔄 Data Flow

```
File Discovery → Parse → AST → Declarations → Index → Search → Visualize
```

## 📁 Key Relationships

- **Containment**: JavaFile contains multiple Declarations
- **Inheritance**: Declaration types (Class, Method, Field)
- **Aggregation**: MavenModule contains JavaFiles and Dependencies
- **Composition**: SearchIndex composed of Tantivy components

## 🎨 Diagram Categories

### Core Types
- **JavaFile**: Main data structure for parsed Java files
- **MavenModule**: Represents Maven project structure
- **Declaration**: Base type for all Java declarations

### Parser Architecture
- **FileParser**: Trait for file parsing
- **ParserPipeline**: Multi-stage parsing process
- **JavaParser**: Tree-sitter based Java parser

### Indexing System
- **IndexManager**: Tantivy-based indexing system
- **SearchIndex**: Full-text search functionality
- **QueryEngine**: Search query processing

### Visualization
- **GraphBuilder**: Relationship graph construction
- **GraphVisualizer**: Mermaid/DOT output generation
- **ReferenceGraph**: Connection analysis

## 🚀 Usage Examples

### GitHub Integration
GitHub will automatically render these diagrams when you push to the repository. Just navigate to the file in GitHub's file browser.

### VS Code Integration
Install the "Mermaid Preview" extension to see live previews while editing.

### Command Line
```bash
# Quick preview
mmdc -p -i doc/uml-architecture.mmd

# Generate documentation site
mmdc -i doc/uml-architecture.mmd -o docs/architecture.html --theme dark
```

## 📋 Troubleshooting

### Mermaid Not Rendering
- Ensure no markdown code blocks (```) are in the .mmd files
- Check for proper Mermaid syntax
- Use online validator at mermaid.live

### Diagram Too Large
- Use zoom functionality in Mermaid Live Editor
- Break into smaller focused diagrams
- Use direction changes (TB, LR, BT, RL)

## 📝 Notes

- All diagrams use pure Mermaid syntax
- No markdown formatting within diagram files
- Compatible with GitHub rendering
- Optimized for readability and navigation