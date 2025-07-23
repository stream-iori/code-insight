use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::{Parser, Node};
use tree_sitter_java::language;

use crate::types::{
    JavaFile, Declaration, DeclarationKind, Field, Method, Parameter, 
    Annotation, SourceRange
};

pub struct JavaParser {
    parser: Parser,
}

impl JavaParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(language())
            .context("Error loading Java grammar")?;
        
        Ok(Self { parser })
    }
    
    pub fn parse_file(&mut self, path: &Path) -> Result<JavaFile> {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read Java file: {:?}", path))?;
        
        let tree = self.parser.parse(&source, None)
            .context("Failed to parse Java file")?;
        
        let mut java_file = JavaFile {
            path: path.to_path_buf(),
            module: None,
            package: String::new(),
            imports: Vec::new(),
            declarations: Vec::new(),
            source_hash: format!("{:x}", md5::compute(&source)),
        };
        
        let root_node = tree.root_node();
        self.parse_root(&root_node, &source, &mut java_file)?;
        
        Ok(java_file)
    }
    
    fn parse_root(&self, node: &Node, source: &str, java_file: &mut JavaFile) -> Result<()> {
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "module_declaration" => {
                    java_file.module = Some(self.get_node_text(&child, source)?);
                }
                "package_declaration" => {
                    java_file.package = self.get_package_name(&child, source)?;
                }
                "import_declaration" => {
                    if let Some(import) = self.get_import_name(&child, source)? {
                        java_file.imports.push(import);
                    }
                }
                "class_declaration" | "interface_declaration" | "enum_declaration" | "record_declaration" | "annotation_type_declaration" => {
                    if let Some(declaration) = self.parse_declaration(&child, source)? {
                        java_file.declarations.push(declaration);
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn parse_declaration(&self, node: &Node, source: &str) -> Result<Option<Declaration>> {
        let kind = match node.kind() {
            "class_declaration" => DeclarationKind::Class,
            "interface_declaration" => DeclarationKind::Interface,
            "enum_declaration" => DeclarationKind::Enum,
            "record_declaration" => DeclarationKind::Record,
            "annotation_type_declaration" => DeclarationKind::Annotation,
            _ => return Ok(None),
        };
        
        let name = self.get_declaration_name(node, source)?;
        let modifiers = self.get_modifiers(node, source)?;
        let annotations = self.get_annotations(node, source)?;
        let signature = self.get_signature(node, source)?;
        
        let (extends, implements) = self.get_inheritance_info(node, source)?;
        let fields = self.get_fields(node, source)?;
        let methods = self.get_methods(node, source)?;
        
        let range = self.get_source_range(node);
        let documentation = self.get_documentation(node, source)?;
        
        Ok(Some(Declaration {
            name,
            kind,
            modifiers,
            annotations,
            signature,
            extends,
            implements,
            fields,
            methods,
            range,
            documentation,
        }))
    }
    
    fn get_declaration_name(&self, node: &Node, source: &str) -> Result<String> {
        for child in node.children(&mut node.walk()) {
            if child.kind() == "identifier" {
                return self.get_node_text(&child, source);
            }
        }
        Ok("Anonymous".to_string())
    }
    
    fn get_modifiers(&self, node: &Node, source: &str) -> Result<Vec<String>> {
        let mut modifiers = Vec::new();
        
        for child in node.children(&mut node.walk()) {
            if child.kind().ends_with("_modifier") {
                if let Ok(text) = self.get_node_text(&child, source) {
                    modifiers.push(text);
                }
            }
        }
        
        Ok(modifiers)
    }
    
    fn get_annotations(&self, node: &Node, source: &str) -> Result<Vec<Annotation>> {
        let mut annotations = Vec::new();
        
        for child in node.children(&mut node.walk()) {
            if child.kind() == "annotation" {
                if let Some(annotation) = self.parse_annotation(&child, source)? {
                    annotations.push(annotation);
                }
            }
        }
        
        Ok(annotations)
    }
    
    fn parse_annotation(&self, node: &Node, source: &str) -> Result<Option<Annotation>> {
        let mut cursor = node.walk();
        let mut name = None;
        let mut values = Vec::new();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if name.is_none() {
                        name = Some(self.get_node_text(&child, source)?);
                    }
                }
                "element_value_pair" => {
                    if let Some((key, value)) = self.parse_annotation_value(&child, source)? {
                        values.push((key, value));
                    }
                }
                _ => {}
            }
        }
        
        Ok(name.map(|n| Annotation {
            name: n,
            values,
        }))
    }
    
    fn parse_annotation_value(&self, node: &Node, source: &str) -> Result<Option<(String, String)>> {
        let mut cursor = node.walk();
        let mut key = None;
        let mut value = None;
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    if key.is_none() {
                        key = Some(self.get_node_text(&child, source)?);
                    }
                }
                "string_literal" | "number_literal" | "true" | "false" => {
                    value = Some(self.get_node_text(&child, source)?);
                }
                _ => {}
            }
        }
        
        Ok(key.zip(value))
    }
    
    fn get_signature(&self, node: &Node, source: &str) -> Result<String> {
        self.get_node_text(node, source)
    }
    
    fn get_inheritance_info(&self, node: &Node, source: &str) -> Result<(Option<String>, Vec<String>)> {
        let mut extends = None;
        let mut implements = Vec::new();
        
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "superclass" => {
                    if let Some(type_node) = child.child_by_field_name("type") {
                        extends = Some(self.get_node_text(&type_node, source)?);
                    }
                }
                "super_interfaces" => {
                    for interface in child.children(&mut child.walk()) {
                        if interface.kind() == "type_identifier" {
                            if let Ok(name) = self.get_node_text(&interface, source) {
                                implements.push(name);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        Ok((extends, implements))
    }
    
    fn get_fields(&self, node: &Node, source: &str) -> Result<Vec<Field>> {
        let mut fields = Vec::new();
        
        let body = node.child_by_field_name("body");
        if let Some(body) = body {
            for child in body.children(&mut body.walk()) {
                if child.kind() == "field_declaration" {
                    if let Some(field) = self.parse_field(&child, source)? {
                        fields.push(field);
                    }
                }
            }
        }
        
        Ok(fields)
    }
    
    fn parse_field(&self, node: &Node, source: &str) -> Result<Option<Field>> {
        let mut cursor = node.walk();
        let mut name = None;
        let mut type_name = None;
        let mut modifiers = Vec::new();
        let mut annotations = Vec::new();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "modifier" => {
                    if let Ok(text) = self.get_node_text(&child, source) {
                        modifiers.push(text);
                    }
                }
                "annotation" => {
                    if let Some(annotation) = self.parse_annotation(&child, source)? {
                        annotations.push(annotation);
                    }
                }
                "variable_declarator" => {
                    if let Some(identifier) = child.child_by_field_name("name") {
                        name = Some(self.get_node_text(&identifier, source)?);
                    }
                }
                "type_identifier" | "integral_type" | "floating_point_type" | "boolean_type" | "void_type" => {
                    type_name = Some(self.get_node_text(&child, source)?);
                }
                _ => {}
            }
        }
        
        Ok(name.zip(type_name).map(|(n, t)| Field {
            name: n,
            type_name: t,
            modifiers,
            annotations,
            range: self.get_source_range(node),
        }))
    }
    
    fn get_methods(&self, node: &Node, source: &str) -> Result<Vec<Method>> {
        let mut methods = Vec::new();
        
        let body = node.child_by_field_name("body");
        if let Some(body) = body {
            for child in body.children(&mut body.walk()) {
                if child.kind() == "method_declaration" {
                    if let Some(method) = self.parse_method(&child, source)? {
                        methods.push(method);
                    }
                }
            }
        }
        
        Ok(methods)
    }
    
    fn parse_method(&self, node: &Node, source: &str) -> Result<Option<Method>> {
        let mut cursor = node.walk();
        let mut name = None;
        let mut return_type = None;
        let mut parameters = Vec::new();
        let mut modifiers = Vec::new();
        let mut annotations = Vec::new();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "modifier" => {
                    if let Ok(text) = self.get_node_text(&child, source) {
                        modifiers.push(text);
                    }
                }
                "annotation" => {
                    if let Some(annotation) = self.parse_annotation(&child, source)? {
                        annotations.push(annotation);
                    }
                }
                "identifier" => {
                    if name.is_none() {
                        name = Some(self.get_node_text(&child, source)?);
                    }
                }
                "formal_parameters" => {
                    parameters = self.parse_parameters(&child, source)?;
                }
                "type_identifier" | "integral_type" | "floating_point_type" | "boolean_type" | "void_type" => {
                    return_type = Some(self.get_node_text(&child, source)?);
                }
                _ => {}
            }
        }
        
        Ok(name.map(|n| Method {
            name: n,
            return_type: return_type.unwrap_or_else(|| "void".to_string()),
            parameters,
            modifiers,
            annotations,
            range: self.get_source_range(node),
            body_range: self.get_method_body_range(&node),
        }))
    }
    
    fn parse_parameters(&self, node: &Node, source: &str) -> Result<Vec<Parameter>> {
        let mut parameters = Vec::new();
        
        for child in node.children(&mut node.walk()) {
            if child.kind() == "formal_parameter" {
                if let Some(param) = self.parse_parameter(&child, source)? {
                    parameters.push(param);
                }
            }
        }
        
        Ok(parameters)
    }
    
    fn parse_parameter(&self, node: &Node, source: &str) -> Result<Option<Parameter>> {
        let mut cursor = node.walk();
        let mut name = None;
        let mut type_name = None;
        let mut annotations = Vec::new();
        
        for child in node.children(&mut cursor) {
            match child.kind() {
                "annotation" => {
                    if let Some(annotation) = self.parse_annotation(&child, source)? {
                        annotations.push(annotation);
                    }
                }
                "identifier" => {
                    if name.is_none() {
                        name = Some(self.get_node_text(&child, source)?);
                    }
                }
                "type_identifier" | "integral_type" | "floating_point_type" | "boolean_type" | "void_type" => {
                    type_name = Some(self.get_node_text(&child, source)?);
                }
                _ => {}
            }
        }
        
        Ok(name.zip(type_name).map(|(n, t)| Parameter {
            name: n,
            type_name: t,
            annotations,
        }))
    }
    
    fn get_package_name(&self, node: &Node, source: &str) -> Result<String> {
        if let Some(name_node) = node.child_by_field_name("name") {
            self.get_node_text(&name_node, source)
        } else {
            Ok(String::new())
        }
    }
    
    fn get_import_name(&self, node: &Node, source: &str) -> Result<Option<String>> {
        if let Some(name_node) = node.child_by_field_name("name") {
            Ok(Some(self.get_node_text(&name_node, source)?))
        } else {
            Ok(None)
        }
    }
    
    fn get_node_text(&self, node: &Node, source: &str) -> Result<String> {
        let start = node.start_byte();
        let end = node.end_byte();
        let text = &source[start..end];
        Ok(text.to_string())
    }
    
    fn get_source_range(&self, node: &Node) -> SourceRange {
        let start = node.start_position();
        let end = node.end_position();
        
        SourceRange {
            start_line: start.row + 1,
            start_column: start.column + 1,
            end_line: end.row + 1,
            end_column: end.column + 1,
        }
    }
    
    fn get_method_body_range(&self, node: &Node) -> Option<SourceRange> {
        if let Some(body) = node.child_by_field_name("body") {
            Some(self.get_source_range(&body))
        } else {
            None
        }
    }
    
    fn get_documentation(&self, node: &Node, source: &str) -> Result<Option<String>> {
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "comment" {
                let text = self.get_node_text(&child, source)?;
                if text.starts_with("/**") {
                    return Ok(Some(text));
                }
            }
        }
        
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_parse_simple_class() {
        let mut parser = JavaParser::new().unwrap();
        let java_content = r#"
            package com.example;
            
            import java.util.List;
            
            /**
             * A simple service class
             */
            @Service
            public class UserService {
                private final UserRepository repository;
                
                @Autowired
                public UserService(UserRepository repository) {
                    this.repository = repository;
                }
                
                public List<User> getAllUsers() {
                    return repository.findAll();
                }
            }
        "#;
        
        let dir = tempdir().unwrap();
        let java_path = dir.path().join("UserService.java");
        std::fs::write(&java_path, java_content).unwrap();
        
        let java_file = parser.parse_file(&java_path).unwrap();
        
        assert_eq!(java_file.package, "com.example");
        assert_eq!(java_file.imports.len(), 1);
        assert_eq!(java_file.imports[0], "java.util.List");
        assert_eq!(java_file.declarations.len(), 1);
        
        let declaration = &java_file.declarations[0];
        assert_eq!(declaration.name, "UserService");
        assert!(matches!(declaration.kind, DeclarationKind::Class));
        assert!(declaration.modifiers.contains(&"public".to_string()));
        assert_eq!(declaration.annotations.len(), 1);
        assert_eq!(declaration.annotations[0].name, "Service");
        assert_eq!(declaration.fields.len(), 1);
        assert_eq!(declaration.methods.len(), 2);
    }
    
    #[test]
    fn test_parse_interface() {
        let mut parser = JavaParser::new().unwrap();
        let java_content = r#"
            package com.example.api;
            
            public interface UserRepository {
                List<User> findAll();
                User findById(Long id);
                void save(User user);
            }
        "#;
        
        let dir = tempdir().unwrap();
        let java_path = dir.path().join("UserRepository.java");
        std::fs::write(&java_path, java_content).unwrap();
        
        let java_file = parser.parse_file(&java_path).unwrap();
        
        assert_eq!(java_file.declarations.len(), 1);
        let declaration = &java_file.declarations[0];
        assert_eq!(declaration.name, "UserRepository");
        assert!(matches!(declaration.kind, DeclarationKind::Interface));
        assert_eq!(declaration.methods.len(), 3);
    }
}