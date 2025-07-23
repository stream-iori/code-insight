use anyhow::Result;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono;

use crate::types::{Declaration, LlmExport, DeclarationKind};
use crate::query::QueryEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub query: Option<String>,
    pub kind: Option<DeclarationKind>,
    pub annotations: Vec<String>,
    pub package: Option<String>,
    pub limit: Option<usize>,
    pub include_source: bool,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Jsonl,
    Markdown,
    LlamaIndex,
    RAG,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub declarations: Vec<LlmExport>,
    pub metadata: ExportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub total_count: usize,
    pub query: LlmRequest,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub project_root: String,
}

pub struct LlmExporter {
    query_engine: QueryEngine,
    project_root: PathBuf,
}

impl LlmExporter {
    pub fn new(query_engine: QueryEngine, project_root: PathBuf) -> Result<Self> {
        Ok(Self {
            query_engine,
            project_root,
        })
    }

    pub async fn export(&self, request: LlmRequest) -> Result<LlmResponse> {
        let declarations = self.find_declarations(&request).await?;
        let exports = self.convert_to_exports(declarations, &request).await?;

        let metadata = ExportMetadata {
            total_count: exports.len(),
            query: request.clone(),
            exported_at: chrono::Utc::now(),
            project_root: self.project_root.to_string_lossy().to_string(),
        };

        Ok(LlmResponse {
            declarations: exports,
            metadata,
        })
    }

    async fn find_declarations(
        &self,
        request: &LlmRequest,
    ) -> Result<Vec<crate::types::SearchResult>> {
        let query = crate::types::SearchQuery {
            query: request.query.clone().unwrap_or_default(),
            kind: crate::types::SearchKind::Exact,
            filters: self.build_filters(request),
            limit: request.limit,
        };

        self.query_engine.search(&query).await
    }

    fn build_filters(&self, request: &LlmRequest) -> Vec<crate::types::SearchFilter> {
        let mut filters = Vec::new();

        if let Some(kind) = &request.kind {
            filters.push(crate::types::SearchFilter::Kind(kind.clone()));
        }

        for annotation in &request.annotations {
            filters.push(crate::types::SearchFilter::Annotation(annotation.clone()));
        }

        if let Some(package) = &request.package {
            filters.push(crate::types::SearchFilter::Package(package.clone()));
        }

        filters
    }

    async fn convert_to_exports(
        &self,
        search_results: Vec<crate::types::SearchResult>,
        request: &LlmRequest,
    ) -> Result<Vec<LlmExport>> {
        let mut exports = Vec::new();

        for result in search_results {
            let export = self.create_export(&result.declaration, &result.file_path, request).await?;
            exports.push(export);
        }

        Ok(exports)
    }

    async fn create_export(
        &self,
        declaration: &Declaration,
        file_path: &PathBuf,
        request: &LlmRequest,
    ) -> Result<LlmExport> {
        let relative_path = Self::get_relative_path(file_path, &self.project_root)?;
        
        let code = if request.include_source {
            self.extract_source_code(file_path, &declaration.range).await?
        } else {
            declaration.signature.clone()
        };

        Ok(LlmExport {
            name: declaration.name.clone(),
            kind: format!("{:?}", declaration.kind).to_lowercase(),
            signature: declaration.signature.clone(),
            documentation: declaration.documentation.clone(),
            code,
            file_path: relative_path,
            line_range: (
                declaration.range.start_line,
                declaration.range.end_line,
            ),
        })
    }

    async fn extract_source_code(
        &self,
        file_path: &PathBuf,
        range: &crate::types::SourceRange,
    ) -> Result<String> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let lines: Vec<&str> = content.lines().collect();
        
        let start = range.start_line.saturating_sub(1);
        let end = range.end_line.min(lines.len());
        
        let extracted: Vec<&str> = lines[start..end].to_vec();
        Ok(extracted.join("\n"))
    }

    fn get_relative_path(
        path: &PathBuf,
        root: &PathBuf,
    ) -> Result<String> {
        let relative = path.strip_prefix(root)
            .unwrap_or(path);
        Ok(relative.to_string_lossy().to_string())
    }

    pub fn format_export(&self, response: &LlmResponse, format: &ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Json => self.format_json(response),
            ExportFormat::Jsonl => self.format_jsonl(response),
            ExportFormat::Markdown => self.format_markdown(response),
            ExportFormat::LlamaIndex => self.format_llama_index(response),
            ExportFormat::RAG => self.format_rag(response),
        }
    }

    fn format_json(&self, response: &LlmResponse) -> Result<String> {
        Ok(serde_json::to_string_pretty(response)?)
    }

    fn format_jsonl(&self, response: &LlmResponse) -> Result<String> {
        let mut lines = Vec::new();
        for declaration in &response.declarations {
            lines.push(serde_json::to_string(declaration)?);
        }
        Ok(lines.join("\n"))
    }

    fn format_markdown(&self, response: &LlmResponse) -> Result<String> {
        let mut markdown = String::new();
        
        markdown.push_str(&format!("# Code Export\n\n"));
        markdown.push_str(&format!("**Total declarations:** {}\n\n", response.metadata.total_count));
        markdown.push_str(&format!("**Exported at:** {}\n\n", response.metadata.exported_at.format("%Y-%m-%d %H:%M:%S UTC")));
        markdown.push_str(&format!("**Project root:** {}\n\n", response.metadata.project_root));

        for declaration in &response.declarations {
            markdown.push_str(&format!("## {}\n\n", declaration.name));
            markdown.push_str(&format!("**Type:** {}\n\n", declaration.kind));
            markdown.push_str(&format!("**File:** {} (lines {}-{})\n\n", 
                declaration.file_path, 
                declaration.line_range.0, 
                declaration.line_range.1));
            
            if let Some(doc) = &declaration.documentation {
                markdown.push_str(&format!("**Documentation:**\n```\n{}\n```\n\n", doc));
            }

            markdown.push_str(&format!("**Signature:**\n```java\n{}\n```\n\n", declaration.signature));
            markdown.push_str(&format!("**Code:**\n```java\n{}\n```\n\n", declaration.code));
            markdown.push_str("---\n\n");
        }

        Ok(markdown)
    }

    fn format_llama_index(&self, response: &LlmResponse) -> Result<String> {
        let mut llama_docs = Vec::new();
        
        for declaration in &response.declarations {
            let document = LlamIndexDocument {
                id: format!("{}: {}", declaration.file_path, declaration.name),
                text: format!("{}\n\n{}", declaration.signature, declaration.code),
                metadata: LlamIndexMetadata {
                    name: declaration.name.clone(),
                    kind: declaration.kind.clone(),
                    file_path: declaration.file_path.clone(),
                    line_range: declaration.line_range,
                    documentation: declaration.documentation.clone(),
                },
            };
            llama_docs.push(document);
        }

        Ok(serde_json::to_string_pretty(&llama_docs)?)
    }

    fn format_rag(&self, response: &LlmResponse) -> Result<String> {
        let mut chunks = Vec::new();
        
        for declaration in &response.declarations {
            let chunk = RagChunk {
                content: format!("{}\n\n{}", declaration.signature, declaration.code),
                metadata: RagMetadata {
                    source: declaration.file_path.clone(),
                    name: declaration.name.clone(),
                    kind: declaration.kind.clone(),
                    line_range: declaration.line_range,
                    documentation: declaration.documentation.clone(),
                    chunk_type: "declaration".to_string(),
                },
            };
            chunks.push(chunk);
        }

        Ok(serde_json::to_string_pretty(&chunks)?)
    }

    pub async fn export_to_file(
        &self,
        request: LlmRequest,
        output_path: &PathBuf,
    ) -> Result<()> {
        let response = self.export(request.clone()).await?;
        let formatted = self.format_export(&response, &request.format)?;
        
        tokio::fs::write(output_path, formatted).await?;
        Ok(())
    }

    pub async fn export_service_classes(&self, limit: Option<usize>) -> Result<LlmResponse> {
        let request = LlmRequest {
            query: None,
            kind: Some(DeclarationKind::Class),
            annotations: vec!["Service".to_string(), "Component".to_string()],
            package: None,
            limit,
            include_source: true,
            format: ExportFormat::Json,
        };
        
        self.export(request).await
    }

    pub async fn export_interfaces(&self, limit: Option<usize>) -> Result<LlmResponse> {
        let request = LlmRequest {
            query: None,
            kind: Some(DeclarationKind::Interface),
            annotations: vec![],
            package: None,
            limit,
            include_source: true,
            format: ExportFormat::Json,
        };
        
        self.export(request).await
    }

    pub async fn export_controllers(&self, limit: Option<usize>) -> Result<LlmResponse> {
        let request = LlmRequest {
            query: None,
            kind: Some(DeclarationKind::Class),
            annotations: vec!["Controller".to_string(), "RestController".to_string()],
            package: None,
            limit,
            include_source: true,
            format: ExportFormat::Json,
        };
        
        self.export(request).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlamIndexDocument {
    id: String,
    text: String,
    metadata: LlamIndexMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlamIndexMetadata {
    name: String,
    kind: String,
    file_path: String,
    line_range: (usize, usize),
    documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RagChunk {
    content: String,
    metadata: RagMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RagMetadata {
    source: String,
    name: String,
    kind: String,
    line_range: (usize, usize),
    documentation: Option<String>,
    chunk_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_llm_exporter() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("test_index");
        let query_engine = crate::query::QueryEngine::new(&index_path).unwrap();
        
        let exporter = LlmExporter::new(query_engine, dir.path().to_path_buf()).unwrap();
        
        let request = LlmRequest {
            query: Some("test".to_string()),
            kind: None,
            annotations: vec![],
            package: None,
            limit: Some(10),
            include_source: false,
            format: ExportFormat::Json,
        };

        let response = exporter.export(request).await.unwrap();
        assert_eq!(response.declarations.len(), 0);
        assert_eq!(response.metadata.total_count, 0);
    }

    #[tokio::test]
    async fn test_format_export() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("test_index");
        let query_engine = crate::query::QueryEngine::new(&index_path).unwrap();
        
        let exporter = LlmExporter::new(query_engine, dir.path().to_path_buf()).unwrap();

        let response = LlmResponse {
            declarations: vec![],
            metadata: ExportMetadata {
                total_count: 0,
                query: LlmRequest {
                    query: None,
                    kind: None,
                    annotations: vec![],
                    package: None,
                    limit: None,
                    include_source: false,
                    format: ExportFormat::Json,
                },
                exported_at: chrono::Utc::now(),
                project_root: "/test".to_string(),
            },
        };

        let json = exporter.format_export(&response, &ExportFormat::Json).unwrap();
        assert!(json.contains("\"declarations\""));
        assert!(json.contains("\"metadata\""));
    }

    #[tokio::test]
    async fn test_format_markdown() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("test_index");
        let query_engine = crate::query::QueryEngine::new(&index_path).unwrap();
        
        let exporter = LlmExporter::new(query_engine, dir.path().to_path_buf()).unwrap();

        let response = LlmResponse {
            declarations: vec![LlmExport {
                name: "TestClass".to_string(),
                kind: "class".to_string(),
                signature: "public class TestClass".to_string(),
                documentation: Some("Test documentation".to_string()),
                code: "public class TestClass {}".to_string(),
                file_path: "TestClass.java".to_string(),
                line_range: (1, 3),
            }],
            metadata: ExportMetadata {
                total_count: 1,
                query: LlmRequest {
                    query: None,
                    kind: None,
                    annotations: vec![],
                    package: None,
                    limit: None,
                    include_source: false,
                    format: ExportFormat::Markdown,
                },
                exported_at: chrono::Utc::now(),
                project_root: "/test".to_string(),
            },
        };

        let markdown = exporter.format_export(&response, &ExportFormat::Markdown).unwrap();
        assert!(markdown.contains("# Code Export"));
        assert!(markdown.contains("TestClass"));
        assert!(markdown.contains("Test documentation"));
    }
}