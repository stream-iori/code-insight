# Code Insight

A powerful Rust-based tool for parsing and analyzing Maven Java projects, designed for LLM/RAG integration and code intelligence.

## Features

### 🔍 **Project Analysis**
- **Maven Module Parsing**: Complete project structure analysis with dependencies
- **Multi-format Support**: Java, XML, and properties file parsing
- **Dependency Graphs**: Visualize module and type relationships

### 🧠 **Intelligent Search**
- **Full-text Search**: Exact, fuzzy, and regex search capabilities
- **Semantic Filtering**: Filter by declaration type, annotations, packages
- **Real-time Indexing**: Fast, incremental indexing with Tantivy

### 🤖 **LLM/RAG Integration**
- **Structured Export**: JSON, JSONL, Markdown, LlamaIndex formats
- **Annotation-based Filtering**: Export `@Service`, `@Controller`, etc.
- **Source Code Inclusion**: Optional full source code export
- **Line Range Metadata**: Precise location information for RAG systems

### 📊 **Visualization**
- **Mermaid Graphs**: Dependency relationship visualization
- **DOT/Graphviz**: Professional graph generation
- **Focused Analysis**: Zoom into specific components
- **Interactive CLI**: Rich command-line interface

### ⚡ **Performance**
- **Async Processing**: Concurrent file processing
- **Rayon Parallelism**: Multi-threaded parsing
- **Memory Efficient**: Streaming processing for large projects
- **Backpressure Handling**: Optimal resource utilization

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/code-insight.git
cd code-insight

# Build the project
cargo build --release

# Install globally
cargo install --path .
```

### Basic Usage

```bash
# Parse and analyze a Maven project
code-insight parse --project-root ./my-java-project

# Build search index
code-insight index --project-root ./my-java-project

# Search for declarations
code-insight search --query "UserService" --kind exact

# Export for LLM/RAG
code-insight export --output export.json --format json --annotation Service

# Generate dependency graph
code-insight graph --output deps.mmd --format mermaid

# Show project statistics
code-insight stats
```

## CLI Commands

### `parse`
Parse project structure and dependencies.
```bash
code-insight parse [--verbose] [--project-root PATH]
```

### `index`
Build search index from source files.
```bash
code-insight index [--force] [--project-root PATH] [--index-path PATH]
```

### `search`
Search declarations with advanced filtering.
```bash
code-insight search \
  --query "UserService" \
  --kind [exact|fuzzy|regex] \
  --filter-kind [class|interface|enum|record|annotation] \
  --filter-annotation "Service" \
  --limit 10
```

### `export`
Export structured data for LLM/RAG systems.
```bash
code-insight export \
  --output export.json \
  --format [json|jsonl|markdown|llama-index|rag] \
  --kind class \
  --annotation "Service" \
  --package "com.example" \
  --limit 100 \
  --include-source
```

### `graph`
Generate dependency graphs.
```bash
code-insight graph \
  --output graph.mmd \
  --format [mermaid|dot|svg] \
  --focus "UserService" \
  --depth 2
```

### `stats`
Display project statistics.
```bash
code-insight stats [--project-root PATH] [--index-path PATH]
```

## Advanced Usage

### LLM Integration Examples

#### Export all service classes
```bash
code-insight export \
  --output services.json \
  --format json \
  --kind class \
  --annotation Service \
  --include-source
```

#### Export interfaces for API documentation
```bash
code-insight export \
  --output interfaces.jsonl \
  --format jsonl \
  --kind interface \
  --include-source
```

#### Generate RAG chunks
```bash
code-insight export \
  --output rag-chunks.json \
  --format rag \
  --annotation Controller \
  --limit 50
```

### Programmatic Usage

```rust
use code_insight::{
    maven::MavenParser,
    indexer::IndexManager,
    query::QueryEngine,
    llm::{LlmExporter, LlmRequest, ExportFormat},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_root = "/path/to/java/project";
    let index_path = "/path/to/index";

    // Build index
    let index_manager = IndexManager::new(index_path)?;
    // ... process files ...

    // Query declarations
    let query_engine = QueryEngine::new(index_path)?;
    let results = query_engine.search_by_kind(DeclarationKind::Class, Some(10)).await?;

    // Export for LLM
    let exporter = LlmExporter::new(query_engine, project_root.into())?;
    let request = LlmRequest {
        query: None,
        kind: Some(DeclarationKind::Class),
        annotations: vec!["Service".to_string()],
        package: None,
        limit: Some(100),
        include_source: true,
        format: ExportFormat::Json,
    };

    let response = exporter.export(request).await?;
    println!("Exported {} declarations", response.metadata.total_count);

    Ok(())
}
```

## Configuration

### Environment Variables
- `CODE_INSIGHT_MAX_WORKERS`: Maximum concurrent workers (default: CPU cores)
- `CODE_INSIGHT_BATCH_SIZE`: Processing batch size (default: 1000)
- `CODE_INSIGHT_INDEX_HEAP_SIZE`: Tantivy heap size in MB (default: 50)

### Performance Tuning

For large projects (>10k files):
```bash
# Increase workers and batch size
export CODE_INSIGHT_MAX_WORKERS=16
export CODE_INSIGHT_BATCH_SIZE=5000

# Run with optimized settings
code-insight index --project-root ./large-project
```

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Maven Parser  │    │   Java Parser   │    │  XML/Properties │
│   (modules)     │    │  (declarations) │    │   (metadata)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Index Manager │
                    │   (Tantivy)     │
                    └─────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Query Engine  │    │   LLM Exporter  │    │  Graph Builder  │
│   (search)      │    │   (RAG)         │    │  (visualization)│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## File Structure

```
code-insight/
├── src/
│   ├── maven/          # Maven project parsing
│   ├── parser/         # Java/XML/properties parsing
│   ├── indexer/        # Tantivy indexing
│   ├── query/          # Search and filtering
│   ├── graph/          # Relationship graphs
│   ├── llm/            # LLM/RAG export
│   ├── cli/            # Command-line interface
│   ├── async/          # Concurrent processing
│   └── types/          # Shared data types
├── tests/              # Integration tests
└── examples/           # Usage examples
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make changes and add tests
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy`
6. Format code: `cargo fmt`
7. Submit a pull request

## Testing

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test integration_test

# Run with coverage
cargo tarpaulin --out Html
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Support

- 📖 [Documentation](https://github.com/your-org/code-insight/wiki)
- 🐛 [Issues](https://github.com/your-org/code-insight/issues)
- 💬 [Discussions](https://github.com/your-org/code-insight/discussions)

## Roadmap

- [ ] TUI interface with code preview
- [ ] IDE extensions (VS Code, IntelliJ)
- [ ] Web interface
- [ ] Database support (PostgreSQL, MongoDB)
- [ ] Advanced type inference
- [ ] Security vulnerability scanning
- [ ] Performance profiling integration