use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tokio;

use crate::parser::JavaStructureParser;
use crate::{
    indexer::IndexManager,
    llm::{ExportFormat, LlmExporter},
    parser::FileParser,
    query::QueryEngine,
    types::{DeclarationKind, SearchKind, SearchQuery},
};

#[derive(Parser)]
#[command(name = "code-insight")]
#[command(about = "A tool for parsing and analyzing Java projects")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, default_value = ".")]
    pub project_root: PathBuf,

    #[arg(short, long, default_value = ".code-insight/index")]
    pub index_path: PathBuf,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Parse Java project structure
    Parse {
        #[arg(short, long)]
        verbose: bool,
    },

    /// Build search index
    Index {
        #[arg(short, long)]
        force: bool,
    },

    /// Search declarations
    Search {
        #[arg(short, long)]
        query: String,

        #[arg(short, long, default_value = "exact")]
        kind: SearchKindArg,

        #[arg(short, long)]
        limit: Option<usize>,

        #[arg(short, long)]
        filter_kind: Option<DeclarationKindArg>,

        #[arg(short, long)]
        filter_annotation: Option<String>,
    },

    /// Export for LLM/RAG systems
    Export {
        #[arg(short, long)]
        output: PathBuf,

        #[arg(short, long, default_value = "json")]
        format: ExportFormatArg,

        #[arg(short, long)]
        kind: Option<DeclarationKindArg>,

        #[arg(short, long)]
        annotation: Option<String>,

        #[arg(short, long)]
        package: Option<String>,

        #[arg(short, long)]
        limit: Option<usize>,

        #[arg(long)]
        include_source: bool,
    },


    /// Run interactive TUI
    Tui,

    /// Show project statistics
    Stats,
}

#[derive(clap::ValueEnum, Clone)]
pub enum SearchKindArg {
    Exact,
    Fuzzy,
    Regex,
}

impl From<SearchKindArg> for SearchKind {
    fn from(arg: SearchKindArg) -> Self {
        match arg {
            SearchKindArg::Exact => SearchKind::Exact,
            SearchKindArg::Fuzzy => SearchKind::Fuzzy,
            SearchKindArg::Regex => SearchKind::Regex,
        }
    }
}

#[derive(clap::ValueEnum, Clone)]
pub enum DeclarationKindArg {
    Class,
    Interface,
    Enum,
    Record,
    Annotation,
}

impl From<DeclarationKindArg> for DeclarationKind {
    fn from(arg: DeclarationKindArg) -> Self {
        match arg {
            DeclarationKindArg::Class => DeclarationKind::Class,
            DeclarationKindArg::Interface => DeclarationKind::Interface,
            DeclarationKindArg::Enum => DeclarationKind::Enum,
            DeclarationKindArg::Record => DeclarationKind::Record,
            DeclarationKindArg::Annotation => DeclarationKind::Annotation,
        }
    }
}

#[derive(clap::ValueEnum, Clone)]
pub enum ExportFormatArg {
    Json,
    Jsonl,
    Markdown,
    LlamaIndex,
    Rag,
}

impl From<ExportFormatArg> for ExportFormat {
    fn from(arg: ExportFormatArg) -> Self {
        match arg {
            ExportFormatArg::Json => ExportFormat::Json,
            ExportFormatArg::Jsonl => ExportFormat::Jsonl,
            ExportFormatArg::Markdown => ExportFormat::Markdown,
            ExportFormatArg::LlamaIndex => ExportFormat::LlamaIndex,
            ExportFormatArg::Rag => ExportFormat::RAG,
        }
    }
}


pub async fn run(args: Args) -> Result<()> {
    match args.command {
        Commands::Parse { verbose } => parse_java_project(&args.project_root, verbose).await,
        Commands::Index { force } => build_index(&args.project_root, &args.index_path, force).await,
        Commands::Search {
            query,
            kind,
            limit,
            filter_kind,
            filter_annotation,
        } => {
            search_declarations(
                &args.index_path,
                &query,
                kind.into(),
                limit,
                filter_kind.map(Into::into),
                filter_annotation,
            )
            .await
        }
        Commands::Export {
            output,
            format,
            kind,
            annotation,
            package,
            limit,
            include_source,
        } => {
            export_for_llm(
                &args.project_root,
                &args.index_path,
                output,
                format.into(),
                kind.map(Into::into),
                annotation,
                package,
                limit,
                include_source,
            )
            .await
        }
        Commands::Tui => run_tui(&args.project_root, &args.index_path).await,
        Commands::Stats => show_stats(&args.project_root, &args.index_path).await,
    }
}

// We should make it configurable, project language and build tools
async fn parse_java_project(project_root: &Path, verbose: bool) -> Result<()> {
    println!("🔍 Parsing Java project at: {}", project_root.display());

    let file_parser = FileParser;

    // Find source files
    let source_files = file_parser.find_source_files(project_root)?;
    println!("📄 Found {} source files", source_files.len());

    // Count by type
    #[derive(Default)]
    struct fileCounts {
        java: usize,
        other: usize,
    }

    let fileCounts = source_files
        .iter()
        .fold(fileCounts::default(), |mut acc, file| {
            match file.extension().and_then(|e| e.to_str()) {
                Some("java") => acc.java += 1,
                _ => acc.other += 1,
            }
            acc
        });

    println!("  - Java files: {}", fileCounts.java);

    Ok(())
}

async fn build_index(project_root: &Path, index_path: &Path, force: bool) -> Result<()> {
    println!("📚 Building search index...");
    println!("Project root: {}", project_root.display());
    println!("Index path: {}", index_path.display());

    if force && index_path.exists() {
        println!("🗑️  Removing existing index...");
        std::fs::remove_dir_all(index_path).context("Failed to remove existing index")?;
    }

    let index_manager = IndexManager::new(index_path)?;
    let file_parser = FileParser::new()?;
    let mut java_structure_parser = JavaStructureParser::new()?;

    let java_files = file_parser
        .find_source_files(project_root)?
        .into_iter()
        .filter(|p| p.extension().map_or(false, |e| e == "java"))
        .collect::<Vec<_>>();

    println!("📄 Found {} Java files to index", java_files.len());

    let mut processed = 0;
    for file_path in java_files {
        match java_structure_parser.parse_structure(&file_path) {
            Ok(java_structure) => {
                index_manager.index_java_file(&java_structure).await?;
                processed += 1;

                if processed % 100 == 0 {
                    println!("  ✅ Indexed {} files...", processed);
                }
            }
            Err(e) => {
                eprintln!("⚠️  Failed to parse {}: {}", file_path.display(), e);
            }
        }
    }

    index_manager.optimize().await?;

    println!("✅ Successfully indexed {} files", processed);
    Ok(())
}

async fn search_declarations(
    index_path: &Path,
    query: &str,
    kind: SearchKind,
    limit: Option<usize>,
    filter_kind: Option<DeclarationKind>,
    filter_annotation: Option<String>,
) -> Result<()> {
    let query_engine = QueryEngine::new(index_path)?;

    let mut filters = Vec::new();
    if let Some(k) = filter_kind {
        filters.push(crate::types::SearchFilter::Kind(k));
    }
    if let Some(ann) = filter_annotation {
        filters.push(crate::types::SearchFilter::Annotation(ann));
    }

    let search_query = SearchQuery {
        query: query.to_string(),
        kind,
        filters,
        limit,
    };

    let results = query_engine.search(&search_query).await?;

    println!("🔍 Found {} results for '{}'", results.len(), query);

    for (i, result) in results.iter().enumerate() {
        println!(
            "{}. {} ({}) - {}",
            i + 1,
            result.declaration.name,
            format!("{:?}", result.declaration.kind).to_lowercase(),
            result.file_path.display()
        );

        if let Some(doc) = &result.declaration.documentation {
            println!("   📖 {}", doc.lines().next().unwrap_or(""));
        }

        println!(
            "   📍 {}:{}-{}\n",
            result.file_path.display(),
            result.declaration.range.start_line,
            result.declaration.range.end_line
        );
    }

    Ok(())
}

async fn export_for_llm(
    project_root: &Path,
    index_path: &Path,
    output: PathBuf,
    format: ExportFormat,
    kind: Option<DeclarationKind>,
    annotation: Option<String>,
    package: Option<String>,
    limit: Option<usize>,
    include_source: bool,
) -> Result<()> {
    println!("🤖 Exporting for LLM/RAG...");

    let query_engine = QueryEngine::new(index_path)?;
    let exporter = LlmExporter::new(query_engine, project_root.to_path_buf())?;

    let request = crate::llm::LlmRequest {
        query: None,
        kind,
        annotations: annotation.map(|a| vec![a]).unwrap_or_default(),
        package,
        limit,
        include_source,
        format: format.clone(),
    };

    let response = exporter.export(request).await?;
    let formatted = exporter.format_export(&response, &format)?;

    tokio::fs::write(&output, formatted)
        .await
        .context("Failed to write output file")?;

    println!(
        "✅ Exported {} declarations to {} in {:?} format",
        response.metadata.total_count,
        output.display(),
        format
    );

    Ok(())
}


async fn run_tui(project_root: &Path, index_path: &Path) -> Result<()> {
    println!("🖥️  Starting interactive TUI...");
    println!("TUI functionality not yet implemented.");
    println!("Project root: {}", project_root.display());
    println!("Index path: {}", index_path.display());

    // TODO: Implement TUI using ratatui
    Ok(())
}

async fn show_stats(project_root: &Path, index_path: &Path) -> Result<()> {
    println!("📊 Project Statistics");
    println!("===================");

    let query_engine = QueryEngine::new(index_path)?;
    let stats = query_engine.get_statistics().await?;

    println!("📁 Project root: {}", project_root.display());
    println!("📚 Total declarations: {}", stats.total_declarations);
    println!("🏗️ Classes: {}", stats.class_count);
    println!("🔧 Interfaces: {}", stats.interface_count);
    println!("📋 Enums: {}", stats.enum_count);
    println!("📦 Records: {}", stats.record_count);
    println!("📝 Annotations: {}", stats.annotation_count);

    let (cache_entries, cache_items) = query_engine.get_cache_stats().await;
    println!("💾 Cache entries: {}", cache_entries);
    println!("💾 Cache items: {}", cache_items);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cli_commands() {
        let dir = tempdir().unwrap();
        let project_root = dir.path();
        let index_path = dir.path().join("index");

        // Test parse command
        let args = Args {
            command: Commands::Parse { verbose: false },
            project_root: project_root.to_path_buf(),
            index_path: index_path.clone(),
        };

        let result = run(args).await;
        assert!(result.is_ok());
    }
}
