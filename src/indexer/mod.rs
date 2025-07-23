use anyhow::{Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tantivy::{
    collector::TopDocs,
    query::{Query, QueryParser, FuzzyTermQuery},
    schema::*,
    TantivyDocument,
    Index, IndexReader, IndexWriter, Searcher, Term,
};
use tokio::sync::RwLock;

use crate::types::{
    Declaration, DeclarationKind, Field, JavaFile, Method, SearchQuery, SearchResult,
};

pub struct IndexManager {
    index: Index,
    reader: IndexReader,
    writer: Arc<RwLock<IndexWriter>>,
    schema: Schema,
}

impl IndexManager {
    pub fn new(index_path: &Path) -> Result<Self> {
        let schema = Self::create_schema()?;
        
        let index = if index_path.exists() {
            Index::open_in_dir(index_path)?
        } else {
            std::fs::create_dir_all(index_path)?;
            Index::create_in_dir(index_path, schema.clone())?
        };

        let reader = index
            .reader_builder()
            .try_into()?;

        let writer = Arc::new(RwLock::new(
            index.writer(50_000_000)? // 50MB heap
        ));

        Ok(Self {
            index,
            reader,
            writer,
            schema,
        })
    }

    fn create_schema() -> Result<Schema> {
        let mut schema_builder = Schema::builder();

        // Basic fields
        schema_builder.add_text_field("name", TEXT | STORED);
        schema_builder.add_text_field("package", TEXT | STORED);
        schema_builder.add_text_field("file_path", STORED);
        schema_builder.add_text_field("signature", TEXT | STORED);
        schema_builder.add_text_field("documentation", TEXT | STORED);

        // Kind field (for exact matching)
        schema_builder.add_text_field("kind", TEXT | STORED);

        // Modifiers and annotations
        schema_builder.add_text_field("modifiers", TEXT | STORED);
        schema_builder.add_text_field("annotations", TEXT | STORED);

        // Inheritance
        schema_builder.add_text_field("extends", TEXT | STORED);
        schema_builder.add_text_field("implements", TEXT | STORED);

        // Fields and methods (as JSON)
        schema_builder.add_text_field("fields", STORED);
        schema_builder.add_text_field("methods", STORED);

        // Source location
        schema_builder.add_u64_field("start_line", STORED);
        schema_builder.add_u64_field("end_line", STORED);
        schema_builder.add_u64_field("start_column", STORED);
        schema_builder.add_u64_field("end_column", STORED);

        // Hash for deduplication
        schema_builder.add_text_field("source_hash", STRING | STORED);

        Ok(schema_builder.build())
    }

    pub async fn index_java_file(&self, java_file: &JavaFile) -> Result<()> {
        let mut writer = self.writer.write().await;
        
        for declaration in &java_file.declarations {
            let doc = self.create_document(declaration, java_file)?;
            writer.add_document(doc)?;
        }

        writer.commit()?;
        Ok(())
    }

    fn create_document(&self, declaration: &Declaration, java_file: &JavaFile) -> Result<TantivyDocument> {
        let schema = &self.schema;
        
        let name_field = schema.get_field("name").unwrap();
        let package_field = schema.get_field("package").unwrap();
        let file_path_field = schema.get_field("file_path").unwrap();
        let signature_field = schema.get_field("signature").unwrap();
        let documentation_field = schema.get_field("documentation").unwrap();
        let kind_field = schema.get_field("kind").unwrap();
        let modifiers_field = schema.get_field("modifiers").unwrap();
        let annotations_field = schema.get_field("annotations").unwrap();
        let extends_field = schema.get_field("extends").unwrap();
        let implements_field = schema.get_field("implements").unwrap();
        let fields_field = schema.get_field("fields").unwrap();
        let methods_field = schema.get_field("methods").unwrap();
        let start_line_field = schema.get_field("start_line").unwrap();
        let end_line_field = schema.get_field("end_line").unwrap();
        let start_column_field = schema.get_field("start_column").unwrap();
        let end_column_field = schema.get_field("end_column").unwrap();
        let source_hash_field = schema.get_field("source_hash").unwrap();

        let mut doc = TantivyDocument::new();
        
        doc.add_text(name_field, &declaration.name);
        doc.add_text(package_field, &java_file.package);
        doc.add_text(file_path_field, java_file.path.to_string_lossy().as_ref());
        doc.add_text(signature_field, &declaration.signature);
        
        if let Some(documentation) = &declaration.documentation {
            doc.add_text(documentation_field, documentation);
        }

        doc.add_text(kind_field, format!("{:?}", declaration.kind));
        doc.add_text(modifiers_field, declaration.modifiers.join(" "));
        
        let annotations: Vec<String> = declaration.annotations
            .iter()
            .map(|a| a.name.clone())
            .collect();
        doc.add_text(annotations_field, annotations.join(" "));

        if let Some(extends) = &declaration.extends {
            doc.add_text(extends_field, extends);
        }

        doc.add_text(implements_field, declaration.implements.join(" "));

        let fields_json = serde_json::to_string(&declaration.fields)?;
        doc.add_text(fields_field, fields_json);

        let methods_json = serde_json::to_string(&declaration.methods)?;
        doc.add_text(methods_field, methods_json);

        doc.add_u64(start_line_field, declaration.range.start_line as u64);
        doc.add_u64(end_line_field, declaration.range.end_line as u64);
        doc.add_u64(start_column_field, declaration.range.start_column as u64);
        doc.add_u64(end_column_field, declaration.range.end_column as u64);

        doc.add_text(source_hash_field, &java_file.source_hash);

        Ok(doc)
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        
        let query_obj = self.build_query(query)?;
        let top_docs = searcher.search(
            &query_obj,
            &TopDocs::with_limit(query.limit.unwrap_or(100)),
        )?;

        let mut results = Vec::new();
        
        for (_score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let result = self.document_to_result(&doc, searcher.clone())?;
            results.push(result);
        }

        Ok(results)
    }

    fn build_query(&self, search: &SearchQuery) -> Result<Box<dyn Query>> {
        let schema = &self.schema;
        
        match search.kind {
            crate::types::SearchKind::Exact => {
                let query_parser = QueryParser::for_index(
                    &self.index,
                    vec![
                        schema.get_field("name").unwrap(),
                        schema.get_field("signature").unwrap(),
                        schema.get_field("documentation").unwrap(),
                    ],
                );
                Ok(query_parser.parse_query(&search.query)?)
            }
            crate::types::SearchKind::Fuzzy => {
                let name_field = schema.get_field("name").unwrap();
                let term = Term::from_field_text(name_field, &search.query);
                let fuzzy_query = FuzzyTermQuery::new(term, 2, true);
                Ok(Box::new(fuzzy_query))
            }
            crate::types::SearchKind::Regex => {
                let query_parser = QueryParser::for_index(
                    &self.index,
                    vec![schema.get_field("name").unwrap()],
                );
                Ok(query_parser.parse_query(&search.query)?)
            }
        }
    }

    fn document_to_result(&self, doc: &TantivyDocument, _searcher: Searcher) -> Result<SearchResult> {
        let schema = &self.schema;
        
        let name_field = schema.get_field("name").unwrap();
        let file_path_field = schema.get_field("file_path").unwrap();
        let signature_field = schema.get_field("signature").unwrap();
        let _start_line_field = schema.get_field("start_line").unwrap();
        let _end_line_field = schema.get_field("end_line").unwrap();

        let name = doc.get_first(name_field)
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let file_path = doc.get_first(file_path_field)
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let signature = doc.get_first(signature_field)
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let declaration = self.create_declaration_from_doc(doc)?;
        
        // Create a simple preview
        let preview = format!("{}: {}", name, signature);

        Ok(SearchResult {
            declaration,
            file_path: PathBuf::from(file_path),
            score: 1.0, // TODO: Calculate actual score
            preview,
        })
    }

    fn create_declaration_from_doc(&self, doc: &TantivyDocument) -> Result<Declaration> {
        let schema = &self.schema;
        
        let get_text = |field_name: &str| {
            let field = schema.get_field(field_name).unwrap();
            doc.get_first(field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };

        let get_u64 = |field_name: &str| {
            let field = schema.get_field(field_name).unwrap();
            doc.get_first(field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize
        };

        let name = get_text("name");
        let kind = match get_text("kind").as_str() {
            "Class" => DeclarationKind::Class,
            "Interface" => DeclarationKind::Interface,
            "Enum" => DeclarationKind::Enum,
            "Record" => DeclarationKind::Record,
            "Annotation" => DeclarationKind::Annotation,
            _ => DeclarationKind::Class,
        };

        let signature = get_text("signature");
        let _package = get_text("package");
        let _file_path = PathBuf::from(get_text("file_path"));

        // Read fields and methods from JSON
        let fields_json = get_text("fields");
        let methods_json = get_text("methods");
        
        let fields: Vec<Field> = serde_json::from_str(&fields_json).unwrap_or_default();
        let methods: Vec<Method> = serde_json::from_str(&methods_json).unwrap_or_default();

        Ok(Declaration {
            name,
            kind,
            modifiers: get_text("modifiers").split_whitespace().map(String::from).collect(),
            annotations: vec![], // TODO: Parse annotations
            signature,
            extends: Some(get_text("extends")).filter(|s| !s.is_empty()),
            implements: get_text("implements").split_whitespace().map(String::from).collect(),
            fields,
            methods,
            range: crate::types::SourceRange {
                start_line: get_u64("start_line"),
                start_column: get_u64("start_column"),
                end_line: get_u64("end_line"),
                end_column: get_u64("end_column"),
            },
            documentation: Some(get_text("documentation")).filter(|s| !s.is_empty()),
        })
    }

    pub async fn delete_by_hash(&self, source_hash: &str) -> Result<()> {
        let mut writer = self.writer.write().await;
        
        let source_hash_field = self.schema.get_field("source_hash").unwrap();
        let term = Term::from_field_text(source_hash_field, source_hash);
        
        writer.delete_term(term);
        writer.commit()?;
        
        Ok(())
    }

    pub async fn optimize(&self) -> Result<()> {
        let mut writer = self.writer.write().await;
        writer.commit()?;
        Ok(())
    }

    pub fn stats(&self) -> Result<(usize, usize)> {
        let searcher = self.reader.searcher();
        let num_docs = searcher.num_docs() as usize;
        
        // Get segment info
        let segment_metas = self.index.searchable_segment_metas()?;
        let num_segments = segment_metas.len();
        
        Ok((num_docs, num_segments))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::types::{DeclarationKind, SourceRange};

    #[tokio::test]
    async fn test_index_creation() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("test_index");
        
        let manager = IndexManager::new(&index_path).unwrap();
        let (num_docs, _) = manager.stats().unwrap();
        
        assert_eq!(num_docs, 0);
    }

    #[tokio::test]
    async fn test_index_and_search() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("test_index");
        
        let manager = IndexManager::new(&index_path).unwrap();
        
        let java_file = crate::types::JavaFile {
            path: PathBuf::from("/test/UserService.java"),
            module: None,
            package: "com.example".to_string(),
            imports: vec![],
            declarations: vec![
                Declaration {
                    name: "UserService".to_string(),
                    kind: DeclarationKind::Class,
                    modifiers: vec!["public".to_string()],
                    annotations: vec![],
                    signature: "public class UserService".to_string(),
                    extends: None,
                    implements: vec![],
                    fields: vec![],
                    methods: vec![],
                    range: SourceRange {
                        start_line: 1,
                        start_column: 1,
                        end_line: 10,
                        end_column: 1,
                    },
                    documentation: Some("Service for user operations".to_string()),
                },
            ],
            source_hash: "abc123".to_string(),
        };

        manager.index_java_file(&java_file).await.unwrap();
        
        let query = SearchQuery {
            query: "UserService".to_string(),
            kind: crate::types::SearchKind::Exact,
            filters: vec![],
            limit: Some(10),
        };

        let results = manager.search(&query).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].declaration.name, "UserService");
    }
}