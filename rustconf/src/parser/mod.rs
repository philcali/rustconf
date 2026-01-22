//! YANG specification parser module.
//!
//! This module provides functionality to parse YANG 1.0 and 1.1 specification files
//! into an abstract syntax tree (AST).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod ast;
pub mod error;
pub mod lexer;

pub use ast::*;
pub use error::ParseError;
pub use lexer::{Lexer, Token};

/// YANG parser with configurable search paths for module resolution.
pub struct YangParser {
    search_paths: Vec<PathBuf>,
    loaded_modules: HashMap<String, YangModule>,
}

impl YangParser {
    /// Create a new YANG parser with default settings.
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
            loaded_modules: HashMap::new(),
        }
    }

    /// Add a search path for resolving YANG module imports.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Parse a YANG file from the given path.
    pub fn parse_file(&mut self, path: &Path) -> Result<YangModule, ParseError> {
        let content = fs::read_to_string(path)?;
        let filename = path.to_string_lossy().to_string();
        self.parse_string(&content, &filename)
    }

    /// Parse YANG content from a string.
    pub fn parse_string(
        &mut self,
        content: &str,
        filename: &str,
    ) -> Result<YangModule, ParseError> {
        let mut lexer = Lexer::new(content);
        let tokens = lexer.tokenize().map_err(|e| ParseError::SyntaxError {
            line: 1,
            column: 1,
            message: e,
        })?;

        let mut parser = ModuleParser::new(tokens, filename);
        let module = parser.parse_module()?;

        // Try to resolve imports recursively (non-fatal if imports can't be found)
        // This allows parsing modules without having all dependencies available
        let _ = self.resolve_imports(&module);

        // Store the parsed module
        self.loaded_modules
            .insert(module.name.clone(), module.clone());

        Ok(module)
    }

    /// Resolve all imports for a module by recursively loading imported modules.
    fn resolve_imports(&mut self, module: &YangModule) -> Result<(), ParseError> {
        for import in &module.imports {
            // Skip if already loaded
            if self.loaded_modules.contains_key(&import.module) {
                continue;
            }

            // Try to find and load the imported module
            let imported_module = self.find_and_load_module(&import.module)?;

            // Recursively resolve imports of the imported module
            self.resolve_imports(&imported_module)?;

            // Store the loaded module
            self.loaded_modules
                .insert(import.module.clone(), imported_module);
        }

        Ok(())
    }

    /// Find and load a module by searching through the search paths.
    fn find_and_load_module(&mut self, module_name: &str) -> Result<YangModule, ParseError> {
        // Try each search path
        for search_path in &self.search_paths {
            // Try with .yang extension
            let module_path = search_path.join(format!("{}.yang", module_name));
            if module_path.exists() {
                let content = fs::read_to_string(&module_path)?;
                let filename = module_path.to_string_lossy().to_string();

                let mut lexer = Lexer::new(&content);
                let tokens = lexer.tokenize().map_err(|e| ParseError::SyntaxError {
                    line: 1,
                    column: 1,
                    message: e,
                })?;

                let mut parser = ModuleParser::new(tokens, &filename);
                return parser.parse_module();
            }
        }

        // Module not found in any search path
        Err(ParseError::UnresolvedImport {
            module: module_name.to_string(),
        })
    }

    /// Get a loaded module by name.
    pub fn get_loaded_module(&self, name: &str) -> Option<&YangModule> {
        self.loaded_modules.get(name)
    }

    /// Get all loaded modules.
    pub fn get_all_loaded_modules(&self) -> &HashMap<String, YangModule> {
        &self.loaded_modules
    }
}

impl Default for YangParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal parser for processing tokens into AST.
struct ModuleParser {
    tokens: Vec<Token>,
    position: usize,
    _filename: String,
}

impl ModuleParser {
    fn new(tokens: Vec<Token>, filename: &str) -> Self {
        Self {
            tokens,
            position: 0,
            _filename: filename.to_string(),
        }
    }

    /// Get the current token without consuming it.
    fn peek(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    /// Consume and return the current token.
    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        token
    }

    /// Expect a specific token and consume it, or return an error.
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let token = self.peek();
        if std::mem::discriminant(token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(format!("Expected {:?}, found {:?}", expected, token)))
        }
    }

    /// Create a syntax error at the current position.
    fn error(&self, message: String) -> ParseError {
        ParseError::SyntaxError {
            line: 1, // TODO: Track line numbers in lexer
            column: self.position,
            message,
        }
    }

    /// Parse a complete YANG module.
    fn parse_module(&mut self) -> Result<YangModule, ParseError> {
        // Expect: module <identifier> { <statements> }
        self.expect(Token::Module)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected module name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        // Parse module statements
        let mut yang_version = None;
        let mut namespace = None;
        let mut prefix = None;
        let mut imports = Vec::new();
        let mut typedefs = Vec::new();
        let mut groupings = Vec::new();
        let mut data_nodes = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::YangVersion => {
                    yang_version = Some(self.parse_yang_version()?);
                }
                Token::Namespace => {
                    namespace = Some(self.parse_namespace()?);
                }
                Token::Prefix => {
                    prefix = Some(self.parse_prefix()?);
                }
                Token::Import => {
                    imports.push(self.parse_import()?);
                }
                Token::Typedef => {
                    typedefs.push(self.parse_typedef()?);
                }
                Token::Grouping => {
                    groupings.push(self.parse_grouping()?);
                }
                Token::Container => {
                    data_nodes.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    data_nodes.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    data_nodes.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    data_nodes.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                Token::Choice => {
                    data_nodes.push(DataNode::Choice(self.parse_choice()?));
                }
                _ => {
                    // Skip unknown statements for now
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        // Validate required fields
        let namespace = namespace.ok_or_else(|| {
            self.error("Missing required 'namespace' statement in module".to_string())
        })?;
        let prefix = prefix.ok_or_else(|| {
            self.error("Missing required 'prefix' statement in module".to_string())
        })?;

        Ok(YangModule {
            name,
            namespace,
            prefix,
            yang_version,
            imports,
            typedefs,
            groupings,
            data_nodes,
            rpcs: Vec::new(),
            notifications: Vec::new(),
        })
    }

    /// Parse yang-version statement: yang-version "1.0" | "1.1" ;
    fn parse_yang_version(&mut self) -> Result<YangVersion, ParseError> {
        self.expect(Token::YangVersion)?;

        let version = match self.advance() {
            Token::StringLiteral(s) if s == "1.0" => YangVersion::V1_0,
            Token::StringLiteral(s) if s == "1.1" => YangVersion::V1_1,
            Token::Number(1) => YangVersion::V1_0,
            Token::Identifier(s) if s == "1" => YangVersion::V1_0,
            token => return Err(self.error(format!("Invalid yang-version value: {:?}", token))),
        };

        self.expect(Token::Semicolon)?;
        Ok(version)
    }

    /// Parse namespace statement: namespace <string> ;
    fn parse_namespace(&mut self) -> Result<String, ParseError> {
        self.expect(Token::Namespace)?;

        let namespace = match self.advance() {
            Token::StringLiteral(s) => s,
            token => {
                return Err(self.error(format!("Expected namespace string, found {:?}", token)))
            }
        };

        self.expect(Token::Semicolon)?;
        Ok(namespace)
    }

    /// Parse prefix statement: prefix <identifier> ;
    fn parse_prefix(&mut self) -> Result<String, ParseError> {
        self.expect(Token::Prefix)?;

        let prefix = match self.advance() {
            Token::Identifier(id) => id,
            token => {
                return Err(self.error(format!("Expected prefix identifier, found {:?}", token)))
            }
        };

        self.expect(Token::Semicolon)?;
        Ok(prefix)
    }

    /// Parse import statement: import <identifier> { prefix <identifier>; [revision-date <string>;] }
    fn parse_import(&mut self) -> Result<Import, ParseError> {
        self.expect(Token::Import)?;

        let module = match self.advance() {
            Token::Identifier(id) => id,
            token => {
                return Err(self.error(format!("Expected import module name, found {:?}", token)))
            }
        };

        self.expect(Token::LeftBrace)?;

        let mut prefix = None;
        let revision = None;

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Prefix => {
                    self.advance();
                    prefix = Some(match self.advance() {
                        Token::Identifier(id) => id,
                        token => {
                            return Err(self
                                .error(format!("Expected prefix identifier, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Revision => {
                    self.advance();
                    // Skip revision-date for now
                    self.skip_until_semicolon()?;
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        let prefix = prefix.ok_or_else(|| {
            self.error("Missing required 'prefix' statement in import".to_string())
        })?;

        Ok(Import {
            module,
            prefix,
            revision,
        })
    }

    /// Parse typedef statement: typedef <identifier> { type <type-spec>; [units <string>;] [default <string>;] [description <string>;] }
    fn parse_typedef(&mut self) -> Result<TypeDef, ParseError> {
        self.expect(Token::Typedef)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected typedef name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut type_spec = None;
        let mut units = None;
        let mut default = None;
        let mut description = None;

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Type => {
                    type_spec = Some(self.parse_type_spec()?);
                }
                Token::Units => {
                    self.advance();
                    units = Some(match self.advance() {
                        Token::StringLiteral(s) | Token::Identifier(s) => s,
                        token => {
                            return Err(
                                self.error(format!("Expected units value, found {:?}", token))
                            )
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Default => {
                    self.advance();
                    default = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        Token::Identifier(s) => s,
                        Token::Number(n) => n.to_string(),
                        token => {
                            return Err(
                                self.error(format!("Expected default value, found {:?}", token))
                            )
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Description => {
                    self.advance();
                    description = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        token => {
                            return Err(self
                                .error(format!("Expected description string, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        let type_spec = type_spec.ok_or_else(|| {
            self.error("Missing required 'type' statement in typedef".to_string())
        })?;

        Ok(TypeDef {
            name,
            type_spec,
            units,
            default,
            description,
        })
    }

    /// Parse type specification: type <type-name> [{ <type-body> }]
    fn parse_type_spec(&mut self) -> Result<TypeSpec, ParseError> {
        self.expect(Token::Type)?;

        let base_type = self.peek().clone();

        let mut type_spec = match base_type {
            Token::Int8 => {
                self.advance();
                TypeSpec::Int8 { range: None }
            }
            Token::Int16 => {
                self.advance();
                TypeSpec::Int16 { range: None }
            }
            Token::Int32 => {
                self.advance();
                TypeSpec::Int32 { range: None }
            }
            Token::Int64 => {
                self.advance();
                TypeSpec::Int64 { range: None }
            }
            Token::Uint8 => {
                self.advance();
                TypeSpec::Uint8 { range: None }
            }
            Token::Uint16 => {
                self.advance();
                TypeSpec::Uint16 { range: None }
            }
            Token::Uint32 => {
                self.advance();
                TypeSpec::Uint32 { range: None }
            }
            Token::Uint64 => {
                self.advance();
                TypeSpec::Uint64 { range: None }
            }
            Token::String => {
                self.advance();
                TypeSpec::String {
                    length: None,
                    pattern: None,
                }
            }
            Token::Boolean => {
                self.advance();
                TypeSpec::Boolean
            }
            Token::Empty => {
                self.advance();
                TypeSpec::Empty
            }
            Token::Binary => {
                self.advance();
                TypeSpec::Binary { length: None }
            }
            Token::Enumeration => {
                self.advance();
                TypeSpec::Enumeration { values: Vec::new() }
            }
            Token::Union => {
                self.advance();
                TypeSpec::Union { types: Vec::new() }
            }
            Token::LeafRef => {
                self.advance();
                TypeSpec::LeafRef {
                    path: String::new(),
                }
            }
            Token::Identifier(_) => {
                // Could be a typedef reference or other type
                // For now, treat as string type
                self.advance();
                TypeSpec::String {
                    length: None,
                    pattern: None,
                }
            }
            _ => return Err(self.error(format!("Expected type name, found {:?}", base_type))),
        };

        // Check for type body (constraints, enum values, union types, etc.)
        if self.peek() == &Token::LeftBrace {
            self.advance();
            type_spec = self.parse_type_body(type_spec)?;
            self.expect(Token::RightBrace)?;
            // No semicolon after type body - the type statement ends with the closing brace
        } else {
            // Expect semicolon after simple type (no body)
            self.expect(Token::Semicolon)?;
        }

        Ok(type_spec)
    }

    /// Parse type body (constraints, enum values, union types)
    fn parse_type_body(&mut self, mut type_spec: TypeSpec) -> Result<TypeSpec, ParseError> {
        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Range => {
                    let range = self.parse_range_constraint()?;
                    type_spec = match type_spec {
                        TypeSpec::Int8 { .. } => TypeSpec::Int8 { range: Some(range) },
                        TypeSpec::Int16 { .. } => TypeSpec::Int16 { range: Some(range) },
                        TypeSpec::Int32 { .. } => TypeSpec::Int32 { range: Some(range) },
                        TypeSpec::Int64 { .. } => TypeSpec::Int64 { range: Some(range) },
                        TypeSpec::Uint8 { .. } => TypeSpec::Uint8 { range: Some(range) },
                        TypeSpec::Uint16 { .. } => TypeSpec::Uint16 { range: Some(range) },
                        TypeSpec::Uint32 { .. } => TypeSpec::Uint32 { range: Some(range) },
                        TypeSpec::Uint64 { .. } => TypeSpec::Uint64 { range: Some(range) },
                        _ => type_spec,
                    };
                }
                Token::Length => {
                    let length = self.parse_length_constraint()?;
                    type_spec = match type_spec {
                        TypeSpec::String { pattern, .. } => TypeSpec::String {
                            length: Some(length),
                            pattern,
                        },
                        TypeSpec::Binary { .. } => TypeSpec::Binary {
                            length: Some(length),
                        },
                        _ => type_spec,
                    };
                }
                Token::Pattern => {
                    let pattern = self.parse_pattern_constraint()?;
                    type_spec = match type_spec {
                        TypeSpec::String { length, .. } => TypeSpec::String {
                            length,
                            pattern: Some(pattern),
                        },
                        _ => type_spec,
                    };
                }
                Token::Enum => {
                    let enum_value = self.parse_enum_value()?;
                    if let TypeSpec::Enumeration { ref mut values } = type_spec {
                        values.push(enum_value);
                    }
                }
                Token::Type => {
                    // Union type member
                    let member_type = self.parse_type_spec()?;
                    if let TypeSpec::Union { ref mut types } = type_spec {
                        types.push(member_type);
                    }
                }
                Token::Identifier(ref id) if id == "path" => {
                    // leafref path
                    self.advance();
                    let path = match self.advance() {
                        Token::StringLiteral(s) | Token::Identifier(s) => s,
                        token => {
                            return Err(
                                self.error(format!("Expected path value, found {:?}", token))
                            )
                        }
                    };
                    self.expect(Token::Semicolon)?;
                    if let TypeSpec::LeafRef { .. } = type_spec {
                        type_spec = TypeSpec::LeafRef { path };
                    }
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }
        Ok(type_spec)
    }

    /// Parse range constraint: range "min..max | min..max"
    fn parse_range_constraint(&mut self) -> Result<RangeConstraint, ParseError> {
        self.expect(Token::Range)?;

        let range_str = match self.advance() {
            Token::StringLiteral(s) | Token::Identifier(s) => s,
            token => return Err(self.error(format!("Expected range value, found {:?}", token))),
        };

        self.expect(Token::Semicolon)?;

        // Parse range string: "1..10 | 20..30"
        let ranges = self.parse_range_string(&range_str)?;
        Ok(RangeConstraint::new(ranges))
    }

    /// Parse range string into Range objects
    fn parse_range_string(&self, range_str: &str) -> Result<Vec<Range>, ParseError> {
        let mut ranges = Vec::new();

        for part in range_str.split('|') {
            let part = part.trim();
            if part.contains("..") {
                let parts: Vec<&str> = part.split("..").collect();
                if parts.len() == 2 {
                    let min = parts[0].trim().parse::<i64>().map_err(|_| {
                        self.error(format!("Invalid range min value: {}", parts[0]))
                    })?;
                    let max = parts[1].trim().parse::<i64>().map_err(|_| {
                        self.error(format!("Invalid range max value: {}", parts[1]))
                    })?;
                    ranges.push(Range::new(min, max));
                }
            } else {
                // Single value range
                let value = part
                    .parse::<i64>()
                    .map_err(|_| self.error(format!("Invalid range value: {}", part)))?;
                ranges.push(Range::new(value, value));
            }
        }

        Ok(ranges)
    }

    /// Parse length constraint: length "min..max | min..max"
    fn parse_length_constraint(&mut self) -> Result<LengthConstraint, ParseError> {
        self.expect(Token::Length)?;

        let length_str = match self.advance() {
            Token::StringLiteral(s) | Token::Identifier(s) => s,
            token => return Err(self.error(format!("Expected length value, found {:?}", token))),
        };

        self.expect(Token::Semicolon)?;

        // Parse length string: "1..10 | 20..30"
        let lengths = self.parse_length_string(&length_str)?;
        Ok(LengthConstraint::new(lengths))
    }

    /// Parse length string into LengthRange objects
    fn parse_length_string(&self, length_str: &str) -> Result<Vec<LengthRange>, ParseError> {
        let mut lengths = Vec::new();

        for part in length_str.split('|') {
            let part = part.trim();
            if part.contains("..") {
                let parts: Vec<&str> = part.split("..").collect();
                if parts.len() == 2 {
                    let min = parts[0].trim().parse::<u64>().map_err(|_| {
                        self.error(format!("Invalid length min value: {}", parts[0]))
                    })?;
                    let max = parts[1].trim().parse::<u64>().map_err(|_| {
                        self.error(format!("Invalid length max value: {}", parts[1]))
                    })?;
                    lengths.push(LengthRange::new(min, max));
                }
            } else {
                // Single value length
                let value = part
                    .parse::<u64>()
                    .map_err(|_| self.error(format!("Invalid length value: {}", part)))?;
                lengths.push(LengthRange::new(value, value));
            }
        }

        Ok(lengths)
    }

    /// Parse pattern constraint: pattern "regex"
    fn parse_pattern_constraint(&mut self) -> Result<PatternConstraint, ParseError> {
        self.expect(Token::Pattern)?;

        let pattern = match self.advance() {
            Token::StringLiteral(s) => s,
            token => return Err(self.error(format!("Expected pattern string, found {:?}", token))),
        };

        self.expect(Token::Semicolon)?;

        Ok(PatternConstraint::new(pattern))
    }

    /// Parse enum value: enum <identifier> [{ value <number>; description <string>; }]
    fn parse_enum_value(&mut self) -> Result<EnumValue, ParseError> {
        self.expect(Token::Enum)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected enum name, found {:?}", token))),
        };

        let mut value = None;
        let mut description = None;

        if self.peek() == &Token::LeftBrace {
            self.advance();

            while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
                match self.peek() {
                    Token::Identifier(ref id) if id == "value" => {
                        self.advance();
                        value = Some(match self.advance() {
                            Token::Number(n) => n as i32,
                            token => {
                                return Err(self.error(format!(
                                    "Expected enum value number, found {:?}",
                                    token
                                )))
                            }
                        });
                        self.expect(Token::Semicolon)?;
                    }
                    Token::Description => {
                        self.advance();
                        description = Some(match self.advance() {
                            Token::StringLiteral(s) => s,
                            token => {
                                return Err(self.error(format!(
                                    "Expected description string, found {:?}",
                                    token
                                )))
                            }
                        });
                        self.expect(Token::Semicolon)?;
                    }
                    _ => {
                        self.skip_statement()?;
                    }
                }
            }

            self.expect(Token::RightBrace)?;
        } else {
            self.expect(Token::Semicolon)?;
        }

        Ok(EnumValue {
            name,
            value,
            description,
        })
    }

    /// Parse grouping statement: grouping <identifier> { <data-definition-statements> }
    fn parse_grouping(&mut self) -> Result<Grouping, ParseError> {
        self.expect(Token::Grouping)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected grouping name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut data_nodes = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    self.advance();
                    description = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        token => {
                            return Err(self
                                .error(format!("Expected description string, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Container => {
                    data_nodes.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    data_nodes.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    data_nodes.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    data_nodes.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Grouping {
            name,
            description,
            data_nodes,
        })
    }

    /// Parse container statement: container <identifier> { <statements> }
    fn parse_container(&mut self) -> Result<Container, ParseError> {
        self.expect(Token::Container)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected container name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut config = true;
        let mut mandatory = false;
        let mut children = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    self.advance();
                    description = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        token => {
                            return Err(self
                                .error(format!("Expected description string, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Config => {
                    self.advance();
                    config = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Mandatory => {
                    self.advance();
                    mandatory = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Container => {
                    children.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    children.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    children.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    children.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                Token::Choice => {
                    children.push(DataNode::Choice(self.parse_choice()?));
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Container {
            name,
            description,
            config,
            mandatory,
            children,
        })
    }

    /// Parse list statement: list <identifier> { <statements> }
    fn parse_list(&mut self) -> Result<List, ParseError> {
        self.expect(Token::List)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected list name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut config = true;
        let mut keys = Vec::new();
        let mut children = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    self.advance();
                    description = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        token => {
                            return Err(self
                                .error(format!("Expected description string, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Config => {
                    self.advance();
                    config = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Key => {
                    self.advance();
                    let key_str = match self.advance() {
                        Token::StringLiteral(s) | Token::Identifier(s) => s,
                        token => {
                            return Err(self.error(format!("Expected key value, found {:?}", token)))
                        }
                    };
                    // Split space-separated keys
                    keys.extend(key_str.split_whitespace().map(String::from));
                    self.expect(Token::Semicolon)?;
                }
                Token::Container => {
                    children.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    children.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    children.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    children.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                Token::Choice => {
                    children.push(DataNode::Choice(self.parse_choice()?));
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(List {
            name,
            description,
            config,
            keys,
            children,
        })
    }

    /// Parse leaf statement: leaf <identifier> { type <type-spec>; <statements> }
    fn parse_leaf(&mut self) -> Result<Leaf, ParseError> {
        self.expect(Token::Leaf)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected leaf name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut type_spec = None;
        let mut description = None;
        let mut mandatory = false;
        let mut default = None;
        let mut config = true;

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Type => {
                    type_spec = Some(self.parse_type_spec()?);
                }
                Token::Description => {
                    self.advance();
                    description = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        token => {
                            return Err(self
                                .error(format!("Expected description string, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Mandatory => {
                    self.advance();
                    mandatory = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Default => {
                    self.advance();
                    default = Some(match self.advance() {
                        Token::StringLiteral(s) | Token::Identifier(s) => s,
                        Token::Number(n) => n.to_string(),
                        token => {
                            return Err(
                                self.error(format!("Expected default value, found {:?}", token))
                            )
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Config => {
                    self.advance();
                    config = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        let type_spec = type_spec
            .ok_or_else(|| self.error("Missing required 'type' statement in leaf".to_string()))?;

        Ok(Leaf {
            name,
            description,
            type_spec,
            mandatory,
            default,
            config,
        })
    }

    /// Parse leaf-list statement: leaf-list <identifier> { type <type-spec>; <statements> }
    fn parse_leaf_list(&mut self) -> Result<LeafList, ParseError> {
        self.expect(Token::LeafList)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected leaf-list name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut type_spec = None;
        let mut description = None;
        let mut config = true;

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Type => {
                    type_spec = Some(self.parse_type_spec()?);
                }
                Token::Description => {
                    self.advance();
                    description = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        token => {
                            return Err(self
                                .error(format!("Expected description string, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Config => {
                    self.advance();
                    config = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        let type_spec = type_spec.ok_or_else(|| {
            self.error("Missing required 'type' statement in leaf-list".to_string())
        })?;

        Ok(LeafList {
            name,
            description,
            type_spec,
            config,
        })
    }

    /// Parse choice statement: choice <identifier> { <case-statements> }
    fn parse_choice(&mut self) -> Result<Choice, ParseError> {
        self.expect(Token::Choice)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected choice name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut mandatory = false;
        let mut cases = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    self.advance();
                    description = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        token => {
                            return Err(self
                                .error(format!("Expected description string, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Mandatory => {
                    self.advance();
                    mandatory = match self.advance() {
                        Token::Identifier(s) if s == "true" => true,
                        Token::Identifier(s) if s == "false" => false,
                        token => {
                            return Err(self
                                .error(format!("Expected 'true' or 'false', found {:?}", token)))
                        }
                    };
                    self.expect(Token::Semicolon)?;
                }
                Token::Case => {
                    cases.push(self.parse_case()?);
                }
                // Shorthand: data nodes directly in choice are implicitly wrapped in a case
                Token::Container | Token::List | Token::Leaf | Token::LeafList => {
                    let data_node = match self.peek() {
                        Token::Container => DataNode::Container(self.parse_container()?),
                        Token::List => DataNode::List(self.parse_list()?),
                        Token::Leaf => DataNode::Leaf(self.parse_leaf()?),
                        Token::LeafList => DataNode::LeafList(self.parse_leaf_list()?),
                        _ => unreachable!(),
                    };
                    // Create an implicit case with the same name as the data node
                    let case_name = match &data_node {
                        DataNode::Container(c) => c.name.clone(),
                        DataNode::List(l) => l.name.clone(),
                        DataNode::Leaf(l) => l.name.clone(),
                        DataNode::LeafList(l) => l.name.clone(),
                        _ => unreachable!(),
                    };
                    cases.push(Case {
                        name: case_name,
                        description: None,
                        data_nodes: vec![data_node],
                    });
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Choice {
            name,
            description,
            mandatory,
            cases,
        })
    }

    /// Parse case statement: case <identifier> { <data-definition-statements> }
    fn parse_case(&mut self) -> Result<Case, ParseError> {
        self.expect(Token::Case)?;

        let name = match self.advance() {
            Token::Identifier(id) => id,
            token => return Err(self.error(format!("Expected case name, found {:?}", token))),
        };

        self.expect(Token::LeftBrace)?;

        let mut description = None;
        let mut data_nodes = Vec::new();

        while self.peek() != &Token::RightBrace && self.peek() != &Token::Eof {
            match self.peek() {
                Token::Description => {
                    self.advance();
                    description = Some(match self.advance() {
                        Token::StringLiteral(s) => s,
                        token => {
                            return Err(self
                                .error(format!("Expected description string, found {:?}", token)))
                        }
                    });
                    self.expect(Token::Semicolon)?;
                }
                Token::Container => {
                    data_nodes.push(DataNode::Container(self.parse_container()?));
                }
                Token::List => {
                    data_nodes.push(DataNode::List(self.parse_list()?));
                }
                Token::Leaf => {
                    data_nodes.push(DataNode::Leaf(self.parse_leaf()?));
                }
                Token::LeafList => {
                    data_nodes.push(DataNode::LeafList(self.parse_leaf_list()?));
                }
                Token::Choice => {
                    data_nodes.push(DataNode::Choice(self.parse_choice()?));
                }
                _ => {
                    self.skip_statement()?;
                }
            }
        }

        self.expect(Token::RightBrace)?;

        Ok(Case {
            name,
            description,
            data_nodes,
        })
    }

    /// Skip an unknown statement (consume until semicolon or closing brace).
    fn skip_statement(&mut self) -> Result<(), ParseError> {
        // Skip the statement keyword
        self.advance();

        // Skip until we find a semicolon or handle a block
        match self.peek() {
            Token::Semicolon => {
                self.advance();
                Ok(())
            }
            Token::LeftBrace => {
                self.advance();
                self.skip_block()?;
                Ok(())
            }
            _ => {
                // Skip identifier/string/number and then expect semicolon or block
                self.advance();
                match self.peek() {
                    Token::Semicolon => {
                        self.advance();
                        Ok(())
                    }
                    Token::LeftBrace => {
                        self.advance();
                        self.skip_block()?;
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }
        }
    }

    /// Skip a block (everything between { and }).
    fn skip_block(&mut self) -> Result<(), ParseError> {
        let mut depth = 1;
        while depth > 0 && self.peek() != &Token::Eof {
            match self.advance() {
                Token::LeftBrace => depth += 1,
                Token::RightBrace => depth -= 1,
                _ => {}
            }
        }
        Ok(())
    }

    /// Skip until semicolon.
    fn skip_until_semicolon(&mut self) -> Result<(), ParseError> {
        while self.peek() != &Token::Semicolon && self.peek() != &Token::Eof {
            self.advance();
        }
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "mod.test.rs"]
mod mod_tests;
