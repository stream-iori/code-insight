mod java;
mod properties;
mod xml;

pub use java::*;
pub use properties::*;
pub use xml::*;

use crate::types::{JavaFile, PropertiesFile, XmlFile};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct FileParser;

impl FileParser {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    //fixme:下面三个方法可以只用一个方法,然后传入一个enum

    pub fn parse_java_file(&self, path: &Path) -> Result<JavaFile> {
        let mut parser = JavaParser::new()?;
        parser.parse_file(path)
    }

    pub fn parse_xml_file(&self, path: &Path) -> Result<XmlFile> {
        let parser = XmlParser;
        parser.parse_file(path)
    }

    pub fn parse_properties_file(&self, path: &Path) -> Result<PropertiesFile> {
        let parser = PropertiesParser;
        parser.parse_file(path)
    }

    ///find files that java project cared.
    pub fn find_source_files(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(root) {
            //shadow variable for exception
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                match path.extension().and_then(|ext| ext.to_str()) {
                    //TODO: should enum file type instead of string
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
