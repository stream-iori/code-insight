use anyhow::Result;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::types::MavenModule;

pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    pub fn analyze_dependencies(
        &self,
        modules: &[MavenModule],
    ) -> Result<DependencyGraph> {
        let mut graph = DependencyGraph::new();
        
        // Create module map for quick lookup
        let _module_map: HashMap<String, &MavenModule> = modules
            .iter()
            .map(|m| (format!("{}:{}:{}", m.group_id, m.artifact_id, m.version), m))
            .collect();
        
        // Build dependency graph
        for module in modules {
            let module_id = format!("{}:{}:{}", module.group_id, module.artifact_id, module.version);
            graph.add_node(module_id.clone(), module.path.clone());
            
            for dep in &module.dependencies {
                let dep_id = format!("{}:{}:{}", dep.group_id, dep.artifact_id, dep.version);
                graph.add_edge(module_id.clone(), dep_id);
            }
        }
        
        Ok(graph)
    }
    
    pub fn find_jar_files(
        &self,
        project_root: &Path,
    ) -> Result<Vec<PathBuf>> {
        let mut jar_files = Vec::new();
        
        // Look in common Maven directories
        let paths_to_check = [
            project_root.join("target"),
            project_root.join(".m2"),
            project_root.join("lib"),
        ];
        
        for path in &paths_to_check {
            if path.exists() {
                for entry in walkdir::WalkDir::new(path) {
                    if let Ok(entry) = entry {
                        if entry.file_type().is_file() && entry.path().extension()
                            .map(|ext| ext == "jar").unwrap_or(false) {
                            jar_files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }
        
        Ok(jar_files)
    }
    
    pub fn resolve_dependency_tree(
        &self,
        modules: &[MavenModule],
    ) -> Result<HashMap<String, Vec<String>>> {
        let mut tree = HashMap::new();
        
        for module in modules {
            let module_id = format!("{}:{}:{}", module.group_id, module.artifact_id, module.version);
            let mut deps = Vec::new();
            
            for dep in &module.dependencies {
                deps.push(format!("{}:{}:{}", dep.group_id, dep.artifact_id, dep.version));
            }
            
            tree.insert(module_id, deps);
        }
        
        Ok(tree)
    }
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, PathBuf>,
    pub edges: Vec<(String, String)>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }
    
    pub fn add_node(&mut self, id: String, path: PathBuf) {
        self.nodes.insert(id, path);
    }
    
    pub fn add_edge(&mut self, from: String, to: String) {
        self.edges.push((from, to));
    }
    
    pub fn get_dependencies(&self,
        module_id: &str,
    ) -> Vec<String> {
        self.edges
            .iter()
            .filter(|(from, _)| from == module_id)
            .map(|(_, to)| to.clone())
            .collect()
    }
    
    pub fn get_dependents(&self,
        module_id: &str,
    ) -> Vec<String> {
        self.edges
            .iter()
            .filter(|(_, to)| to == module_id)
            .map(|(from, _)| from.clone())
            .collect()
    }
    
    pub fn to_mermaid(&self,
    ) -> String {
        let mut mermaid = String::from("graph TD\n");
        
        for (from, to) in &self.edges {
            mermaid.push_str(&format!("    {} --> {}\n", from, to));
        }
        
        mermaid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dependency_analysis() {
        let analyzer = DependencyAnalyzer;
        let modules = vec![
            MavenModule {
                group_id: "com.example".to_string(),
                artifact_id: "app".to_string(),
                version: "1.0.0".to_string(),
                packaging: None,
                path: PathBuf::from("/app"),
                dependencies: vec![crate::types::MavenDependency {
                    group_id: "org.lib".to_string(),
                    artifact_id: "core".to_string(),
                    version: "2.0.0".to_string(),
                    scope: None,
                    optional: false,
                }],
                submodules: vec![],
            },
            MavenModule {
                group_id: "org.lib".to_string(),
                artifact_id: "core".to_string(),
                version: "2.0.0".to_string(),
                packaging: None,
                path: PathBuf::from("/lib"),
                dependencies: vec![],
                submodules: vec![],
            },
        ];
        
        let graph = analyzer.analyze_dependencies(&modules).unwrap();
        
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].0, "com.example:app:1.0.0");
        assert_eq!(graph.edges[0].1, "org.lib:core:2.0.0");
    }
    
    #[test]
    fn test_dependency_tree() {
        let analyzer = DependencyAnalyzer;
        let modules = vec![
            MavenModule {
                group_id: "com.example".to_string(),
                artifact_id: "app".to_string(),
                version: "1.0.0".to_string(),
                packaging: None,
                path: PathBuf::from("/app"),
                dependencies: vec![
                    crate::types::MavenDependency {
                        group_id: "org.lib".to_string(),
                        artifact_id: "core".to_string(),
                        version: "2.0.0".to_string(),
                        scope: None,
                        optional: false,
                    },
                    crate::types::MavenDependency {
                        group_id: "org.lib".to_string(),
                        artifact_id: "utils".to_string(),
                        version: "1.5.0".to_string(),
                        scope: None,
                        optional: false,
                    },
                ],
                submodules: vec![],
            },
        ];
        
        let tree = analyzer.resolve_dependency_tree(&modules).unwrap();
        
        assert_eq!(tree.len(), 1);
        let deps = tree.get("com.example:app:1.0.0").unwrap();
        assert_eq!(deps.len(), 2);
    }
}