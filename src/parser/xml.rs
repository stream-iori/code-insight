use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

use crate::types::XmlFile;

pub struct XmlParser;

impl XmlParser {
    pub fn parse_file(&self, path: &Path) -> Result<XmlFile> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read XML file: {:?}", path))?;
        
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut root_element = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    root_element = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                    break;
                }
                Ok(Event::Empty(ref e)) => {
                    root_element = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(XmlFile {
            path: path.to_path_buf(),
            root_element: root_element.unwrap_or_else(|| "unknown".to_string()),
            content: content.clone(),
        })
    }
    
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
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name != "properties" && tag_name != "project" {
                        current_key = Some(tag_name);
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
    fn test_parse_xml_file() {
        let parser = XmlParser;
        let xml_content = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <beans xmlns="http://www.springframework.org/schema/beans">
                <bean id="userService" class="com.example.UserService"/>
            </beans>
        "#;
        
        let dir = tempdir().unwrap();
        let xml_path = dir.path().join("application.xml");
        std::fs::write(&xml_path, xml_content).unwrap();
        
        let xml_file = parser.parse_file(&xml_path).unwrap();
        
        assert_eq!(xml_file.root_element, "beans");
        assert!(xml_file.content.contains("UserService"));
    }
    
    #[test]
    fn test_extract_spring_beans() {
        let parser = XmlParser;
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
        let parser = XmlParser;
        let xml_content = r#"
            <project>
                <properties>
                    <maven.compiler.source>11</maven.compiler.source>
                    <maven.compiler.target>11</maven.compiler.target>
                </properties>
            </project>
        "#;
        
        let properties = parser.extract_maven_properties(xml_content).unwrap();
        
        assert_eq!(properties.len(), 2);
        assert!(properties.contains(&("maven.compiler.source".to_string(), "11".to_string())));
        assert!(properties.contains(&("maven.compiler.target".to_string(), "11".to_string())));
    }
}