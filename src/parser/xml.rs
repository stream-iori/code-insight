use crate::parser::{FileMeta, FileParseable, FileSuffix};
use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct XmlFileParser;

impl XmlFileParser {
    pub fn new() -> Self {
        XmlFileParser
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlSourceFile {
    pub file_meta: FileMeta,
    pub root_element: String,
    pub content: String,
    pub spring_beans: Vec<String>,
    pub maven_properties: Vec<(String, String)>,
}

impl FileParseable<XmlSourceFile> for XmlFileParser {
    fn parse_file(&mut self, path: &Path) -> Result<XmlSourceFile> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read XML file: {:?}", path))?;
        
        let mut xml_file = XmlSourceFile {
            file_meta: FileMeta::new(path, FileSuffix::Xml, content.as_str()),
            root_element: "unknown".to_string(),
            content: content.clone(),
            spring_beans: Vec::new(),
            maven_properties: Vec::new(),
        };

        // Parse XML structure
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    xml_file.root_element = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        // Extract additional information
        xml_file.spring_beans = self.extract_spring_beans(&content)?;
        xml_file.maven_properties = self.extract_maven_properties(&content)?;

        Ok(xml_file)
    }
}

/// Main XML parser that follows the java.rs pattern
pub struct XmlParser {
    inner: XmlFileParser,
}

impl XmlParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: XmlFileParser::new(),
        })
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<XmlSourceFile> {
        self.inner.parse_file(path)
    }
}

impl XmlFileParser {
    pub fn extract_spring_beans(&self, xml_content: &str) -> Result<Vec<String>> {
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        
        let mut beans = Vec::new();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name == "bean" || tag_name.ends_with(":bean") {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                if key == "class" {
                                    let value = attr.decode_and_unescape_value(&reader)?;
                                    beans.push(value.to_string());
                                }
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(beans)
    }
    
    pub fn extract_maven_properties(&self, xml_content: &str) -> Result<Vec<(String, String)>> {
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);
        
        let mut properties = Vec::new();
        let mut buf = Vec::new();
        let mut current_key = None;
        let mut in_properties = false;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name == "properties" {
                        in_properties = true;
                    } else if in_properties && tag_name != "properties" {
                        current_key = Some(tag_name);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name == "properties" {
                        in_properties = false;
                    }
                }
                Ok(Event::Text(e)) => {
                    if let Some(key) = current_key.take() {
                        let value = e.unescape().unwrap_or_default();
                        properties.push((key, value.to_string()));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(properties)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_parse_xml_source_file() {
        let mut parser = XmlParser::new().unwrap();
        let xml_content = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <beans xmlns="http://www.springframework.org/schema/beans">
                <bean id="userService" class="com.example.UserService"/>
                <bean id="userRepository" class="com.example.UserRepository"/>
            </beans>
        "#;
        
        let dir = tempdir().unwrap();
        let xml_path = dir.path().join("application.xml");
        std::fs::write(&xml_path, xml_content).unwrap();
        
        let xml_file = parser.parse_file(&xml_path).unwrap();
        
        assert_eq!(xml_file.file_meta.name, "application.xml");
        assert_eq!(xml_file.root_element, "beans");
        assert!(xml_file.content.contains("UserService"));
        assert_eq!(xml_file.spring_beans.len(), 2);
        assert!(xml_file.spring_beans.contains(&"com.example.UserService".to_string()));
        assert!(xml_file.spring_beans.contains(&"com.example.UserRepository".to_string()));
    }
    
    #[test]
    fn test_parse_maven_pom() {
        let mut parser = XmlParser::new().unwrap();
        let xml_content = r#"
            <project xmlns="http://maven.apache.org/POM/4.0.0">
                <properties>
                    <maven.compiler.source>11</maven.compiler.source>
                    <maven.compiler.target>11</maven.compiler.target>
                    <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
                </properties>
            </project>
        "#;
        
        let dir = tempdir().unwrap();
        let xml_path = dir.path().join("pom.xml");
        std::fs::write(&xml_path, xml_content).unwrap();
        
        let xml_file = parser.parse_file(&xml_path).unwrap();
        
        assert_eq!(xml_file.file_meta.name, "pom.xml");
        assert_eq!(xml_file.root_element, "project");
        assert_eq!(xml_file.maven_properties.len(), 3);
        assert!(xml_file.maven_properties.contains(&("maven.compiler.source".to_string(), "11".to_string())));
        assert!(xml_file.maven_properties.contains(&("maven.compiler.target".to_string(), "11".to_string())));
    }
    
    #[test]
    fn test_extract_spring_beans() {
        let parser = XmlFileParser::new();
        let xml_content = r#"
            <beans>
                <bean id="service1" class="com.example.Service1"/>
                <bean id="service2" class="com.example.Service2"/>
            </beans>
        "#;
        
        let beans = parser.extract_spring_beans(xml_content).unwrap();
        
        assert_eq!(beans.len(), 2);
        assert!(beans.contains(&"com.example.Service1".to_string()));
        assert!(beans.contains(&"com.example.Service2".to_string()));
    }
    
    #[test]
    fn test_extract_maven_properties() {
        let parser = XmlFileParser::new();
        let xml_content = r#"
            <project>
                <properties>
                    <version>1.0.0</version>
                    <name>test-project</name>
                </properties>
            </project>
        "#;
        
        let properties = parser.extract_maven_properties(xml_content).unwrap();
        
        assert_eq!(properties.len(), 2);
        assert!(properties.contains(&("version".to_string(), "1.0.0".to_string())));
        assert!(properties.contains(&("name".to_string(), "test-project".to_string())));
    }
}