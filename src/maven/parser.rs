use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

use crate::types::{MavenDependency, MavenModule};

pub struct MavenParser;

impl MavenParser {
    pub fn parse_pom_file(&self, path: &Path) -> Result<MavenModule> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read POM file: {:?}", path))?;
        
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut module = MavenModule {
            group_id: String::new(),
            artifact_id: String::new(),
            version: String::new(),
            packaging: None,
            path: path.to_path_buf(),
            dependencies: Vec::new(),
            submodules: Vec::new(),
        };
        
        let mut current_element = String::new();
        let mut current_dependency: Option<MavenDependency> = None;
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current_element = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let tag_name = &current_element;
                    
                    if tag_name == "dependency" {
                        current_dependency = Some(MavenDependency {
                            group_id: String::new(),
                            artifact_id: String::new(),
                            version: String::new(),
                            scope: None,
                            optional: false,
                        });
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default();
                    let tag_name = &current_element;
                    
                    if let Some(dep) = &mut current_dependency {
                        match tag_name.as_ref() {
                            "groupId" => dep.group_id = text.to_string(),
                            "artifactId" => dep.artifact_id = text.to_string(),
                            "version" => dep.version = text.to_string(),
                            "scope" => dep.scope = Some(text.to_string()),
                            "optional" => dep.optional = text == "true",
                            _ => {}
                        }
                    } else {
                        match tag_name.as_ref() {
                            "groupId" => module.group_id = text.to_string(),
                            "artifactId" => module.artifact_id = text.to_string(),
                            "version" => module.version = text.to_string(),
                            "packaging" => module.packaging = Some(text.to_string()),
                            "module" => module.submodules.push(text.to_string()),
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name == "dependency" {
                        if let Some(dep) = current_dependency.take() {
                            if !dep.group_id.is_empty() && !dep.artifact_id.is_empty() {
                                module.dependencies.push(dep);
                            }
                        }
                    }
                    current_element.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("XML parsing error: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        
        if module.group_id.is_empty() || module.artifact_id.is_empty() {
            return Err(anyhow::anyhow!("Invalid POM file: missing groupId or artifactId"));
        }
        
        Ok(module)
    }
    
    pub fn find_maven_modules(&self, root_path: &Path) -> Result<Vec<MavenModule>> {
        let mut modules = Vec::new();
        
        for entry in walkdir::WalkDir::new(root_path) {
            let entry = entry?;
            if entry.file_name() == "pom.xml" {
                let pom_path = entry.path();
                if let Ok(module) = self.parse_pom_file(pom_path) {
                    modules.push(module);
                }
            }
        }
        
        Ok(modules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_parse_pom_file() {
        let parser = MavenParser;
        let pom_content = r#"
            <project>
                <groupId>com.example</groupId>
                <artifactId>my-app</artifactId>
                <version>1.0.0</version>
                <packaging>jar</packaging>
                <dependencies>
                    <dependency>
                        <groupId>org.springframework</groupId>
                        <artifactId>spring-core</artifactId>
                        <version>5.3.0</version>
                    </dependency>
                </dependencies>
            </project>
        "#;
        
        let dir = tempdir().unwrap();
        let pom_path = dir.path().join("pom.xml");
        std::fs::write(&pom_path, pom_content).unwrap();
        
        let module = parser.parse_pom_file(&pom_path).unwrap();
        
        assert_eq!(module.group_id, "com.example");
        assert_eq!(module.artifact_id, "my-app");
        assert_eq!(module.version, "1.0.0");
        assert_eq!(module.packaging, Some("jar".to_string()));
        assert_eq!(module.dependencies.len(), 1);
        assert_eq!(module.dependencies[0].group_id, "org.springframework");
    }
}