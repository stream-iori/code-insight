use anyhow::Result;
use std::path::Path;
use std::fs;
use tempfile::tempdir;

use code_insight::{
    parser::{JavaStructureParser},
};

#[tokio::test]
async fn test_parser_edge_cases() -> Result<()> {
    let dir = tempdir()?;
    
    // Test Java parser edge cases
    test_java_parser_edge_cases(dir.path())?;
    
    Ok(())
}

fn test_java_parser_edge_cases(test_dir: &Path) -> Result<()> {
    let mut parser = JavaStructureParser::new()?;
    
    // Test 1: Empty Java file
    let empty_java = test_dir.join("Empty.java");
    fs::write(&empty_java, "")?;
    let result = parser.parse_structure(&empty_java);
    assert!(result.is_ok(), "Empty file should be handled gracefully");
    
    // Test 2: Java file with only comments
    let comments_only = test_dir.join("CommentsOnly.java");
    fs::write(&comments_only, 
        r#"
        // This is a comment
        /* Another comment */
        /** Javadoc comment */
        "#
    )?;
    let result = parser.parse_structure(&comments_only);
    assert!(result.is_ok(), "Comments-only file should be handled");
    
    // Test 3: Java file with invalid syntax
    let invalid_syntax = test_dir.join("Invalid.java");
    fs::write(&invalid_syntax, "public class { }")?;
    let result = parser.parse_structure(&invalid_syntax);
    // Note: tree-sitter is quite forgiving, so invalid syntax might not always return error
    let _ = result; // Just check it doesn't panic
    
    // Test 4: Java file with unicode characters in names
    let unicode_class = test_dir.join("Unicode.java");
    fs::write(&unicode_class, 
        r#"
        package com.example;
        
        public class Пользователь {
            private String имя;
            
            public String получитьИмя() {
                return имя;
            }
        }
        "#
    )?;
    let result = parser.parse_structure(&unicode_class);
    assert!(result.is_ok(), "Unicode characters should be handled");
    let java_structure = result?;
    assert_eq!(java_structure.top_level_classes[0].name, "Пользователь");
    
    // Test 5: Java file with deeply nested generics
    let nested_generics = test_dir.join("NestedGenerics.java");
    fs::write(&nested_generics, 
        r#"
        import java.util.*;
        
        public class NestedGenerics {
            private Map<String, List<Map<Integer, Set<String>>>> complexMap;
            
            public List<Map<String, List<Integer>>> complexMethod(
                Map<String, List<Map<Integer, String>>> param) {
                return null;
            }
        }
        "#
    )?;
    let result = parser.parse_structure(&nested_generics);
    assert!(result.is_ok(), "Deeply nested generics should be handled");
    
    // Test 6: Java file with annotations having complex values
    let complex_annotations = test_dir.join("ComplexAnnotations.java");
    fs::write(&complex_annotations, 
        r#"
        @Entity(table="users", schema="public")
        @Table(name="user_table", indexes={
            @Index(name="idx_email", columnList="email"),
            @Index(name="idx_status", columnList="status")
        })
        public class User {
            @Id
            @GeneratedValue(strategy=GenerationType.IDENTITY)
            private Long id;
            
            @Column(name="user_email", nullable=false, length=255)
            private String email;
            
            @ElementCollection(fetch=FetchType.EAGER)
            @CollectionTable(name="user_roles", joinColumns=@JoinColumn(name="user_id"))
            @Column(name="role_name")
            private Set<String> roles;
        }
        "#
    )?;
    let result = parser.parse_structure(&complex_annotations);
    assert!(result.is_ok(), "Complex annotations should be handled");
    
    // Test 7: Java file with lambda expressions and method references
    let modern_features = test_dir.join("ModernFeatures.java");
    fs::write(&modern_features, 
        r#"
        import java.util.*;
        import java.util.stream.*;
        
        public class ModernFeatures {
            public void processUsers(List<User> users) {
                users.stream()
                    .filter(u -> u.getAge() > 18)
                    .map(User::getName)
                    .collect(Collectors.toList());
            }
            
            public interface User {
                String getName();
                int getAge();
            }
        }
        "#
    )?;
    let result = parser.parse_structure(&modern_features);
    assert!(result.is_ok(), "Modern Java features should be handled");
    
    // Test 8: Java file with varargs
    let varargs_class = test_dir.join("Varargs.java");
    fs::write(&varargs_class, 
        r#"
        public class Varargs {
            public void processStrings(String... strings) {
                for (String s : strings) {
                    System.out.println(s);
                }
            }
            
            public void processMixed(String header, int... numbers) {
                System.out.println(header + numbers.length);
            }
        }
        "#
    )?;
    let result = parser.parse_structure(&varargs_class);
    assert!(result.is_ok(), "Varargs should be handled");
    
    // Test 9: Java file with inner classes and enums
    let inner_classes = test_dir.join("InnerClasses.java");
    fs::write(&inner_classes, 
        r#"
        public class OuterClass {
            private String outerField;
            
            public class InnerClass {
                private String innerField;
                
                public void innerMethod() {
                    System.out.println(outerField + innerField);
                }
            }
            
            public static class StaticNestedClass {
                public void nestedMethod() { }
            }
            
            public enum Status {
                ACTIVE("active"),
                INACTIVE("inactive");
                
                private final String value;
                
                Status(String value) {
                    this.value = value;
                }
            }
            
            @FunctionalInterface
            public interface Action {
                void perform();
            }
        }
        "#
    )?;
    let result = parser.parse_structure(&inner_classes);
    assert!(result.is_ok(), "Inner classes and enums should be handled");
    // Tree-sitter might parse differently, just check it doesn't fail
    let _ = result?.top_level_classes.len();
    
    // Test 10: Java file with records (Java 14+)
    let record_class = test_dir.join("RecordClass.java");
    fs::write(&record_class, 
        r#"
        public record UserRecord(Long id, String name, String email) {
            public UserRecord {
                if (name == null || name.isBlank()) {
                    throw new IllegalArgumentException("Name cannot be blank");
                }
            }
            
            public String displayName() {
                return name + " (" + id + ")";
            }
        }
        "#
    )?;
    let result = parser.parse_structure(&record_class);
    assert!(result.is_ok(), "Java records should be handled");
    
    Ok(())
}