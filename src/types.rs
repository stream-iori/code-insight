use serde::{Deserialize, Serialize};
use std::path::PathBuf;


/// A class, interface, enum, etc. found in Java code
/// This is like "I found a class called UserService"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Declaration {
    /// Name of the class/interface/etc.
    pub name: String,
    /// What type of declaration this is (class, interface, etc.)
    pub kind: DeclarationKind,
    /// Keywords like "public", "private", "static"
    pub modifiers: Vec<String>,
    /// Annotations like @Service, @RestController
    pub annotations: Vec<Annotation>,
    /// The full signature line (e.g., "public class UserService")
    pub signature: String,
    /// What class this extends (if any)
    pub extends: Option<String>,
    /// What interfaces this implements
    pub implements: Vec<String>,
    /// Fields (variables) inside this class
    pub fields: Vec<Field>,
    /// Methods (functions) inside this class
    pub methods: Vec<Method>,
    /// Where in the file this appears (line numbers)
    pub range: SourceRange,
    /// JavaDoc comments above this declaration
    pub documentation: Option<String>,
}

/// Different types of Java declarations you can find
/// Think: "Is this a class? An interface? An enum?"
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DeclarationKind {
    /// A regular class like "public class UserService"
    Class,
    /// An interface like "public interface UserRepository"
    Interface,
    /// An enum like "public enum UserStatus { ACTIVE, INACTIVE }"
    Enum,
    /// A record (newer Java feature) like "public record User(String name, int age)"
    Record,
    /// An annotation like "@interface MyAnnotation"
    Annotation,
}

/// A field (variable) inside a Java class
/// Example: "private String username;"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Field name (e.g., "username")
    pub name: String,
    /// Field type (e.g., "String", "int", "List<User>")
    pub type_name: String,
    /// Modifiers like "private", "final"
    pub modifiers: Vec<String>,
    /// Annotations like @NotNull, @Size(min=3)
    pub annotations: Vec<Annotation>,
}

/// A method (function) inside a Java class
/// Example: "public User findUserById(Long id) { ... }"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    /// Method name (e.g., "findUserById")
    pub name: String,
    /// Return type (e.g., "User", "void", "List<String>")
    pub return_type: String,
    /// Method parameters
    pub parameters: Vec<Parameter>,
    /// Modifiers like "public", "static"
    pub modifiers: Vec<String>,
    /// Annotations like @GetMapping, @Transactional
    pub annotations: Vec<Annotation>,
    /// Where the method signature appears in file
    pub range: SourceRange,
    /// Where the method body starts and ends
    pub body_range: Option<SourceRange>,
}

/// A parameter in a method
/// Example: "Long id" in "findUserById(Long id)"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name (e.g., "id")
    pub name: String,
    /// Parameter type (e.g., "Long", "String")
    pub type_name: String,
    /// Annotations like @Valid, @NotNull
    pub annotations: Vec<Annotation>,
}

/// An annotation like @Service, @RestController, @NotNull
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    /// Annotation name (e.g., "Service", "NotNull")
    pub name: String,
    /// Key-value pairs inside the annotation
    /// Example: @Size(min=5, max=50) becomes [("min", "5"), ("max", "50")]
    pub values: Vec<(String, String)>,
}

/// Location in source code (line and column numbers)
/// Useful for showing "this class is on line 15, column 5"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRange {
    /// Line number where this starts (1-based)
    pub start_line: usize,
    /// Column number where this starts (1-based)
    pub start_column: usize,
    /// Line number where this ends
    pub end_line: usize,
    /// Column number where this ends
    pub end_column: usize,
}

/// An XML file (.xml) that might contain Spring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlFile {
    /// Full path to the .xml file
    pub path: PathBuf,
    /// Root XML element name
    pub root_element: String,
    /// Raw XML content
    pub content: String,
}

/// A properties file (.properties) with key=value pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertiesFile {
    /// Full path to the .properties file
    pub path: PathBuf,
    /// All key=value pairs found in the file
    pub properties: Vec<(String, String)>,
}

/// Search query for finding code
/// Like "find me all classes named UserService"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// What to search for (e.g., "UserService")
    pub query: String,
    /// How to search (exact match, fuzzy, or regex)
    pub kind: SearchKind,
    /// Additional filters (by type, annotation, etc.)
    pub filters: Vec<SearchFilter>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

/// Different ways to search for code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchKind {
    /// Exact match ("UserService" must match exactly)
    Exact,
    /// Fuzzy match ("UserServ" might match "UserService")
    Fuzzy,
    /// Regular expression match
    Regex,
}

/// Ways to filter search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchFilter {
    /// Only find classes, interfaces, etc.
    Kind(DeclarationKind),
    /// Only classes with specific annotation
    Annotation(String),
    /// Only in specific package
    Package(String),
    /// Only in specific module
    Module(String),
}

/// Search result from the index
/// "I found UserService.java, here's what I found"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The actual class/interface/etc. found
    pub declaration: Declaration,
    /// Which file it was found in
    pub file_path: PathBuf,
    /// How well it matches the search (0.0 to 1.0)
    pub score: f32,
    /// Short preview text
    pub preview: String,
}

/// Data exported for AI/LLM systems
/// Clean, structured format for AI tools to consume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmExport {
    /// Name of the class/method/etc.
    pub name: String,
    /// Type ("class", "interface", "method", etc.)
    pub kind: String,
    /// Signature line (e.g., "public class UserService")
    pub signature: String,
    /// JavaDoc if available
    pub documentation: Option<String>,
    /// Actual source code
    pub code: String,
    /// Relative file path
    pub file_path: String,
    /// Line numbers (start, end)
    pub line_range: (usize, usize),
}

/// Relationship graph between classes
/// Shows how classes connect to each other
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceGraph {
    /// All the classes/interfaces found
    pub nodes: Vec<GraphNode>,
    /// How they relate to each other
    pub edges: Vec<GraphEdge>,
}

/// A single class/interface in the relationship graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier (usually the full class name)
    pub id: String,
    /// Display label (usually just the class name)
    pub label: String,
    /// What type of declaration this is
    pub kind: DeclarationKind,
    /// Where this file is located
    pub file_path: PathBuf,
}

/// A relationship between two classes
/// Like "UserService extends BaseService" or "UserService uses UserRepository"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source class (from)
    pub from: String,
    /// Target class (to)
    pub to: String,
    /// Type of relationship
    pub relationship: RelationshipType,
}

/// Different types of relationships between classes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    /// Class inheritance ("extends")
    Extends,
    /// Interface implementation ("implements")
    Implements,
    /// Usage relationship ("uses")
    Uses,
    /// Reference relationship
    References,
    /// Dependency relationship
    DependsOn,
}