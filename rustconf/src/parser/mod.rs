//! YANG specification parser module.
//!
//! This module provides functionality to parse YANG 1.0 and 1.1 specification files
//! into an abstract syntax tree (AST).

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
}

impl YangParser {
    /// Create a new YANG parser with default settings.
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
        }
    }

    /// Add a search path for resolving YANG module imports.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Parse a YANG file from the given path.
    pub fn parse_file(&self, path: &Path) -> Result<YangModule, ParseError> {
        let content = fs::read_to_string(path)?;
        let filename = path.to_string_lossy().to_string();
        self.parse_string(&content, &filename)
    }

    /// Parse YANG content from a string.
    pub fn parse_string(&self, content: &str, filename: &str) -> Result<YangModule, ParseError> {
        let mut lexer = Lexer::new(content);
        let tokens = lexer.tokenize().map_err(|e| ParseError::SyntaxError {
            line: 1,
            column: 1,
            message: e,
        })?;

        let mut parser = ModuleParser::new(tokens, filename);
        parser.parse_module()
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
            typedefs: Vec::new(),
            groupings: Vec::new(),
            data_nodes: Vec::new(),
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
