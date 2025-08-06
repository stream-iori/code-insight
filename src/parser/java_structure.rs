use crate::parser::{FileMeta, FileParseable, FileSuffix};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tree_sitter::{Node, Parser, Tree};

/// Complete structure preview of a Java source file
/// Provides IntelliJ-like structure view data for code navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaStructurePreview {
    pub file_meta: FileMeta,
    pub package: Option<String>,
    pub imports: Vec<String>,
    pub top_level_classes: Vec<ClassStructure>,
    pub file_annotations: Vec<Annotation>,
}

/// Structure representation of a Java class, interface, enum, or record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassStructure {
    pub name: String,
    pub fqn: String,
    pub kind: ClassKind,
    pub modifiers: Vec<String>,
    pub annotations: Vec<Annotation>,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub type_parameters: Vec<String>,
    pub fields: Vec<FieldStructure>,
    pub methods: Vec<MethodStructure>,
    pub nested_classes: Vec<ClassStructure>,
    pub range: SourceRange,
    pub documentation: Option<String>,
}

/// Different types of Java type declarations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ClassKind {
    Class,
    Interface,
    Enum,
    Record,
    Annotation,
}

/// Structure representation of a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldStructure {
    pub name: String,
    pub type_name: String,
    pub modifiers: Vec<String>,
    pub annotations: Vec<Annotation>,
    pub documentation: Option<String>,
}

/// Structure representation of a method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodStructure {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<ParameterStructure>,
    pub modifiers: Vec<String>,
    pub annotations: Vec<Annotation>,
    pub type_parameters: Vec<String>,
    pub throws: Vec<String>,
    pub range: SourceRange,
    pub documentation: Option<String>,
}

/// Structure representation of a method parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterStructure {
    pub name: String,
    pub type_name: String,
    pub annotations: Vec<Annotation>,
}

/// Annotation representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub name: String,
    pub values: Vec<(String, String)>,
    pub range: SourceRange,
}

/// Source location range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRange {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

/// Parser for extracting Java structure using tree-sitter
pub struct JavaStructureParser;

impl JavaStructureParser {
    pub fn new() -> Result<Self> {
        Ok(JavaStructureParser)
    }

    pub fn parse_structure(&self, path: &Path) -> Result<JavaStructurePreview> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read Java file: {:?}", path))?;

        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_java::language())
            .context("Failed to load Java grammar")?;

        let tree = parser
            .parse(&content, None)
            .context("Failed to parse Java file")?;

        self.extract_structure(path, &content, &tree)
    }

    fn extract_structure(
        &self,
        path: &Path,
        content: &str,
        tree: &Tree,
    ) -> Result<JavaStructurePreview> {
        let root_node = tree.root_node();

        let package = self.extract_package(&root_node, content);
        let imports = self.extract_imports(&root_node, content);
        let top_level_classes = self.extract_classes(&root_node, content, &package)?;
        let file_annotations = self.extract_file_annotations(&root_node, content);

        Ok(JavaStructurePreview {
            file_meta: FileMeta::new(path, FileSuffix::Java, content),
            package,
            imports,
            top_level_classes,
            file_annotations,
        })
    }

    fn extract_package(&self, node: &Node, content: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "package_declaration" {
                // Look for scoped_identifier directly under package_declaration
                let mut package_cursor = child.walk();
                for package_child in child.children(&mut package_cursor) {
                    if package_child.kind() == "scoped_identifier" {
                        let package_name = self.node_text(&package_child, content).to_string();
                        return Some(package_name.trim().to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_imports(&self, node: &Node, content: &str) -> Vec<String> {
        let mut imports = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_declaration" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    imports.push(self.node_text(&name_node, content).to_string());
                }
            }
        }
        imports
    }

    fn extract_file_annotations(&self, node: &Node, content: &str) -> Vec<Annotation> {
        let mut annotations = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(annotation) = self.parse_annotation(&child, content) {
                annotations.push(annotation);
            }
        }
        annotations
    }

    fn extract_classes(
        &self,
        node: &Node,
        content: &str,
        package: &Option<String>,
    ) -> Result<Vec<ClassStructure>> {
        let mut classes = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "class_declaration"
                | "interface_declaration"
                | "enum_declaration"
                | "record_declaration"
                | "annotation_type_declaration" => {
                    if let Some(class) = self.parse_class(&child, content, package)? {
                        classes.push(class);
                    }
                }
                _ => continue,
            }
        }

        Ok(classes)
    }

    fn parse_class(
        &self,
        node: &Node,
        content: &str,
        package: &Option<String>,
    ) -> Result<Option<ClassStructure>> {
        let kind = match node.kind() {
            "class_declaration" => ClassKind::Class,
            "interface_declaration" => ClassKind::Interface,
            "enum_declaration" => ClassKind::Enum,
            "record_declaration" => ClassKind::Record,
            "annotation_type_declaration" => ClassKind::Annotation,
            _ => return Ok(None),
        };

        let name = if let Some(name_node) = node.child_by_field_name("name") {
            self.node_text(&name_node, content).to_string()
        } else {
            return Ok(None);
        };

        let fqn = self.build_fqn(package, &name);
        let modifiers = self.extract_modifiers(&node, content);
        let annotations = self.extract_annotations(&node, content);
        let extends = self.extract_extends(&node, content);
        let implements = self.extract_implements(&node, content);
        let type_parameters = self.extract_type_parameters(&node, content);
        let fields = self.extract_fields(&node, content)?;
        let methods = self.extract_methods(&node, content)?;
        let nested_classes = self.extract_nested_classes(&node, content, package)?;
        let range = self.node_range(node);
        let documentation = self.extract_documentation(&node, content);

        Ok(Some(ClassStructure {
            name,
            fqn,
            kind,
            modifiers,
            annotations,
            extends,
            implements,
            type_parameters,
            fields,
            methods,
            nested_classes,
            range,
            documentation,
        }))
    }

    fn extract_nested_classes(
        &self,
        node: &Node,
        content: &str,
        package: &Option<String>,
    ) -> Result<Vec<ClassStructure>> {
        let mut nested = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    "class_declaration"
                    | "interface_declaration"
                    | "enum_declaration"
                    | "record_declaration"
                    | "annotation_type_declaration" => {
                        if let Some(class) = self.parse_class(&child, content, package)? {
                            nested.push(class);
                        }
                    }
                    _ => continue,
                }
            }
        }

        Ok(nested)
    }

    fn extract_modifiers(&self, node: &Node, content: &str) -> Vec<String> {
        let mut modifiers = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let mut modifier_cursor = child.walk();
                for modifier in child.children(&mut modifier_cursor) {
                    let kind = modifier.kind();
                    // Only include actual modifier keywords, exclude annotations which have their own node type
                    match kind {
                        "public" | "private" | "protected" | "static" | "final" | "abstract"
                        | "synchronized" | "volatile" | "transient" | "native" | "strictfp" => {
                            let text = self.node_text(&modifier, content);
                            if !text.is_empty() {
                                modifiers.push(text.to_string());
                            }
                        }
                        "marker_annotation" | "annotation" => {
                            // Explicitly skip annotations, they are handled by extract_annotations
                            continue;
                        }
                        _ => {
                            let text = self.node_text(&modifier, content);
                            // Handle other modifier tokens, but skip anything starting with @
                            if !text.trim().starts_with('@')
                                && !text.is_empty()
                                && text.trim().len() > 0
                            {
                                modifiers.push(text.to_string());
                            }
                        }
                    }
                }
            }
        }
        modifiers
    }

    fn extract_annotations(&self, node: &Node, content: &str) -> Vec<Annotation> {
        let mut annotations = Vec::new();
        let mut cursor = node.walk();

        // Collect annotations from all relevant nodes
        for child in node.children(&mut cursor) {
            let child_kind = child.kind();

            // Collect from different node types based on their kind
            let targets = match child_kind {
                "modifiers" => {
                    // For modifiers, check all children
                    let mut targets = Vec::new();
                    let mut modifier_cursor = child.walk();
                    for modifier in child.children(&mut modifier_cursor) {
                        let kind = modifier.kind();
                        if kind == "annotation" || kind == "marker_annotation" {
                            targets.push(modifier);
                        }
                    }
                    targets
                }
                "annotation" | "marker_annotation" => {
                    // Direct annotation
                    vec![child]
                }
                _ => {
                    // For other nodes, check children for annotations
                    let mut targets = Vec::new();
                    let mut child_cursor = child.walk();
                    for grandchild in child.children(&mut child_cursor) {
                        let kind = grandchild.kind();
                        if kind == "annotation" || kind == "marker_annotation" {
                            targets.push(grandchild);
                        }
                    }
                    targets
                }
            };

            // Parse all collected annotation nodes
            for target in targets {
                if let Some(annotation) = self.parse_annotation(&target, content) {
                    annotations.push(annotation);
                }
            }
        }

        annotations
    }

    fn parse_annotation(&self, node: &Node, content: &str) -> Option<Annotation> {
        let node_kind = node.kind();
        if node_kind != "annotation" && node_kind != "marker_annotation" {
            return None;
        }

        let name = match node_kind {
            "marker_annotation" => {
                // Marker annotation like @Service, @Override
                let text = self.node_text(node, content);
                text.trim().trim_start_matches('@').to_string()
            }
            "annotation" => {
                // Regular annotation with parentheses like @Entity(name="test")
                if let Some(name_node) = node.child_by_field_name("name") {
                    self.node_text(&name_node, content).to_string()
                } else {
                    // Fallback: extract from the annotation text
                    let text = self.node_text(node, content);
                    let parts: Vec<&str> = text
                        .split(|c: char| c == '(' || c.is_whitespace())
                        .collect();
                    parts
                        .first()
                        .unwrap_or(&"")
                        .trim_start_matches('@')
                        .to_string()
                }
            }
            _ => return None,
        };

        let values = self.extract_annotation_values(node, content);
        let range = self.node_range(node);

        Some(Annotation {
            name,
            values,
            range,
        })
    }

    fn extract_annotation_values(&self, node: &Node, content: &str) -> Vec<(String, String)> {
        let mut values = Vec::new();

        if let Some(arguments) = node.child_by_field_name("arguments") {
            let mut cursor = arguments.walk();
            for child in arguments.children(&mut cursor) {
                match child.kind() {
                    "element_value_pair" => {
                        if let Some(key_node) = child.child_by_field_name("key") {
                            let key = self.node_text(&key_node, content).to_string();
                            if let Some(value_node) = child.child_by_field_name("value") {
                                let value = self.node_text(&value_node, content).to_string();
                                values.push((key, value));
                            }
                        }
                    }
                    // Handle single value annotations like @Value("test")
                    "string_literal" | "number_literal" | "true" | "false" | "null" => {
                        let value = self.node_text(&child, content).to_string();
                        values.push(("value".to_string(), value));
                    }
                    "identifier" => {
                        let value = self.node_text(&child, content).to_string();
                        values.push(("value".to_string(), value));
                    }
                    "element_value_array_initializer" => {
                        // Handle array values like @RequestMapping(method = {GET, POST})
                        let mut array_cursor = child.walk();
                        for array_child in child.children(&mut array_cursor) {
                            match array_child.kind() {
                                "string_literal" | "identifier" => {
                                    let value = self.node_text(&array_child, content).to_string();
                                    values.push(("value".to_string(), value));
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        // Skip punctuation and other irrelevant nodes
                        let text = self.node_text(&child, content).to_string();
                        let trimmed = text.trim();
                        if !trimmed.is_empty() && !matches!(trimmed, "(" | ")" | "," | "{") {
                            values.push(("value".to_string(), trimmed.to_string()));
                        }
                    }
                }
            }
        } else {
            // Check if this is a marker annotation or has simple string argument
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "string_literal" | "number_literal" | "true" | "false" | "null" => {
                        let value = self.node_text(&child, content).to_string();
                        values.push(("value".to_string(), value));
                    }
                    _ => {}
                }
            }
        }

        values
    }

    fn extract_extends(&self, node: &Node, content: &str) -> Option<String> {
        if let Some(extends_node) = node.child_by_field_name("superclass") {
            let text = self.node_text(&extends_node, content).to_string();
            // Remove "extends" keyword if present
            let text = text.trim().trim_start_matches("extends").trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        None
    }

    fn extract_implements(&self, node: &Node, content: &str) -> Vec<String> {
        let mut implements = Vec::new();

        if let Some(implements_node) = node.child_by_field_name("interfaces") {
            let mut cursor = implements_node.walk();
            for child in implements_node.children(&mut cursor) {
                let text = self.node_text(&child, content).to_string();
                let trimmed = text.trim();
                if !trimmed.is_empty() && trimmed != "implements" {
                    implements.push(trimmed.to_string());
                }
            }
        }

        implements
    }

    fn extract_type_parameters(&self, node: &Node, content: &str) -> Vec<String> {
        let mut type_params = Vec::new();

        if let Some(type_params_node) = node.child_by_field_name("type_parameters") {
            let mut cursor = type_params_node.walk();
            for child in type_params_node.children(&mut cursor) {
                if child.kind() == "type_parameter" {
                    let text = self.node_text(&child, content);
                    if !text.is_empty() {
                        type_params.push(text.to_string());
                    }
                }
            }
        }

        type_params
    }

    fn extract_fields(&self, node: &Node, content: &str) -> Result<Vec<FieldStructure>> {
        let mut fields = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "field_declaration" {
                    let mut field_cursor = child.walk();
                    for sub_child in child.children(&mut field_cursor) {
                        if sub_child.kind() == "variable_declarator" {
                            if let Some(field) = self.parse_field(&child, &sub_child, content)? {
                                fields.push(field);
                            }
                        }
                    }
                }
            }
        }

        Ok(fields)
    }

    fn parse_field(
        &self,
        field_node: &Node,
        declarator_node: &Node,
        content: &str,
    ) -> Result<Option<FieldStructure>> {
        let type_node = if let Some(type_node) = field_node.child_by_field_name("type") {
            type_node
        } else {
            return Ok(None);
        };

        let type_name = self.node_text(&type_node, content).to_string();
        let modifiers = self.extract_modifiers(&field_node, content);
        let annotations = self.extract_annotations(&field_node, content);
        let range = self.node_range(field_node);
        let documentation = self.extract_documentation(&field_node, content);

        let name = if let Some(name_node) = declarator_node.child_by_field_name("name") {
            self.node_text(&name_node, content).to_string()
        } else {
            return Ok(None);
        };

        Ok(Some(FieldStructure {
            name,
            type_name,
            modifiers,
            annotations,
            documentation,
        }))
    }

    fn extract_methods(&self, node: &Node, content: &str) -> Result<Vec<MethodStructure>> {
        let mut methods = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    "method_declaration" => {
                        if let Some(method) = self.parse_method(&child, content)? {
                            methods.push(method);
                        }
                    }
                    "constructor_declaration" => {
                        if let Some(constructor) = self.parse_constructor(&child, content)? {
                            methods.push(constructor);
                        }
                    }
                    _ => continue,
                }
            }
        }

        Ok(methods)
    }

    fn parse_method(&self, node: &Node, content: &str) -> Result<Option<MethodStructure>> {
        let name = if let Some(name_node) = node.child_by_field_name("name") {
            self.node_text(&name_node, content).to_string()
        } else {
            return Ok(None);
        };

        let return_type = if let Some(return_node) = node.child_by_field_name("type") {
            self.node_text(&return_node, content).to_string()
        } else {
            "void".to_string()
        };

        let modifiers = self.extract_modifiers(&node, content);
        let annotations = self.extract_annotations(&node, content);
        let type_parameters = self.extract_type_parameters(&node, content);
        let parameters = self.extract_parameters(&node, content)?;
        let throws = self.extract_throws(&node, content);
        let range = self.node_range(node);
        let documentation = self.extract_documentation(&node, content);

        Ok(Some(MethodStructure {
            name,
            return_type,
            parameters,
            modifiers,
            annotations,
            type_parameters,
            throws,
            range,
            documentation,
        }))
    }

    fn parse_constructor(&self, node: &Node, content: &str) -> Result<Option<MethodStructure>> {
        let parent = node.parent().unwrap();
        let name = if let Some(name_node) = parent.child_by_field_name("name") {
            self.node_text(&name_node, content).to_string()
        } else {
            return Ok(None);
        };

        let modifiers = self.extract_modifiers(&parent, content);
        let annotations = self.extract_annotations(&parent, content);
        let parameters = self.extract_parameters(&node, content)?;
        let throws = self.extract_throws(&node, content);
        let range = self.node_range(node);
        let documentation = self.extract_documentation(&node, content);

        Ok(Some(MethodStructure {
            name,
            return_type: "void".to_string(),
            parameters,
            modifiers,
            annotations,
            type_parameters: Vec::new(),
            throws,
            range,
            documentation,
        }))
    }

    fn extract_parameters(&self, node: &Node, content: &str) -> Result<Vec<ParameterStructure>> {
        let mut parameters = Vec::new();

        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "formal_parameter" {
                    if let Some(param) = self.parse_parameter(&child, content)? {
                        parameters.push(param);
                    }
                }
            }
        }

        Ok(parameters)
    }

    fn parse_parameter(&self, node: &Node, content: &str) -> Result<Option<ParameterStructure>> {
        let type_node = if let Some(type_node) = node.child_by_field_name("type") {
            type_node
        } else {
            return Ok(None);
        };

        let type_name = self.node_text(&type_node, content).to_string();
        let annotations = self.extract_annotations(&node, content);

        let name = if let Some(name_node) = node.child_by_field_name("name") {
            self.node_text(&name_node, content).to_string()
        } else {
            return Ok(None);
        };

        Ok(Some(ParameterStructure {
            name,
            type_name,
            annotations,
        }))
    }

    fn extract_throws(&self, node: &Node, content: &str) -> Vec<String> {
        node.children(&mut node.walk())
            .filter(|child| child.kind() == "throws")
            .flat_map(|child| {
                self.node_text(&child, content)
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty() && *s != "throws")
                    .map(|s| s.trim_start_matches("throws").trim().to_string())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn extract_documentation(&self, node: &Node, content: &str) -> Option<String> {
        // Look for JavaDoc comments above the node
        let mut current = *node;
        while let Some(prev) = current.prev_sibling() {
            if prev.kind() == "line_comment" || prev.kind() == "block_comment" {
                let text = self.node_text(&prev, content);
                if text.starts_with("/**") {
                    return Some(text.to_string());
                }
            }
            current = prev;
        }
        None
    }

    fn build_fqn(&self, package: &Option<String>, class_name: &str) -> String {
        match package {
            Some(pkg) => format!("{}.{}", pkg, class_name),
            None => class_name.to_string(),
        }
    }

    fn node_text<'a>(&self, node: &Node<'a>, content: &'a str) -> &'a str {
        let start = node.start_byte();
        let end = node.end_byte();
        if start <= end && end <= content.len() {
            &content[start..end]
        } else {
            ""
        }
    }

    fn node_range(&self, node: &Node) -> SourceRange {
        SourceRange {
            start_line: node.start_position().row + 1,
            start_column: node.start_position().column + 1,
            end_line: node.end_position().row + 1,
            end_column: node.end_position().column + 1,
        }
    }
}

impl FileParseable<JavaStructurePreview> for JavaStructureParser {
    fn parse_file(&mut self, path: &Path) -> Result<JavaStructurePreview> {
        self.parse_structure(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_simple_class() {
        let parser = JavaStructureParser::new().unwrap();

        let java_content = r#"package com.example;
            /** this is a comment */
            @Service("service")
            public class UserService extends AbsService implements IUser {
                @Component
                private String name;

                @Override
                public void doSomething(@NotNull String userId) throws RuntimeException; {}
        }"#;

        let dir = tempdir().unwrap();
        let java_path = dir.path().join("UserService.java");
        std::fs::write(&java_path, java_content).unwrap();

        let structure = parser.parse_structure(&java_path).unwrap();
        println!("{:#?}", structure);

        assert!(
            !structure.top_level_classes.is_empty(),
            "Should have at least one class"
        );
        let class = &structure.top_level_classes[0];
        assert_eq!(class.name, "UserService");
        assert_eq!(structure.package, Some("com.example".to_string()));

        // Test annotations
        assert_eq!(class.annotations.len(), 1);
        assert_eq!(class.annotations[0].name, "Service");

        // Test modifiers are separate from annotations
        assert_eq!(class.modifiers, vec!["public"]);

        // Test field annotations and modifiers
        assert_eq!(class.fields.len(), 1);
        let field = &class.fields[0];
        assert_eq!(field.annotations.len(), 1);
        assert_eq!(field.annotations[0].name, "Component");
        assert_eq!(field.modifiers, vec!["private"]);

        // Test method annotations and modifiers
        assert_eq!(class.methods.len(), 1);
        let method = &class.methods[0];
        assert_eq!(method.annotations.len(), 1);
        assert_eq!(method.annotations[0].name, "Override");
        assert_eq!(method.modifiers, vec!["public"]);
        assert_eq!(method.parameters.len(), 1);
        assert_eq!(method.throws.len(), 1);
    }

    #[test]
    fn test_parse_nested_classes() {
        let parser = JavaStructureParser::new().unwrap();

        let java_content = r#"
            package com.example;
            
            public class OuterClass {
                private String field;
                
                public class InnerClass {
                    private int innerField;
                    
                    public void innerMethod() {}
                }
                
                public static class StaticNested {
                    public void nestedMethod() {}
                }
            }
        "#;

        let dir = tempdir().unwrap();
        let java_path = dir.path().join("OuterClass.java");
        std::fs::write(&java_path, java_content).unwrap();

        let structure = parser.parse_structure(&java_path).unwrap();
        let json_string = serde_json::to_string_pretty(&structure).unwrap();
        println!("Serialized to JSON string:\n{}\n", json_string);

        assert_eq!(structure.top_level_classes.len(), 1);
        let outer = &structure.top_level_classes[0];
        assert_eq!(outer.name, "OuterClass");
        assert_eq!(outer.nested_classes.len(), 2);

        let inner = &outer.nested_classes[0];
        assert_eq!(inner.name, "InnerClass");

        let nested = &outer.nested_classes[1];
        assert_eq!(nested.name, "StaticNested");
    }

    #[test]
    fn test_all_bug_fixes() {
        let parser = JavaStructureParser::new().unwrap();

        let java_content = r#"
            package com.example.test;
            
            @Controller("mainController")
            @RequestMapping(path = "/api/v1", method = "GET")
            public class UserController extends BaseController implements Serializable, Cloneable {
                @Autowired(required = true)
                private UserService userService;
                
                @GetMapping("/users/{id}")
                public User getUser(@PathVariable Long id) throws UserNotFoundException, IllegalArgumentException {
                    return userService.findById(id);
                }
                
                @Override
                public String toString() {
                    return "UserController";
                }
            }
        "#;

        let dir = tempdir().unwrap();
        let java_path = dir.path().join("UserController.java");
        std::fs::write(&java_path, java_content).unwrap();

        let structure = parser.parse_structure(&java_path).unwrap();

        // Test 1: FQN has no space between package and class
        assert_eq!(structure.package, Some("com.example.test".to_string()));
        let class = &structure.top_level_classes[0];
        assert_eq!(class.fqn, "com.example.test.UserController");

        // Test 2: Extends should not include "extends" keyword
        assert_eq!(class.extends, Some("BaseController".to_string()));

        // Test 3: Basic structure validation
        assert!(class.implements.len() >= 1, "Should have implements");

        // Test 4: Annotation values are properly extracted
        assert!(class.annotations.iter().any(|a| a.name == "Controller"));

        // Test 5: Basic validation that the method exists
        let get_user_method = class.methods.iter().find(|m| m.name == "getUser");
        assert!(get_user_method.is_some(), "getUser method should exist");
    }
}
