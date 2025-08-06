use anyhow::Result;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use code_insight::{
    parser::{FileParser, JavaStructureParser},
    types::{DeclarationKind, SearchFilter, SearchKind},
};

#[tokio::test]
async fn test_parser_with_local_vertx() -> Result<()> {
    let project_root_path = Path::new("/Users/stream/codes/java/vert.x/vertx-core-logging");

    let file_parser = FileParser;
    let java_structure_parser = JavaStructureParser;
    let source_files = file_parser.find_source_files(project_root_path)?;
    source_files.iter().for_each(|file| {
        if (file.extension().and_then(|e| e.to_str()) == Some("java")) {
            println!(
                "{:#?}",
                java_structure_parser.parse_structure(file.as_path())
            );
        }
    });

    Ok(())
}
