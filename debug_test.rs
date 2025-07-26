use code_insight::parser::{JavaParser, FileParser};
use tempfile::tempdir;
use std::fs;

fn main() {
    let dir = tempdir().unwrap();
    let project_root = dir.path();
    
    // Create test structure
    let src_dir = project_root.join("src/main/java/com/example");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(src_dir.join("model")).unwrap();
    
    // Create User.java
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
    fs::write(src_dir.join("model/User.java"), user_java).unwrap();
    
    // Test parsing
    let mut parser = JavaParser::new().unwrap();
    let result = parser.parse_file(&src_dir.join("model/User.java"));
    
    match result {
        Ok(java_file) => {
            println!("Successfully parsed User.java");
            println!("Package: {}", java_file.package);
            println!("Declarations: {}", java_file.declarations.len());
            for decl in &java_file.declarations {
                println!("  - {}: {:?}", decl.name, decl.kind);
            }
        }
        Err(e) => {
            println!("Failed to parse: {}", e);
        }
    }
}