use code_insight::{
    indexer::IndexManager,
    parser::{JavaParser, FileParser},
    types::{DeclarationKind, SearchQuery, SearchFilter},
};
use tempfile::tempdir;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let project_root = dir.path();
    let index_path = dir.path().join("test_index");

    // Create test project
    let src_dir = project_root.join("src/main/java/com/example");
    fs::create_dir_all(src_dir.join("model"))?;
    
    let user_java = r#"
package com.example.model;

public class User {
    private Long id;
    private String name;
    
    public User(Long id, String name) {
        this.id = id;
        this.name = name;
    }
    
    public Long getId() { return id; }
    public String getName() { return name; }
}
"#;
    fs::write(src_dir.join("model/User.java"), user_java)?;

    // Parse and index
    let index_manager = IndexManager::new(&index_path)?;
    let mut java_parser = JavaParser::new()?;
    
    let file_path = src_dir.join("model/User.java");
    let java_file = java_parser.parse_file(&file_path)?;
    
    println!("Parsed file: {} declarations", java_file.declarations.len());
    for decl in &java_file.declarations {
        println!("  - {}: {:?}", decl.name, decl.kind);
    }
    
    index_manager.index_java_file(&java_file).await?;
    index_manager.optimize().await?;
    
    // Check stats
    let (num_docs, _) = index_manager.stats()?;
    println!("Indexed documents: {}", num_docs);
    
    // Test search
    let query_engine = code_insight::query::QueryEngine::new(&index_path)?;
    let classes = query_engine.search_by_kind(DeclarationKind::Class, None).await?;
    println!("Found {} classes", classes.len());
    
    for class in classes {
        println!("  - {}: {}", class.declaration.name, class.declaration.kind);
    }
    
    Ok(())
}