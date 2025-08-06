/// Main library exports for the code-insight tool
/// 
/// This tool helps you understand Java projects by:
/// 1. Analyzing Java source code 
/// 2. Building searchable indexes
/// 3. Creating dependency graphs
/// 4. Exporting data for AI tools
/// 
/// # Basic Usage
/// 
/// ```rust,no_run
/// use code_insight::{
///     parser::FileParser,
///     indexer::IndexManager,
/// };
/// use std::path::Path;
/// 
/// // Build a search index
/// let indexer = IndexManager::new(Path::new("/path/to/index")).unwrap();
/// ```

pub mod types;
pub mod parser;
pub mod indexer;
pub mod query;
pub mod llm;
pub mod cli;
pub mod r#async;
mod type_config;

pub use types::*;
pub use cli::*;