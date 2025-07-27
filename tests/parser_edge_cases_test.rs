use anyhow::Result;
use std::path::Path;
use std::fs;
use tempfile::tempdir;

use code_insight::{
    parser::{JavaStructureParser, XmlParser, PropertiesParser},
};

#[tokio::test]
async fn test_parser_edge_cases() -> Result<()> {
    let dir = tempdir()?;
    
    // Test Java parser edge cases
    test_java_parser_edge_cases(dir.path())?;
    
    // Test XML parser edge cases
    test_xml_parser_edge_cases(dir.path())?;
    
    // Test Properties parser edge cases
    test_properties_parser_edge_cases(dir.path())?;
    
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

fn test_xml_parser_edge_cases(test_dir: &Path) -> Result<()> {
    let mut parser = XmlParser::new()?;
    
    // Test 1: Empty XML file
    let empty_xml = test_dir.join("empty.xml");
    fs::write(&empty_xml, "")?;
    let result = parser.parse_file(&empty_xml);
    assert!(result.is_ok(), "Empty XML should be handled");
    assert_eq!(result?.root_element, "unknown");
    
    // Test 2: Malformed XML - quick_xml is quite forgiving
    let malformed_xml = test_dir.join("malformed.xml");
    fs::write(&malformed_xml, "<root><child></root>")?;
    let result = parser.parse_file(&malformed_xml);
    // quick_xml might not return error for this, just check it doesn't panic
    let _ = result;
    
    // Test 3: XML with special characters
    let special_chars_xml = test_dir.join("special.xml");
    fs::write(&special_chars_xml, 
        r#"
        <configuration>
            <name>Test &amp; Configuration</name>
            <description><![CDATA[This is a <test> description]]></description>
            <unicode>Привет мир</unicode>
            <escaped>Line 1
            Line 2</escaped>
        </configuration>
        "#
    )?;
    let result = parser.parse_file(&special_chars_xml);
    assert!(result.is_ok(), "XML with special characters should be handled");
    
    // Test 4: XML with namespaces
    let namespace_xml = test_dir.join("namespace.xml");
    fs::write(&namespace_xml, 
        r#"
        <beans xmlns="http://www.springframework.org/schema/beans"
               xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
               xmlns:context="http://www.springframework.org/schema/context"
               xsi:schemaLocation="http://www.springframework.org/schema/beans
                                   http://www.springframework.org/schema/beans/spring-beans.xsd
                                   http://www.springframework.org/schema/context
                                   http://www.springframework.org/schema/context/spring-context.xsd">
            <context:component-scan base-package="com.example"/>
            <bean id="dataSource" class="org.springframework.jdbc.datasource.DriverManagerDataSource">
                <property name="driverClassName" value="com.mysql.jdbc.Driver"/>
                <property name="url" value="jdbc:mysql://localhost:3306/test"/>
            </bean>
        </beans>
        "#
    )?;
    let result = parser.parse_file(&namespace_xml);
    assert!(result.is_ok(), "XML with namespaces should be handled");
    
    // Test 5: XML with processing instructions and comments
    let complex_xml = test_dir.join("complex.xml");
    fs::write(&complex_xml, 
        r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <!-- This is a configuration file -->
        <configuration version="1.0">
            <?xml-stylesheet type="text/xsl" href="config.xsl"?>
            <servers>
                <server id="1" enabled="true">
                    <name>Production</name>
                    <host>prod.example.com</host>
                    <port>8080</port>
                </server>
                <server id="2" enabled="false">
                    <name>Development</name>
                    <host>dev.example.com</host>
                    <port>9090</port>
                </server>
            </servers>
        </configuration>
        "#
    )?;
    let result = parser.parse_file(&complex_xml);
    assert!(result.is_ok(), "Complex XML should be handled");
    
    // Test 6: XML with self-closing tags
    let self_closing_xml = test_dir.join("self_closing.xml");
    fs::write(&self_closing_xml, 
        r#"
        <configuration>
            <database url="localhost" port="5432" />
            <cache enabled="true" size="1000" />
            <logging level="INFO" />
        </configuration>
        "#
    )?;
    let result = parser.parse_file(&self_closing_xml);
    assert!(result.is_ok(), "XML with self-closing tags should be handled");
    
    // Test 7: XML with empty tags
    let empty_tags_xml = test_dir.join("empty_tags.xml");
    fs::write(&empty_tags_xml, 
        r#"
        <root>
            <empty></empty>
            <another_empty />
            <nested><deep_empty></deep_empty></nested>
        </root>
        "#
    )?;
    let result = parser.parse_file(&empty_tags_xml);
    assert!(result.is_ok(), "XML with empty tags should be handled");
    
    Ok(())
}

fn test_properties_parser_edge_cases(test_dir: &Path) -> Result<()> {
    let mut parser = PropertiesParser::new()?;
    
    // Test 1: Empty properties file
    let empty_props = test_dir.join("empty.properties");
    fs::write(&empty_props, "")?;
    let result = parser.parse_file(&empty_props);
    assert!(result.is_ok(), "Empty properties file should be handled");
    assert!(result?.properties.is_empty());
    
    // Test 2: Properties file with only comments
    let comments_only = test_dir.join("comments.properties");
    fs::write(&comments_only, 
        r#"
        # This is a comment
        ! This is also a comment
        # Another comment
        
        # Empty line above
        "#
    )?;
    let result = parser.parse_file(&comments_only);
    assert!(result.is_ok(), "Comments-only properties should be handled");
    assert!(result?.properties.is_empty());
    
    // Test 3: Properties with special characters
    let special_props = test_dir.join("special.properties");
    fs::write(&special_props, 
        r#"
        # Special characters in keys and values
        key.with.dots=value
        key_with_underscores=value_with_underscores
        key-with-dashes=value-with-dashes
        key with spaces=value with spaces
        unicode.key=Значение
        url=https://example.com/path?param=value&another=test
        escaped.value=This is a \= test with \: escaped chars
        multiline.value=Line 1 \
                        Line 2 \
                        Line 3
        "#
    )?;
    let result = parser.parse_file(&special_props);
    assert!(result.is_ok(), "Properties with special characters should be handled");
    let props = result?.properties;
    assert!(!props.is_empty());
    
    // Test 4: Properties with no values
    let no_values = test_dir.join("no_values.properties");
    fs::write(&no_values, 
        r#"
        key1=
        key2 = 
        key3=
        "#
    )?;
    let result = parser.parse_file(&no_values);
    assert!(result.is_ok(), "Properties with empty values should be handled");
    let props = result?.properties;
    assert_eq!(props.len(), 3);
    
    // Test 5: Properties with equals in value
    let equals_in_value = test_dir.join("equals.properties");
    fs::write(&equals_in_value, 
        r#"
        database.url=jdbc:mysql://localhost:3306/test?user=admin&password=secret
        equation=a=b+c
        url=http://example.com/path?param1=value1&param2=value2
        "#
    )?;
    let result = parser.parse_file(&equals_in_value);
    assert!(result.is_ok(), "Properties with equals in values should be handled");
    
    // Test 6: Properties with leading/trailing spaces
    let spaces_props = test_dir.join("spaces.properties");
    fs::write(&spaces_props, 
        r#"
        key1 = value1
        key2= value2
        key3 =value3
        key4 = value4 with spaces
        key5 = value5 
        "#
    )?;
    let result = parser.parse_file(&spaces_props);
    assert!(result.is_ok(), "Properties with spaces should be handled");
    let props = result?.properties;
    assert_eq!(props.len(), 5);
    
    // Test 7: Properties with duplicate keys
    let duplicate_keys = test_dir.join("duplicates.properties");
    fs::write(&duplicate_keys, 
        r#"
        key1 = value1
        key2 = value2
        key1 = new_value1
        key3 = value3
        key2 = new_value2
        "#
    )?;
    let result = parser.parse_file(&duplicate_keys);
    assert!(result.is_ok(), "Properties with duplicate keys should be handled");
    let props = result?.properties;
    // Should keep all entries, including duplicates
    assert!(props.len() >= 3);
    
    // Test 8: Properties with very long values
    let long_values = test_dir.join("long.properties");
    let long_value = "x".repeat(1000);
    fs::write(&long_values, format!("long.key={}", long_value))?;
    let result = parser.parse_file(&long_values);
    assert!(result.is_ok(), "Properties with long values should be handled");
    
    // Test 9: Properties with escaped unicode
    let unicode_props = test_dir.join("unicode.properties");
    fs::write(&unicode_props, 
        r#"
        russian=\u041f\u0440\u0438\u0432\u0435\u0442
        chinese=\u4f60\u597d
        arabic=\u0645\u0631\u062d\u0628\u0627
        emoji=\uD83D\uDE00\uD83D\uDE01\uD83D\uDE02
        "#
    )?;
    let result = parser.parse_file(&unicode_props);
    assert!(result.is_ok(), "Properties with escaped unicode should be handled");
    
    Ok(())
}