use code_insight::parser::JavaParser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple test file
    let java_content = r#"
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
    
    let test_path = "/tmp/test_user.java";
    fs::write(test_path, java_content)?;
    
    let mut parser = JavaParser::new()?;
    let java_file = parser.parse_file(test_path.as_ref())?;
    
    println!("Package: '{}'", java_file.package);
    println!("Declarations: {}", java_file.declarations.len());
    for decl in &java_file.declarations {
        println!("  - {}: {:?}", decl.name, decl.kind);
    }
    
    fs::remove_file(test_path)?;
    Ok(())
}