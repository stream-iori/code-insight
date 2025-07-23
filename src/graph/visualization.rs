use anyhow::Result;
use std::collections::{HashMap, HashSet};

use crate::types::{ReferenceGraph, GraphNode, RelationshipType, DeclarationKind};

pub struct GraphVisualizer;

impl GraphVisualizer {
    pub fn to_mermaid(
        &self,
        graph: &ReferenceGraph,
        config: &VisualizationConfig,
    ) -> Result<String> {
        let mut mermaid = String::new();
        
        if config.direction == Direction::LeftToRight {
            mermaid.push_str("graph LR\n");
        } else {
            mermaid.push_str("graph TD\n");
        }

        // Add styling for different node types
        mermaid.push_str(&self.generate_styles(config));

        // Add nodes
        for node in &graph.nodes {
            let node_style = self.get_node_style(node, config);
            mermaid.push_str(&format!("    {}[{}]{}\n", 
                node.id, 
                self.escape_label(&node.label),
                node_style
            ));
        }

        // Add edges
        for edge in &graph.edges {
            let edge_style = self.get_edge_style(&edge.relationship);
            mermaid.push_str(&format!("    {} --{}--> {}\n", 
                edge.from, 
                edge_style,
                edge.to
            ));
        }

        Ok(mermaid)
    }

    pub fn to_dot(
        &self,
        graph: &ReferenceGraph,
        config: &VisualizationConfig,
    ) -> Result<String> {
        let mut dot = String::from("digraph G {\n");
        
        if config.direction == Direction::LeftToRight {
            dot.push_str("    rankdir=LR;\n");
        }

        dot.push_str("    node [fontname=\"Helvetica\"];\n");
        dot.push_str("    edge [fontname=\"Helvetica\"];\n");

        // Add nodes with styling
        for node in &graph.nodes {
            let node_attrs = self.get_dot_node_attrs(node, config);
            dot.push_str(&format!("    \"{}\" [{}];\n", 
                node.id, 
                node_attrs
            ));
        }

        // Add edges with styling
        for edge in &graph.edges {
            let edge_attrs = self.get_dot_edge_attrs(&edge.relationship);
            dot.push_str(&format!("    \"{}\" -> \"{}\" [{}];\n", 
                edge.from, 
                edge.to,
                edge_attrs
            ));
        }

        dot.push_str("}\n");
        Ok(dot)
    }

    pub fn to_svg(&self, graph: &ReferenceGraph, config: &VisualizationConfig) -> Result<String> {
        let dot = self.to_dot(graph, config)?;
        
        // Use external graphviz tool to convert dot to SVG
        // This is a placeholder - would need actual graphviz integration
        Ok(format!("<!-- SVG generated from:\n{}\n-->", dot))
    }

    pub fn generate_focused_graph(
        &self,
        graph: &ReferenceGraph,
        focus_node: &str,
        depth: usize,
    ) -> Result<ReferenceGraph> {
        let mut focused_nodes = HashSet::new();
        let mut focused_edges = Vec::new();

        // Find the focus node and its neighbors up to specified depth
        self.collect_neighbors(graph, focus_node, depth, &mut focused_nodes);

        // Filter edges to only include those between focused nodes
        for edge in &graph.edges {
            if focused_nodes.contains(&edge.from) && focused_nodes.contains(&edge.to) {
                focused_edges.push(edge.clone());
            }
        }

        let nodes = graph.nodes
            .iter()
            .filter(|node| focused_nodes.contains(&node.id))
            .cloned()
            .collect();

        Ok(ReferenceGraph {
            nodes,
            edges: focused_edges,
        })
    }

    fn collect_neighbors(
        &self,
        graph: &ReferenceGraph,
        node_id: &str,
        depth: usize,
        collected: &mut HashSet<String>,
    ) {
        if depth == 0 || collected.contains(node_id) {
            return;
        }

        collected.insert(node_id.to_string());

        // Collect direct neighbors
        for edge in &graph.edges {
            if edge.from == node_id {
                self.collect_neighbors(graph, &edge.to, depth - 1, collected);
            } else if edge.to == node_id {
                self.collect_neighbors(graph, &edge.from, depth - 1, collected);
            }
        }
    }

    pub fn generate_dependency_matrix(
        &self,
        graph: &ReferenceGraph,
    ) -> Result<DependencyMatrix> {
        let mut node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        node_ids.sort();

        let mut matrix = vec![vec![false; node_ids.len()]; node_ids.len()];
        let node_index: HashMap<String, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        for edge in &graph.edges {
            if let (Some(from_idx), Some(to_idx)) = (
                node_index.get(&edge.from),
                node_index.get(&edge.to),
            ) {
                matrix[*from_idx][*to_idx] = true;
            }
        }

        Ok(DependencyMatrix {
            nodes: node_ids,
            matrix,
        })
    }

    fn generate_styles(&self, _config: &VisualizationConfig) -> String {
        let mut styles = String::new();

        // Class styling
        styles.push_str("    classDef classNode fill:#e1f5fe,stroke:#01579b,stroke-width:2px\n");
        styles.push_str("    classDef interfaceNode fill:#f3e5f5,stroke:#4a148c,stroke-width:2px\n");
        styles.push_str("    classDef enumNode fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px\n");
        styles.push_str("    classDef recordNode fill:#fff3e0,stroke:#e65100,stroke-width:2px\n");
        styles.push_str("    classDef annotationNode fill:#fce4ec,stroke:#880e4f,stroke-width:2px\n");

        styles
    }

    fn get_node_style(&self, node: &GraphNode, _config: &VisualizationConfig) -> String {
        let class_name = match node.kind {
            DeclarationKind::Class => "classNode",
            DeclarationKind::Interface => "interfaceNode",
            DeclarationKind::Enum => "enumNode",
            DeclarationKind::Record => "recordNode",
            DeclarationKind::Annotation => "annotationNode",
        };

        format!(":::{}", class_name)
    }

    fn get_edge_style(&self, relationship: &RelationshipType) -> String {
        match relationship {
            RelationshipType::Extends => "extends",
            RelationshipType::Implements => "implements",
            RelationshipType::Uses => "uses",
            RelationshipType::References => "references",
            RelationshipType::DependsOn => "depends on",
        }
        .to_string()
    }

    fn get_dot_node_attrs(&self, node: &GraphNode, _config: &VisualizationConfig) -> String {
        let color = match node.kind {
            DeclarationKind::Class => "lightblue",
            DeclarationKind::Interface => "lightcoral",
            DeclarationKind::Enum => "lightgreen",
            DeclarationKind::Record => "lightyellow",
            DeclarationKind::Annotation => "lightpink",
        };

        format!("shape=box, style=filled, fillcolor={}, label=\"{}\"", 
            color, 
            self.escape_label(&node.label)
        )
    }

    fn get_dot_edge_attrs(&self, relationship: &RelationshipType) -> String {
        let (style, label) = match relationship {
            RelationshipType::Extends => ("solid", "extends"),
            RelationshipType::Implements => ("dashed", "implements"),
            RelationshipType::Uses => ("dotted", "uses"),
            RelationshipType::References => ("solid", "references"),
            RelationshipType::DependsOn => ("bold", "depends on"),
        };

        format!("style={}, label=\"{}\"", style, label)
    }

    fn escape_label(&self, label: &str) -> String {
        label.replace('"', "\\\"").replace('\n', "\\n")
    }

    pub fn generate_summary(&self, graph: &ReferenceGraph) -> GraphSummary {
        let mut node_counts = HashMap::new();
        let mut relationship_counts = HashMap::new();

        for node in &graph.nodes {
            *node_counts.entry(node.kind.clone()).or_insert(0) += 1;
        }

        for edge in &graph.edges {
            *relationship_counts.entry(edge.relationship.clone()).or_insert(0) += 1;
        }

        GraphSummary {
            total_nodes: graph.nodes.len(),
            total_edges: graph.edges.len(),
            node_counts,
            relationship_counts,
            isolated_nodes: self.count_isolated_nodes(graph),
            strongly_connected_components: self.count_strongly_connected_components(graph),
        }
    }

    fn count_isolated_nodes(&self, graph: &ReferenceGraph) -> usize {
        let mut connected = HashSet::new();
        
        for edge in &graph.edges {
            connected.insert(&edge.from);
            connected.insert(&edge.to);
        }

        graph.nodes.iter().filter(|n| !connected.contains(&n.id)).count()
    }

    fn count_strongly_connected_components(&self, _graph: &ReferenceGraph) -> usize {
        // Placeholder for Tarjan's algorithm implementation
        // This would need a proper SCC algorithm
        0
    }
}

#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    pub direction: Direction,
    pub show_labels: bool,
    pub color_scheme: ColorScheme,
    pub node_size: NodeSize,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            direction: Direction::TopDown,
            show_labels: true,
            color_scheme: ColorScheme::Default,
            node_size: NodeSize::Medium,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    TopDown,
    LeftToRight,
    BottomUp,
    RightToLeft,
}

#[derive(Debug, Clone)]
pub enum ColorScheme {
    Default,
    Dark,
    Light,
    HighContrast,
}

#[derive(Debug, Clone)]
pub enum NodeSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone)]
pub struct DependencyMatrix {
    pub nodes: Vec<String>,
    pub matrix: Vec<Vec<bool>>,
}

#[derive(Debug, Clone)]
pub struct GraphSummary {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_counts: HashMap<DeclarationKind, usize>,
    pub relationship_counts: HashMap<RelationshipType, usize>,
    pub isolated_nodes: usize,
    pub strongly_connected_components: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DeclarationKind, GraphNode, GraphEdge};

    #[test]
    fn test_mermaid_generation() {
        let visualizer = GraphVisualizer;
        let config = VisualizationConfig::default();
        
        let graph = ReferenceGraph {
            nodes: vec![
                GraphNode {
                    id: "UserService".to_string(),
                    label: "UserService".to_string(),
                    kind: DeclarationKind::Class,
                    file_path: "/test/UserService.java".into(),
                },
                GraphNode {
                    id: "UserRepository".to_string(),
                    label: "UserRepository".to_string(),
                    kind: DeclarationKind::Interface,
                    file_path: "/test/UserRepository.java".into(),
                },
            ],
            edges: vec![
                GraphEdge {
                    from: "UserService".to_string(),
                    to: "UserRepository".to_string(),
                    relationship: RelationshipType::Uses,
                },
            ],
        };

        let mermaid = visualizer.to_mermaid(&graph, &config).unwrap();
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("UserService"));
        assert!(mermaid.contains("UserRepository"));
    }

    #[test]
    fn test_dot_generation() {
        let visualizer = GraphVisualizer;
        let config = VisualizationConfig::default();
        
        let graph = ReferenceGraph {
            nodes: vec![
                GraphNode {
                    id: "A".to_string(),
                    label: "A".to_string(),
                    kind: DeclarationKind::Class,
                    file_path: "/test/A.java".into(),
                },
                GraphNode {
                    id: "B".to_string(),
                    label: "B".to_string(),
                    kind: DeclarationKind::Interface,
                    file_path: "/test/B.java".into(),
                },
            ],
            edges: vec![
                GraphEdge {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    relationship: RelationshipType::Implements,
                },
            ],
        };

        let dot = visualizer.to_dot(&graph, &config).unwrap();
        assert!(dot.contains("digraph G"));
        assert!(dot.contains("A -> B"));
    }

    #[test]
    fn test_focused_graph() {
        let visualizer = GraphVisualizer;
        
        let graph = ReferenceGraph {
            nodes: vec![
                GraphNode {
                    id: "A".to_string(),
                    label: "A".to_string(),
                    kind: DeclarationKind::Class,
                    file_path: "/test/A.java".into(),
                },
                GraphNode {
                    id: "B".to_string(),
                    label: "B".to_string(),
                    kind: DeclarationKind::Class,
                    file_path: "/test/B.java".into(),
                },
                GraphNode {
                    id: "C".to_string(),
                    label: "C".to_string(),
                    kind: DeclarationKind::Class,
                    file_path: "/test/C.java".into(),
                },
            ],
            edges: vec![
                GraphEdge {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    relationship: RelationshipType::Uses,
                },
                GraphEdge {
                    from: "B".to_string(),
                    to: "C".to_string(),
                    relationship: RelationshipType::Uses,
                },
            ],
        };

        let focused = visualizer.generate_focused_graph(&graph, "A", 1).unwrap();
        assert_eq!(focused.nodes.len(), 2);
        assert_eq!(focused.edges.len(), 1);
    }

    #[test]
    fn test_summary_generation() {
        let visualizer = GraphVisualizer;
        
        let graph = ReferenceGraph {
            nodes: vec![
                GraphNode {
                    id: "A".to_string(),
                    label: "A".to_string(),
                    kind: DeclarationKind::Class,
                    file_path: "/test/A.java".into(),
                },
                GraphNode {
                    id: "B".to_string(),
                    label: "B".to_string(),
                    kind: DeclarationKind::Interface,
                    file_path: "/test/B.java".into(),
                },
            ],
            edges: vec![
                GraphEdge {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    relationship: RelationshipType::Implements,
                },
            ],
        };

        let summary = visualizer.generate_summary(&graph);
        assert_eq!(summary.total_nodes, 2);
        assert_eq!(summary.total_edges, 1);
        assert_eq!(summary.node_counts.get(&DeclarationKind::Class), Some(&1));
        assert_eq!(summary.node_counts.get(&DeclarationKind::Interface), Some(&1));
    }
}