mod java;
mod xml;
mod properties;

pub use java::*;
pub use xml::*;
pub use properties::*;

use anyhow::Result;
use std::path::{Path, PathBuf};
use crate::types::{JavaFile, XmlFile, PropertiesFile};

pub struct FileParser;

impl FileParser {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn parse_java_file(&self, path: &Path
    ) -> Result<JavaFile> {
        let mut parser = JavaParser::new()?;
        parser.parse_file(path)
    }
    
    pub fn parse_xml_file(&self, path: &Path
    ) -> Result<XmlFile> {
        let parser = XmlParser;
        parser.parse_file(path)
    }
    
    pub fn parse_properties_file(&self, path: &Path
    ) -> Result<PropertiesFile> {
        let parser = PropertiesParser;
        parser.parse_file(path)
    }
    
    pub fn find_source_files(&self, root: &Path
    ) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for entry in walkdir::WalkDir::new(root) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                match path.extension()
                    .and_then(|ext| ext.to_str()) {
                    Some("java") | Some("xml") | Some("properties") => {
                        files.push(path.to_path_buf());
                    }
                    _ => {}
                }
            }
        }
        
        Ok(files)
    }
}