mod java_structure;

pub use java_structure::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub struct FileParser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSuffix {
    Java,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub path: PathBuf,
    pub name: String,
    pub suffix: FileSuffix,
    pub hash_value: String,
}

impl FileMeta {
    pub fn new(path: &Path, suffix: FileSuffix, source: &str) -> Self {
        let hash_value = format!("{:x}", md5::compute(source));
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        FileMeta {
            path: path.to_path_buf(),
            name,
            suffix,
            hash_value,
        }
    }
}

pub trait FileParseable<T> {
    ///Parse file with some definition
    fn parse_file(&mut self, path: &Path) -> Result<T>;
}

impl FileParser {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn parse_java_structure(&self, path: &Path) -> Result<JavaStructurePreview> {
        let mut parser = JavaStructureParser::new()?;
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
                    Some("java") => {
                        files.push(path.to_path_buf());
                    }
                    _ => {}
                }
            }
        }

        Ok(files)
    }
}
