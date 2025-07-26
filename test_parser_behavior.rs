use code_insight::parser::JavaParser;
use std::fs;

fn main() {
    let mut parser = JavaParser::new().unwrap();
    
    // Test 1: Invalid syntax
    let invalid_content = "invalid java syntax {";
    fs::write("/tmp/test_invalid.java", invalid_content).unwrap();
    
    let result = parser.parse_file(std::path::Path::new("/tmp/test_invalid.java"));
    println!("Invalid syntax test: {:?}", result);
    
    // Test 2: Empty file
    fs::write("/tmp/test_empty.java", "").unwrap();
    let result = parser.parse_file(std::path::Path::new("/tmp/test_empty.java"));
    println!("Empty file test: {:?}", result);
}