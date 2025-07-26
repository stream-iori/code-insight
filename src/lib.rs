/// Main library exports for the code-insight tool
/// 
/// This tool helps you understand Java projects by:
/// 1. Parsing Maven pom.xml files
/// 2. Analyzing Java source code 
/// 3. Building searchable indexes
/// 4. Creating dependency graphs
/// 5. Exporting data for AI tools
/// 
/// # Basic Usage
/// 
/// ```rust,no_run
/// use code_insight::{
///     maven::MavenParser,
///     parser::FileParser,
///     indexer::IndexManager,
/// };
/// use std::path::Path;
/// 
/// // Parse a Maven project
/// let parser = MavenParser;
/// let modules = parser.find_maven_modules(Path::new("/path/to/project")).unwrap();
/// 
/// // Build a search index
/// let indexer = IndexManager::new(Path::new("/path/to/index")).unwrap();
/// ```

pub mod types;
pub mod maven;
pub mod parser;
pub mod indexer;
pub mod query;
pub mod graph;
pub mod llm;
pub mod cli;
pub mod r#async;

pub use types::*;
pub use cli::*;