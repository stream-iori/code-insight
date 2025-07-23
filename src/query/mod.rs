use anyhow::Result;
use std::path::Path;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::indexer::IndexManager;
use crate::types::{SearchQuery, SearchResult, DeclarationKind, SearchFilter};

pub struct QueryEngine {
    index_manager: IndexManager,
    cache: RwLock<HashMap<String, Vec<SearchResult>>>,
}

impl QueryEngine {
    pub fn new(index_path: &Path) -> Result<Self> {
        let index_manager = IndexManager::new(index_path)?;
        
        Ok(Self {
            index_manager,
            cache: RwLock::new(HashMap::new()),
        })
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        // Check cache first
        let cache_key = format!("{:?}:{}", query.kind, query.query);
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let mut results = self.index_manager.search(query).await?;
        
        // Apply filters
        results = self.apply_filters(results, &query.filters)?;
        
        // Apply sorting
        results = self.sort_results(results, &query.kind);

        // Cache results
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, results.clone());
        }

        Ok(results)
    }

    pub async fn search_by_kind(&self, kind: DeclarationKind, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        let query = SearchQuery {
            query: format!("{:?}", kind),
            kind: crate::types::SearchKind::Exact,
            filters: vec![SearchFilter::Kind(kind)],
            limit,
        };
        
        self.search(&query).await
    }

    pub async fn search_by_annotation(&self, annotation: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        let query = SearchQuery {
            query: annotation.to_string(),
            kind: crate::types::SearchKind::Exact,
            filters: vec![SearchFilter::Annotation(annotation.to_string())],
            limit,
        };
        
        self.search(&query).await
    }

    pub async fn search_by_package(&self, package: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        let query = SearchQuery {
            query: package.to_string(),
            kind: crate::types::SearchKind::Exact,
            filters: vec![SearchFilter::Package(package.to_string())],
            limit,
        };
        
        self.search(&query).await
    }

    pub async fn fuzzy_search(&self, query: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            kind: crate::types::SearchKind::Fuzzy,
            filters: vec![],
            limit,
        };
        
        self.search(&search_query).await
    }

    pub async fn exact_search(&self, query: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            kind: crate::types::SearchKind::Exact,
            filters: vec![],
            limit,
        };
        
        self.search(&search_query).await
    }

    pub async fn regex_search(&self, pattern: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: pattern.to_string(),
            kind: crate::types::SearchKind::Regex,
            filters: vec![],
            limit,
        };
        
        self.search(&search_query).await
    }

    fn apply_filters(&self, mut results: Vec<SearchResult>, filters: &[SearchFilter]) -> Result<Vec<SearchResult>> {
        for filter in filters {
            results = match filter {
                SearchFilter::Kind(kind) => {
                    results.into_iter()
                        .filter(|r| r.declaration.kind == *kind)
                        .collect()
                }
                SearchFilter::Annotation(annotation) => {
                    results.into_iter()
                        .filter(|r| {
                            r.declaration.annotations.iter()
                                .any(|a| a.name.contains(annotation))
                        })
                        .collect()
                }
                SearchFilter::Package(package) => {
                    results.into_iter()
                        .filter(|r| {
                            r.file_path.to_string_lossy().contains(package)
                        })
                        .collect()
                }
                SearchFilter::Module(module) => {
                    results.into_iter()
                        .filter(|r| {
                            r.file_path.to_string_lossy().contains(module)
                        })
                        .collect()
                }
            };
        }

        Ok(results)
    }

    fn sort_results(&self, mut results: Vec<SearchResult>, kind: &crate::types::SearchKind) -> Vec<SearchResult> {
        match kind {
            crate::types::SearchKind::Fuzzy => {
                // Sort by score (highest first)
                results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            }
            crate::types::SearchKind::Exact => {
                // Sort by name for exact matches
                results.sort_by(|a, b| a.declaration.name.cmp(&b.declaration.name));
            }
            crate::types::SearchKind::Regex => {
                // Sort by file path for regex matches
                results.sort_by(|a, b| a.file_path.cmp(&b.file_path));
            }
        }
        results
    }

    pub async fn get_statistics(&self) -> Result<QueryStatistics> {
        let (total_docs, _) = self.index_manager.stats()?;
        
        let class_count = self.search_by_kind(DeclarationKind::Class, None).await?.len();
        let interface_count = self.search_by_kind(DeclarationKind::Interface, None).await?.len();
        let enum_count = self.search_by_kind(DeclarationKind::Enum, None).await?.len();
        let record_count = self.search_by_kind(DeclarationKind::Record, None).await?.len();
        let annotation_count = self.search_by_kind(DeclarationKind::Annotation, None).await?.len();

        Ok(QueryStatistics {
            total_declarations: total_docs,
            class_count,
            interface_count,
            enum_count,
            record_count,
            annotation_count,
        })
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        (cache.len(), cache.values().map(|v| v.len()).sum())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStatistics {
    pub total_declarations: usize,
    pub class_count: usize,
    pub interface_count: usize,
    pub enum_count: usize,
    pub record_count: usize,
    pub annotation_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_query_engine() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("test_index");
        
        let query_engine = QueryEngine::new(&index_path).unwrap();
        
        // Test empty search
        let results = query_engine.exact_search("test", Some(10)).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_search_with_filters() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("test_index");
        
        let query_engine = QueryEngine::new(&index_path).unwrap();
        
        // Test statistics when empty
        let stats = query_engine.get_statistics().await.unwrap();
        assert_eq!(stats.total_declarations, 0);
        assert_eq!(stats.class_count, 0);
    }
}