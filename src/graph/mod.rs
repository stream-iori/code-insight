mod visualization;

pub use visualization::*;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::types::{Declaration, ReferenceGraph, GraphNode, GraphEdge, RelationshipType};

pub struct GraphBuilder {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
    type_references: HashMap<String, HashSet<String>>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            type_references: HashMap::new(),
        }
    }

    pub fn add_declaration(&mut self, declaration: &Declaration, file_path: &PathBuf) {
        let node_id = format!("{}:{}", file_path.display(), declaration.name);
        
        let node = GraphNode {
            id: node_id.clone(),
            label: declaration.name.clone(),
            kind: declaration.kind.clone(),
            file_path: file_path.clone(),
        };
        
        self.nodes.insert(node_id.clone(), node);
        
        // Add inheritance relationships
        if let Some(extends) = &declaration.extends {
            self.add_edge(
                node_id.clone(),
                format!("extends:{}", extends),
                RelationshipType::Extends,
            );
        }
        
        for implements in &declaration.implements {
            self.add_edge(
                node_id.clone(),
                format!("implements:{}", implements),
                RelationshipType::Implements,
            );
        }
    }

    pub fn add_edge(
        &mut self,
        from: String,
        to: String,
        relationship: RelationshipType,
    ) {
        self.edges.push(GraphEdge {
            from,
            to,
            relationship,
        });
    }

    pub fn add_type_reference(
        &mut self,
        from_type: String,
        to_type: String,
    ) {
        self.type_references
            .entry(from_type)
            .or_insert_with(HashSet::new)
            .insert(to_type);
    }

    pub fn build(&self) -> ReferenceGraph {
        ReferenceGraph {
            nodes: self.nodes.values().cloned().collect(),
            edges: self.edges.clone(),
        }
    }

    pub fn get_dependencies(&self, type_name: &str) -> Vec<String> {
        self.type_references
            .get(type_name)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_else(Vec::new)
    }

    pub fn get_dependents(&self, type_name: &str) -> Vec<String> {
        let mut dependents = Vec::new();
        
        for (from, to_set) in &self.type_references {
            if to_set.contains(type_name) {
                dependents.push(from.clone());
            }
        }
        
        dependents
    }

    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                self.dfs_find_cycle(
                    node,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_find_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(dependencies) = self.type_references.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    self.dfs_find_cycle(dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|x| x == dep).unwrap();
                    let cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    pub fn calculate_complexity(&self, type_name: &str) -> usize {
        let mut complexity = 0;
        
        // Count direct dependencies
        if let Some(deps) = self.type_references.get(type_name) {
            complexity += deps.len();
        }
        
        // Count dependents
        complexity += self.get_dependents(type_name).len();
        
        complexity
    }

    pub fn get_components(&self) -> Vec<Vec<String>> {
        let mut components = Vec::new();
        let mut visited = HashSet::new();

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                let mut component = Vec::new();
                self.bfs_component(node, &mut visited, &mut component);
                components.push(component);
            }
        }

        components
    }

    fn bfs_component(
        &self,
        start: &str,
        visited: &mut HashSet<String>,
        component: &mut Vec<String>,
    ) {
        use std::collections::VecDeque;
        
        let mut queue = VecDeque::new();
        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(node) = queue.pop_front() {
            component.push(node.clone());

            // Add dependencies
            if let Some(deps) = self.type_references.get(&node) {
                for dep in deps {
                    if !visited.contains(dep) {
                        visited.insert(dep.clone());
                        queue.push_back(dep.clone());
                    }
                }
            }

            // Add dependents
            for (from, to_set) in &self.type_references {
                if to_set.contains(&node) && !visited.contains(from) {
                    visited.insert(from.clone());
                    queue.push_back(from.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DeclarationKind, Declaration};
    use tempfile::tempdir;

    #[test]
    fn test_graph_builder() {
        let mut builder = GraphBuilder::new();
        
        let declaration = Declaration {
            name: "UserService".to_string(),
            kind: DeclarationKind::Class,
            modifiers: vec!["public".to_string()],
            annotations: vec![],
            signature: "public class UserService".to_string(),
            extends: Some("BaseService".to_string()),
            implements: vec!["UserInterface".to_string()],
            fields: vec![],
            methods: vec![],
            range: crate::types::SourceRange {
                start_line: 1,
                start_column: 1,
                end_line: 10,
                end_column: 1,
            },
            documentation: None,
        };

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("UserService.java");

        builder.add_declaration(&declaration, &file_path);
        
        let graph = builder.build();
        
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(graph.nodes[0].label, "UserService");
    }

    #[test]
    fn test_type_references() {
        let mut builder = GraphBuilder::new();
        
        builder.add_type_reference("UserService".to_string(), "UserRepository".to_string());
        builder.add_type_reference("UserService".to_string(), "User".to_string());
        builder.add_type_reference("UserController".to_string(), "UserService".to_string());
        
        assert_eq!(builder.get_dependencies("UserService"), vec!["UserRepository", "User"]);
        assert_eq!(builder.get_dependents("UserService"), vec!["UserController"]);
    }

    #[test]
    fn test_components() {
        let mut builder = GraphBuilder::new();
        
        // Create isolated components
        builder.add_type_reference("A".to_string(), "B".to_string());
        builder.add_type_reference("B".to_string(), "C".to_string());
        builder.add_type_reference("X".to_string(), "Y".to_string());
        
        let components = builder.get_components();
        
        assert_eq!(components.len(), 2);
        assert!(components.iter().any(|c| c.contains(&"A".to_string())));
        assert!(components.iter().any(|c| c.contains(&"X".to_string())));
    }
}