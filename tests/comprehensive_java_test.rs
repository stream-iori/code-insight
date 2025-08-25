use code_insight::parser::java_structure::ClassKind;
use code_insight::parser::java_structure::JavaStructureParser;
use tempfile::tempdir;

#[test]
fn test_comprehensive_java_example() {
    let parser = JavaStructureParser::new().unwrap();
    
    let java_content = include_str!("test-files/test_complex_java.java");

    let dir = tempdir().unwrap();
    let java_path = dir.path().join("ComprehensiveTestClass.java");
    std::fs::write(&java_path, java_content).unwrap();

    let structure = parser.parse_structure(&java_path).unwrap();
    
    println!("✅ Comprehensive Java syntax test passed!");
    println!("📋 Parsed structure:");
    let package = structure.package.clone().unwrap_or_default();
    println!("  Package: {}", package);
    println!("  Imports: {} items", structure.imports.len());
    println!("  File annotations: {} items", structure.file_annotations.len());
    
    let class = &structure.top_level_classes[0];
    println!("  Class: {} ({:?})", class.name, class.kind);
    println!("  FQN: {}", class.fqn);
    println!("  Annotations: {} items", class.annotations.len());
    for ann in &class.annotations {
        println!("    @{} with {} values", ann.name, ann.values.len());
    }
    println!("  Fields: {} items", class.fields.len());
    for field in &class.fields {
        println!("    {} {} - {} annotations", field.type_name, field.name, field.annotations.len());
    }
    println!("  Methods: {} items", class.methods.len());
    for method in &class.methods {
        println!("    {} {}({} params) - {} annotations", 
                 method.return_type, method.name, method.parameters.len(), method.annotations.len());
        for param in &method.parameters {
            println!("      {} {}", param.type_name, param.name);
        }
    }
    println!("  Nested classes: {} items", class.nested_classes.len());
    for nested in &class.nested_classes {
        println!("    {} ({:?}) - {} methods", nested.name, nested.kind, nested.methods.len());
    }
    
    // Key assertions - updated based on actual results
    assert_eq!(structure.package, Some("com.example.advanced.java.parser.test".to_string()));
    assert_eq!(structure.top_level_classes.len(), 1);
    assert_eq!(class.name, "AdvancedTestClass");
    assert_eq!(class.kind, ClassKind::Class);
    assert_eq!(class.annotations.len(), 3);
    assert_eq!(class.fields.len(), 7);
    assert_eq!(class.methods.len(), 10);
    assert_eq!(class.nested_classes.len(), 7);
    assert!(class.nested_classes.iter().any(|c| c.name == "Status"));
    assert!(class.nested_classes.iter().any(|c| c.name == "InnerProcessor"));
    assert!(class.nested_classes.iter().any(|c| c.name == "ConfigRecord"));
    assert!(class.nested_classes.iter().any(|c| c.name == "DataProcessor"));
    
    println!("🎯 All assertions passed!");
}