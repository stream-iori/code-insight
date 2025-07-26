use anyhow::{Context, Result};
use std::path::Path;
use tree_sitter::{Node, Parser};
use tree_sitter_java::language;

use crate::types::{
    Annotation, Declaration, DeclarationKind, Field, JavaFile, Method, Parameter, SourceRange,
};

/// Categorized enum for tree-sitter Java node kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JavaNodeKind {
    // Declaration types
    ModuleDeclaration,
    PackageDeclaration,
    ImportDeclaration,
    ClassDeclaration,
    InterfaceDeclaration,
    EnumDeclaration,
    RecordDeclaration,
    AnnotationTypeDeclaration,
    FieldDeclaration,
    MethodDeclaration,
    ConstructorDeclaration,
    
    // Modifiers and annotations
    Modifier,
    Annotation,
    
    // Identifiers and references
    Identifier,
    ScopedIdentifier,
    Asterisk,
    
    // Type-related
    TypeIdentifier,
    IntegralType,
    FloatingPointType,
    BooleanType,
    VoidType,
    GenericType,
    ArrayType,
    
    // Parameters and variables
    FormalParameters,
    FormalParameter,
    VariableDeclarator,
    
    // Inheritance
    Superclass,
    SuperInterfaces,
    
    // Literals and values
    StringLiteral,
    NumberLiteral,
    True,
    False,
    
    // Comments and documentation
    Comment,
    
    // Annotation elements
    ElementValuePair,
    
    // Access modifiers
    Public,
    Private,
    Protected,
    Static,
    Final,
    Abstract,
    Synchronized,
    Volatile,
    Transient,
    Native,
    Strictfp,
    
    Unknown,
}

impl JavaNodeKind {
    fn from_str(kind: &str) -> Self {
        match kind {
            "module_declaration" => JavaNodeKind::ModuleDeclaration,
            "package_declaration" => JavaNodeKind::PackageDeclaration,
            "import_declaration" => JavaNodeKind::ImportDeclaration,
            "class_declaration" => JavaNodeKind::ClassDeclaration,
            "interface_declaration" => JavaNodeKind::InterfaceDeclaration,
            "enum_declaration" => JavaNodeKind::EnumDeclaration,
            "record_declaration" => JavaNodeKind::RecordDeclaration,
            "annotation_type_declaration" => JavaNodeKind::AnnotationTypeDeclaration,
            "field_declaration" => JavaNodeKind::FieldDeclaration,
            "method_declaration" => JavaNodeKind::MethodDeclaration,
            "constructor_declaration" => JavaNodeKind::ConstructorDeclaration,
            "annotation" => JavaNodeKind::Annotation,
            "modifier" => JavaNodeKind::Modifier,
            "identifier" => JavaNodeKind::Identifier,
            "scoped_identifier" => JavaNodeKind::ScopedIdentifier,
            "asterisk" => JavaNodeKind::Asterisk,
            "superclass" => JavaNodeKind::Superclass,
            "super_interfaces" => JavaNodeKind::SuperInterfaces,
            "type_identifier" => JavaNodeKind::TypeIdentifier,
            "integral_type" => JavaNodeKind::IntegralType,
            "floating_point_type" => JavaNodeKind::FloatingPointType,
            "boolean_type" => JavaNodeKind::BooleanType,
            "void_type" => JavaNodeKind::VoidType,
            "generic_type" => JavaNodeKind::GenericType,
            "array_type" => JavaNodeKind::ArrayType,
            "formal_parameters" => JavaNodeKind::FormalParameters,
            "formal_parameter" => JavaNodeKind::FormalParameter,
            "variable_declarator" => JavaNodeKind::VariableDeclarator,
            "comment" => JavaNodeKind::Comment,
            "string_literal" => JavaNodeKind::StringLiteral,
            "number_literal" => JavaNodeKind::NumberLiteral,
            "true" => JavaNodeKind::True,
            "false" => JavaNodeKind::False,
            "public" => JavaNodeKind::Public,
            "private" => JavaNodeKind::Private,
            "protected" => JavaNodeKind::Protected,
            "static" => JavaNodeKind::Static,
            "final" => JavaNodeKind::Final,
            "abstract" => JavaNodeKind::Abstract,
            "synchronized" => JavaNodeKind::Synchronized,
            "volatile" => JavaNodeKind::Volatile,
            "transient" => JavaNodeKind::Transient,
            "native" => JavaNodeKind::Native,
            "strictfp" => JavaNodeKind::Strictfp,
            "element_value_pair" => JavaNodeKind::ElementValuePair,
            _ => JavaNodeKind::Unknown,
        }
    }

    // Category methods for better organization
    pub fn is_declaration(self) -> bool {
        matches!(
            self,
            JavaNodeKind::ClassDeclaration
                | JavaNodeKind::InterfaceDeclaration
                | JavaNodeKind::EnumDeclaration
                | JavaNodeKind::RecordDeclaration
                | JavaNodeKind::AnnotationTypeDeclaration
        )
    }

    pub fn is_modifier(self) -> bool {
        matches!(
            self,
            JavaNodeKind::Modifier
                | JavaNodeKind::Public
                | JavaNodeKind::Private
                | JavaNodeKind::Protected
                | JavaNodeKind::Static
                | JavaNodeKind::Final
                | JavaNodeKind::Abstract
                | JavaNodeKind::Synchronized
                | JavaNodeKind::Volatile
                | JavaNodeKind::Transient
                | JavaNodeKind::Native
                | JavaNodeKind::Strictfp
        )
    }

    pub fn is_type(self) -> bool {
        matches!(
            self,
            JavaNodeKind::TypeIdentifier
                | JavaNodeKind::IntegralType
                | JavaNodeKind::FloatingPointType
                | JavaNodeKind::BooleanType
                | JavaNodeKind::VoidType
                | JavaNodeKind::GenericType
                | JavaNodeKind::ArrayType
        )
    }

    pub fn to_declaration_kind(self) -> Option<DeclarationKind> {
        match self {
            JavaNodeKind::ClassDeclaration => Some(DeclarationKind::Class),
            JavaNodeKind::InterfaceDeclaration => Some(DeclarationKind::Interface),
            JavaNodeKind::EnumDeclaration => Some(DeclarationKind::Enum),
            JavaNodeKind::RecordDeclaration => Some(DeclarationKind::Record),
            JavaNodeKind::AnnotationTypeDeclaration => Some(DeclarationKind::Annotation),
            _ => None,
        }
    }
}

/// A Java source code parser that generates fully qualified signatures for declarations.
///
/// This parser uses tree-sitter to parse Java source code and generates
/// fully qualified names (FQN) for class, interface, enum, and other declarations.
///
/// # Examples
///
/// ```
/// use code_insight::parser::JavaParser;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let mut parser = JavaParser::new().unwrap();
/// let java_content = r#"
///     package com.example.service;
///     
///     public class UserService {
///         private String name;
///     }
/// "#;
/// 
/// let dir = tempdir().unwrap();
/// let java_path = dir.path().join("UserService.java");
/// fs::write(&java_path, java_content).unwrap();
/// 
/// let java_file = parser.parse_file(&java_path).unwrap();
/// assert_eq!(java_file.declarations[0].signature, "public class com.example.service.UserService");
/// ```
///
/// # FQN Signatures with Modifiers
///
/// This parser generates fully qualified names (FQN) that include:
/// - Access modifiers (public, private, protected)
/// - Other modifiers (static, final, abstract, etc.)
/// - Package name as part of the FQN
/// - Declaration type (class, interface, enum, etc.)
///
/// ```
/// use code_insight::parser::JavaParser;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let mut parser = JavaParser::new().unwrap();
/// 
/// // Example with multiple modifiers
/// let java_content = r#"
///     package com.example.util;
///     
///     public final class StringUtils {
///         public static String trim(String input) {
///             return input.trim();
///         }
///     }
/// "#;
/// 
/// let dir = tempdir().unwrap();
/// let java_path = dir.path().join("StringUtils.java");
/// fs::write(&java_path, java_content).unwrap();
/// 
/// let java_file = parser.parse_file(&java_path).unwrap();
/// let signature = java_file.declarations[0].signature.clone();
/// assert_eq!(signature, "public final class com.example.util.StringUtils");
/// 
/// // Example with interface
/// let interface_content = r#"
///     package com.example.api;
///     
///     public interface Repository {
///         void save(Object obj);
///     }
/// "#;
/// 
/// let interface_path = dir.path().join("Repository.java");
/// fs::write(&interface_path, interface_content).unwrap();
/// 
/// let interface_file = parser.parse_file(&interface_path).unwrap();
/// let interface_signature = interface_file.declarations[0].signature.clone();
/// assert_eq!(interface_signature, "public interface com.example.api.Repository");
/// 
/// // Example without package (default package)
/// let default_content = r#"
///     public class DefaultClass {
///         private int value;
///     }
/// "#;
/// 
/// let default_path = dir.path().join("DefaultClass.java");
/// fs::write(&default_path, default_content).unwrap();
/// 
/// let default_file = parser.parse_file(&default_path).unwrap();
/// let default_signature = default_file.declarations[0].signature.clone();
/// assert_eq!(default_signature, "public class DefaultClass");
/// ```
pub struct JavaParser {
    //tree-sitter的parser
    parser: Parser,
}

impl JavaParser {
    pub fn new() -> Result<Self> {
        //init tree-sitter parser
        let mut parser = Parser::new();
        parser
            .set_language(language())
            .context("Error loading Java grammar")?;

        Ok(Self { parser })
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<JavaFile> {
        /*
         * !<learning>
         * 读取指定的java source
         * anyhow 针对result提供了with_context
         */
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read Java file: {:?}", path))?;

        /* !<learning> anyhow 针对option 提供了 context */
        let tree = self
            .parser
            .parse(&source, None)
            .context("Failed to parse Java file")?;

        //构建 Java File 对象
        let mut java_file = JavaFile {
            path: path.to_path_buf(),
            module: None, //暂时用不到,目前没有使用
            package: String::new(),
            imports: Vec::new(),
            declarations: Vec::new(), //如果有属性的话
            source_hash: format!("{:x}", md5::compute(&source)),
        };

        let root_node = tree.root_node();
        self.parse_root(&root_node, &source, &mut java_file)?;

        Ok(java_file)
    }

    fn parse_root(&self, node: &Node, source: &str, java_file: &mut JavaFile) -> Result<()> {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            let kind = JavaNodeKind::from_str(child.kind());
            match kind {
                JavaNodeKind::ModuleDeclaration => {
                    java_file.module = Some(self.get_node_text(&child, source)?);
                }
                JavaNodeKind::PackageDeclaration => {
                    java_file.package = self.get_package_name(&child, source)?;
                }
                JavaNodeKind::ImportDeclaration => {
                    if let Some(import) = self.get_import_name(&child, source)? {
                        java_file.imports.push(import);
                    }
                }
                kind if kind.is_declaration() => {
                    if let Some(declaration) = self.parse_declaration(&child, source, java_file)? {
                        java_file.declarations.push(declaration);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn parse_declaration(&self, node: &Node, source: &str, java_file: &JavaFile) -> Result<Option<Declaration>> {
        let node_kind = JavaNodeKind::from_str(node.kind());
        let kind = match node_kind.to_declaration_kind() {
            Some(k) => k,
            None => return Ok(None),
        };

        let name = self.get_declaration_name(node, source)?;
        let modifiers = self.get_modifiers(node, source)?;
        let annotations = self.get_annotations(node, source)?;
        let signature = self.get_signature(node, source, java_file)?;

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
            let kind = JavaNodeKind::from_str(child.kind());
            if kind == JavaNodeKind::Identifier {
                return self.get_node_text(&child, source);
            }
        }
        Ok("Anonymous".to_string())
    }

    fn get_modifiers(&self, node: &Node, source: &str) -> Result<Vec<String>> {
        let mut modifiers = Vec::new();
        
        // Get the full text of the declaration node
        let declaration_text = self.get_node_text(node, source)?;
        
        // Common Java modifiers to look for
        let java_modifiers = [
            "public", "private", "protected", "static", "final", "abstract",
            "synchronized", "volatile", "transient", "native", "strictfp"
        ];
        
        // Split the declaration text into tokens and look for modifiers
        let tokens: Vec<&str> = declaration_text.split_whitespace().collect();
        
        for token in tokens {
            let clean_token = token.trim();
            if java_modifiers.contains(&clean_token) && !modifiers.contains(&clean_token.to_string()) {
                modifiers.push(clean_token.to_string());
            }
            
            // Stop when we hit the class/interface/enum keyword
            if matches!(clean_token, "class" | "interface" | "enum" | "record" | "@interface") {
                break;
            }
        }

        Ok(modifiers)
    }

    fn get_annotations(&self, node: &Node, source: &str) -> Result<Vec<Annotation>> {
        let mut annotations = Vec::new();

        for child in node.children(&mut node.walk()) {
            let kind = JavaNodeKind::from_str(child.kind());
            if kind == JavaNodeKind::Annotation {
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
            let kind = JavaNodeKind::from_str(child.kind());
            match kind {
                JavaNodeKind::Identifier => {
                    if name.is_none() {
                        name = Some(self.get_node_text(&child, source)?);
                    }
                }
                JavaNodeKind::ElementValuePair => {
                    if let Some((key, value)) = self.parse_annotation_value(&child, source)? {
                        values.push((key, value));
                    }
                }
                _ => {}
            }
        }

        Ok(name.map(|n| Annotation { name: n, values }))
    }

    fn parse_annotation_value(
        &self,
        node: &Node,
        source: &str,
    ) -> Result<Option<(String, String)>> {
        let mut cursor = node.walk();
        let mut key = None;
        let mut value = None;

        for child in node.children(&mut cursor) {
            let kind = JavaNodeKind::from_str(child.kind());
            match kind {
                JavaNodeKind::Identifier => {
                    if key.is_none() {
                        key = Some(self.get_node_text(&child, source)?);
                    }
                }
                JavaNodeKind::StringLiteral
                | JavaNodeKind::NumberLiteral
                | JavaNodeKind::True
                | JavaNodeKind::False => {
                    value = Some(self.get_node_text(&child, source)?);
                }
                _ => {}
            }
        }

        Ok(key.zip(value))
    }

    /// Generates a fully qualified signature for Java declarations
    ///
    /// This method constructs signatures that include the package name,
    /// providing fully qualified names for classes, interfaces, enums, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// // With package
    /// // Input: package com.example; public class UserService {}
    /// // Output: "public class com.example.UserService"
    ///
    /// // Without package (default package)
    /// // Input: public class DefaultClass {}
    /// // Output: "public class DefaultClass"
    ///
    /// // Interface with package
    /// // Input: package com.api; public interface Repository {}
    /// // Output: "public interface com.api.Repository"
    /// ```
    fn get_signature(
        &self,
        node: &Node,
        source: &str,
        java_file: &JavaFile,
    ) -> Result<String> {
        let node_kind = JavaNodeKind::from_str(node.kind());
        let kind = match node_kind.to_declaration_kind() {
            Some(k) => k,
            None => return self.get_node_text(node, source),
        };

        let name = self.get_declaration_name(node, source)?;
        let modifiers = self.get_modifiers(node, source)?;
        
        let mut signature_parts = Vec::new();
        
        // Add modifiers
        if !modifiers.is_empty() {
            signature_parts.extend(modifiers);
        }
        
        // Add kind and name with FQN
        match kind {
            DeclarationKind::Class => signature_parts.push("class".to_string()),
            DeclarationKind::Interface => signature_parts.push("interface".to_string()),
            DeclarationKind::Enum => signature_parts.push("enum".to_string()),
            DeclarationKind::Record => signature_parts.push("record".to_string()),
            DeclarationKind::Annotation => signature_parts.push("@interface".to_string()),
        }
        
        // Use fully qualified name
        let fqn = if java_file.package.is_empty() {
            name
        } else {
            format!("{}.{}", java_file.package, name)
        };
        signature_parts.push(fqn);
        
        Ok(signature_parts.join(" "))
    }

    fn get_inheritance_info(
        &self,
        node: &Node,
        source: &str,
    ) -> Result<(Option<String>, Vec<String>)> {
        let mut extends = None;
        let mut implements = Vec::new();

        for child in node.children(&mut node.walk()) {
            let kind = JavaNodeKind::from_str(child.kind());
            match kind {
                JavaNodeKind::Superclass => {
                    if let Some(type_node) = child.child_by_field_name("type") {
                        extends = Some(self.get_node_text(&type_node, source)?);
                    }
                }
                JavaNodeKind::SuperInterfaces => {
                    for interface in child.children(&mut child.walk()) {
                        let interface_kind = JavaNodeKind::from_str(interface.kind());
                        if interface_kind == JavaNodeKind::TypeIdentifier {
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

        //语义驱动,获取指定的Node
        let body = node.child_by_field_name("body");
        if let Some(body) = body {
            for child in body.children(&mut body.walk()) {
                let kind = JavaNodeKind::from_str(child.kind());
                if kind == JavaNodeKind::FieldDeclaration {
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
        //下面是field包含的字段
        let mut name = None;
        let mut type_name = None;
        let mut modifiers = Vec::new();
        let mut annotations = Vec::new();

        for child in node.children(&mut cursor) {
            let kind = JavaNodeKind::from_str(child.kind());
            match kind {
                JavaNodeKind::Modifier => {
                    if let Ok(text) = self.get_node_text(&child, source) {
                        modifiers.push(text);
                    }
                }
                JavaNodeKind::Annotation => {
                    if let Some(annotation) = self.parse_annotation(&child, source)? {
                        annotations.push(annotation);
                    }
                }
                JavaNodeKind::VariableDeclarator => {
                    if let Some(identifier) = child.child_by_field_name("name") {
                        name = Some(self.get_node_text(&identifier, source)?);
                    }
                }
                kind if kind.is_type() => {
                    type_name = Some(self.get_node_text(&child, source)?);
                }
                _ => {
                    // Handle nested type structures
                    if type_name.is_none() {
                        let mut type_cursor = child.walk();
                        for type_child in child.children(&mut type_cursor) {
                            let type_child_kind = JavaNodeKind::from_str(type_child.kind());
                            if type_child_kind.is_type() || type_child_kind == JavaNodeKind::Identifier {
                                type_name = Some(self.get_node_text(&type_child, source)?);
                            }
                        }
                    }
                }
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
                let kind = JavaNodeKind::from_str(child.kind());
                if kind == JavaNodeKind::MethodDeclaration {
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
            let kind = JavaNodeKind::from_str(child.kind());
            match kind {
                JavaNodeKind::Modifier => {
                    if let Ok(text) = self.get_node_text(&child, source) {
                        modifiers.push(text);
                    }
                }
                JavaNodeKind::Annotation => {
                    if let Some(annotation) = self.parse_annotation(&child, source)? {
                        annotations.push(annotation);
                    }
                }
                JavaNodeKind::Identifier => {
                    if name.is_none() {
                        name = Some(self.get_node_text(&child, source)?);
                    }
                }
                JavaNodeKind::FormalParameters => {
                    parameters = self.parse_parameters(&child, source)?;
                }
                kind if kind.is_type() => {
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
            let kind = JavaNodeKind::from_str(child.kind());
            if kind == JavaNodeKind::FormalParameter {
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
            let kind = JavaNodeKind::from_str(child.kind());
            match kind {
                JavaNodeKind::Annotation => {
                    if let Some(annotation) = self.parse_annotation(&child, source)? {
                        annotations.push(annotation);
                    }
                }
                JavaNodeKind::Identifier => {
                    if name.is_none() {
                        name = Some(self.get_node_text(&child, source)?);
                    }
                }
                JavaNodeKind::TypeIdentifier
                | JavaNodeKind::IntegralType
                | JavaNodeKind::FloatingPointType
                | JavaNodeKind::BooleanType
                | JavaNodeKind::VoidType => {
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
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = JavaNodeKind::from_str(child.kind());
            match kind {
                JavaNodeKind::ScopedIdentifier | JavaNodeKind::Identifier => {
                    return self.get_node_text(&child, source);
                }
                _ => {
                    // Recursively look for identifier within nested structures
                    if let Ok(text) = self.get_node_text(&child, source) {
                        if !text.is_empty() && text != "package" && !text.contains(";") {
                            return Ok(text.trim().to_string());
                        }
                    }
                }
            }
        }
        Ok(String::new())
    }

    fn get_import_name(&self, node: &Node, source: &str) -> Result<Option<String>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = JavaNodeKind::from_str(child.kind());
            match kind {
                JavaNodeKind::ScopedIdentifier | JavaNodeKind::Identifier | JavaNodeKind::Asterisk => {
                    return Ok(Some(self.get_node_text(&child, source)?));
                }
                _ => {
                    // Recursively look for import path, import 语法最多两层
                    let mut import_cursor = child.walk();
                    for import_child in child.children(&mut import_cursor) {
                        let import_child_kind = JavaNodeKind::from_str(import_child.kind());
                        match import_child_kind {
                            JavaNodeKind::ScopedIdentifier | JavaNodeKind::Identifier => {
                                return Ok(Some(self.get_node_text(&import_child, source)?));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(None)
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
            let kind = JavaNodeKind::from_str(child.kind());
            if kind == JavaNodeKind::Comment {
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
        
        // Debug the actual structure
        println!("Declaration: {:?}", declaration);
        println!("Modifiers: {:?}", declaration.modifiers);
        println!("Signature: {}", declaration.signature);
        
        // Test FQN signature with modifiers and package
        let expected_signature = "public class com.example.UserService";
        assert_eq!(declaration.signature, expected_signature);
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
        
        // Test FQN signature for interface
        let expected_signature = "public interface com.example.api.UserRepository";
        assert_eq!(declaration.signature, expected_signature);
    }

    #[test]
    fn test_fqn_signatures() {
        let mut parser = JavaParser::new().unwrap();
        
        // Test with package
        let java_with_package = r#"
            package com.test.nested;
            
            public class TestClass {
                private int value;
            }
        "#;
        
        let dir = tempdir().unwrap();
        let java_path = dir.path().join("TestClass.java");
        std::fs::write(&java_path, java_with_package).unwrap();
        
        let java_file = parser.parse_file(&java_path).unwrap();
        assert_eq!(java_file.declarations.len(), 1);
        assert_eq!(java_file.declarations[0].signature, "public class com.test.nested.TestClass");
        
        // Test without package (default package)
        let java_without_package = r#"
            public class DefaultPackageClass {
                private String name;
            }
        "#;
        
        let java_path2 = dir.path().join("DefaultPackageClass.java");
        std::fs::write(&java_path2, java_without_package).unwrap();
        
        let java_file2 = parser.parse_file(&java_path2).unwrap();
        assert_eq!(java_file2.declarations.len(), 1);
        assert_eq!(java_file2.declarations[0].signature, "public class DefaultPackageClass");
    }
}
