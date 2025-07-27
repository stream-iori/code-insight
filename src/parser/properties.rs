use crate::parser::{FileMeta, FileParseable, FileSuffix};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct PropertiesFileParser;

impl PropertiesFileParser {
    pub fn new() -> Self {
        PropertiesFileParser
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertiesSourceFile {
    pub file_meta: FileMeta,
    pub properties: Vec<(String, String)>,
    pub properties_map: HashMap<String, String>,
    pub comments: Vec<String>,
}

impl FileParseable<PropertiesSourceFile> for PropertiesFileParser {
    fn parse_file(&mut self, path: &Path) -> Result<PropertiesSourceFile> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read properties file: {:?}", path))?;
        
        let (properties, comments) = self.parse_content_with_comments(&content)?;
        let properties_map: HashMap<String, String> = properties.iter()
            .cloned()
            .collect();

        Ok(PropertiesSourceFile {
            file_meta: FileMeta::new(path, FileSuffix::Property, content.as_str()),
            properties: properties.clone(),
            properties_map,
            comments,
        })
    }
}

/// Main Properties parser that follows the java.rs pattern
pub struct PropertiesParser {
    inner: PropertiesFileParser,
}

impl PropertiesParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: PropertiesFileParser::new(),
        })
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<PropertiesSourceFile> {
        self.inner.parse_file(path)
    }
}

impl PropertiesFileParser {
    pub fn parse_content(&self, content: &str) -> Result<Vec<(String, String)>> {
        let (properties, _) = self.parse_content_with_comments(content)?;
        Ok(properties)
    }

    pub fn parse_content_with_comments(&self, content: &str) -> Result<(Vec<(String, String)>, Vec<String>)> {
        let mut properties = Vec::new();
        let mut comments = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines
            if line.is_empty() {
                continue;
            }
            
            // Collect comments
            if line.starts_with('#') || line.starts_with('!') {
                comments.push(line.to_string());
                continue;
            }
            
            // Parse key=value pairs
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();
                
                // Handle escaped characters
                let value = self.unescape_properties(value);
                
                properties.push((key.to_string(), value));
            }
        }
        
        Ok((properties, comments))
    }
    
    pub fn parse_to_map(&self, content: &str) -> Result<HashMap<String, String>> {
        let properties = self.parse_content(content)?;
        Ok(properties.into_iter().collect())
    }
    
    pub fn get_property(&self, content: &str, key: &str) -> Option<String> {
        self.parse_content(content)
            .ok()
            .and_then(|props| {
                props.into_iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v)
            })
    }
    
    fn unescape_properties(&self, value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars();
        
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some(c) => result.push(c),
                    None => break,
                }
            } else {
                result.push(c);
            }
        }
        
        result
    }
    
    pub fn merge_properties(&self, files: &[PropertiesSourceFile]) -> HashMap<String, String> {
        let mut merged = HashMap::new();
        
        for file in files {
            for (key, value) in &file.properties {
                merged.insert(key.clone(), value.clone());
            }
        }
        
        merged
    }
    
    pub fn find_properties_files(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for entry in walkdir::WalkDir::new(root) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                match path.extension()
                    .and_then(|ext| ext.to_str()) {
                    Some("properties") => {
                        files.push(path.to_path_buf());
                    }
                    _ => {}
                }
            }
        }
        
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_parse_properties_source_file() {
        let mut parser = PropertiesParser::new().unwrap();
        let props_content = r#"
            # Database configuration
            database.url=jdbc:mysql://localhost:3306/mydb
            database.username=admin
            database.password=secret
            
            # Application settings
            app.name=My Application
            app.version=1.0.0
        "#;
        
        let dir = tempdir().unwrap();
        let props_path = dir.path().join("application.properties");
        std::fs::write(&props_path, props_content).unwrap();
        
        let props_file = parser.parse_file(&props_path).unwrap();
        
        assert_eq!(props_file.file_meta.name, "application.properties");
        assert_eq!(props_file.properties.len(), 5);
        assert!(props_file.properties.contains(&("database.url".to_string(), "jdbc:mysql://localhost:3306/mydb".to_string())));
        assert!(props_file.properties.contains(&("app.name".to_string(), "My Application".to_string())));
        assert_eq!(props_file.comments.len(), 2);
        assert_eq!(props_file.properties_map["database.username"], "admin");
    }
    
    #[test]
    fn test_parse_properties_content() {
        let parser = PropertiesFileParser::new();
        let content = r#"
            key1=value1
            key2=value2
            key3=value with spaces
        "#;
        
        let (properties, comments) = parser.parse_content_with_comments(content).unwrap();
        
        assert_eq!(properties.len(), 3);
        assert_eq!(properties[0], ("key1".to_string(), "value1".to_string()));
        assert_eq!(properties[1], ("key2".to_string(), "value2".to_string()));
        assert_eq!(properties[2], ("key3".to_string(), "value with spaces".to_string()));
        assert_eq!(comments.len(), 0);
    }
    
    #[test]
    fn test_unescape_properties() {
        let parser = PropertiesFileParser::new();
        
        assert_eq!(parser.unescape_properties("line\\nbreak"), "line\nbreak");
        assert_eq!(parser.unescape_properties("tab\\there"), "tab\there");
        assert_eq!(parser.unescape_properties("backslash\\\\here"), "backslash\\here");
    }
    
    #[test]
    fn test_get_property() {
        let parser = PropertiesFileParser::new();
        let content = r#"
            app.name=MyApp
            app.version=1.0.0
        "#;
        
        assert_eq!(parser.get_property(content, "app.name"), Some("MyApp".to_string()));
        assert_eq!(parser.get_property(content, "nonexistent"), None);
    }
    
    #[test]
    fn test_merge_properties() {
        let parser = PropertiesFileParser::new();
        
        let file1 = PropertiesSourceFile {
            file_meta: FileMeta::new(Path::new("file1.properties"), FileSuffix::Property, ""),
            properties: vec![
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ],
            properties_map: HashMap::new(),
            comments: vec![],
        };
        
        let file2 = PropertiesSourceFile {
            file_meta: FileMeta::new(Path::new("file2.properties"), FileSuffix::Property, ""),
            properties: vec![
                ("key2".to_string(), "new_value2".to_string()),
                ("key3".to_string(), "value3".to_string()),
            ],
            properties_map: HashMap::new(),
            comments: vec![],
        };
        
        let merged = parser.merge_properties(&[file1, file2]);
        
        assert_eq!(merged.len(), 3);
        assert_eq!(merged["key1"], "value1");
        assert_eq!(merged["key2"], "new_value2"); // Second file overrides
        assert_eq!(merged["key3"], "value3");
    }
    
    #[test]
    fn test_comments_extraction() {
        let parser = PropertiesFileParser::new();
        let content = r#"
            # This is a comment
            key1=value1
            ! Another comment
            key2=value2
            # Final comment
        "#;
        
        let (properties, comments) = parser.parse_content_with_comments(content).unwrap();
        
        assert_eq!(properties.len(), 2);
        assert_eq!(comments.len(), 3);
        assert!(comments.contains(&"# This is a comment".to_string()));
        assert!(comments.contains(&"! Another comment".to_string()));
        assert!(comments.contains(&"# Final comment".to_string()));
    }
}