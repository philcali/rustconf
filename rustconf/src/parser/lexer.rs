//! YANG lexer for tokenizing YANG input.
//!
//! This module provides token types and a lexer implementation using nom
//! to tokenize YANG specification files.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, multispace1},
    combinator::{map, recognize, value},
    multi::many0,
    sequence::{delimited, pair},
    IResult,
};

/// Token types for YANG language elements.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Module,
    Submodule,
    Namespace,
    Prefix,
    Import,
    Include,
    Revision,
    YangVersion,
    Organization,
    Contact,
    Description,
    Reference,

    // Data definition keywords
    Container,
    List,
    Leaf,
    LeafList,
    Choice,
    Case,
    Grouping,
    Uses,
    Typedef,
    Type,

    // Type keywords
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    String,
    Boolean,
    Enumeration,
    Enum,
    Binary,
    Bits,
    Bit,
    Union,
    LeafRef,
    IdentityRef,
    Empty,
    InstanceIdentifier,

    // Statement keywords
    Config,
    Mandatory,
    Default,
    Status,
    Units,
    Range,
    Length,
    Pattern,
    Key,
    Unique,
    MinElements,
    MaxElements,
    OrderedBy,
    Must,
    When,
    Presence,
    IfFeature,
    Feature,

    // RPC and notification keywords
    Rpc,
    Input,
    Output,
    Notification,
    Action,

    // Extension keywords
    Extension,
    Argument,
    Augment,
    Deviation,
    Deviate,
    Identity,
    Base,

    // Identifiers and literals
    Identifier(String),
    StringLiteral(String),
    Number(i64),

    // Operators and delimiters
    LeftBrace,  // {
    RightBrace, // }
    Semicolon,  // ;
    Plus,       // +
    Colon,      // :
    Slash,      // /
    Dot,        // .
    DoubleDot,  // ..
    Pipe,       // |

    // Special
    Eof,
}

impl Token {
    /// Returns true if this token is a keyword.
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::Module
                | Token::Submodule
                | Token::Namespace
                | Token::Prefix
                | Token::Import
                | Token::Include
                | Token::Revision
                | Token::YangVersion
                | Token::Organization
                | Token::Contact
                | Token::Description
                | Token::Reference
                | Token::Container
                | Token::List
                | Token::Leaf
                | Token::LeafList
                | Token::Choice
                | Token::Case
                | Token::Grouping
                | Token::Uses
                | Token::Typedef
                | Token::Type
                | Token::Int8
                | Token::Int16
                | Token::Int32
                | Token::Int64
                | Token::Uint8
                | Token::Uint16
                | Token::Uint32
                | Token::Uint64
                | Token::String
                | Token::Boolean
                | Token::Enumeration
                | Token::Enum
                | Token::Binary
                | Token::Bits
                | Token::Bit
                | Token::Union
                | Token::LeafRef
                | Token::IdentityRef
                | Token::Empty
                | Token::InstanceIdentifier
                | Token::Config
                | Token::Mandatory
                | Token::Default
                | Token::Status
                | Token::Units
                | Token::Range
                | Token::Length
                | Token::Pattern
                | Token::Key
                | Token::Unique
                | Token::MinElements
                | Token::MaxElements
                | Token::OrderedBy
                | Token::Must
                | Token::When
                | Token::Presence
                | Token::IfFeature
                | Token::Feature
                | Token::Rpc
                | Token::Input
                | Token::Output
                | Token::Notification
                | Token::Action
                | Token::Extension
                | Token::Argument
                | Token::Augment
                | Token::Deviation
                | Token::Deviate
                | Token::Identity
                | Token::Base
        )
    }
}

/// Lexer for tokenizing YANG input.
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input.
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    /// Get the next token from the input.
    pub fn next_token(&mut self) -> Result<Token, String> {
        // Skip whitespace and comments
        self.skip_whitespace_and_comments()?;

        if self.position >= self.input.len() {
            return Ok(Token::Eof);
        }

        let remaining = &self.input[self.position..];

        // Try to parse a token
        match token(remaining) {
            Ok((rest, tok)) => {
                self.position = self.input.len() - rest.len();
                Ok(tok)
            }
            Err(e) => Err(format!(
                "Lexer error at position {}: {:?}",
                self.position, e
            )),
        }
    }

    /// Tokenize the entire input into a vector of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            if tok == Token::Eof {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }
        Ok(tokens)
    }

    /// Skip whitespace and comments.
    fn skip_whitespace_and_comments(&mut self) -> Result<(), String> {
        loop {
            let remaining = &self.input[self.position..];

            // Skip whitespace
            if let Ok((rest, _)) = multispace1::<_, nom::error::Error<_>>(remaining) {
                self.position = self.input.len() - rest.len();
                continue;
            }

            // Skip single-line comments (//)
            if remaining.starts_with("//") {
                if let Some(newline_pos) = remaining.find('\n') {
                    self.position += newline_pos + 1;
                } else {
                    self.position = self.input.len();
                }
                continue;
            }

            // Skip multi-line comments (/* ... */)
            if remaining.starts_with("/*") {
                if let Some(end_pos) = remaining.find("*/") {
                    self.position += end_pos + 2;
                } else {
                    return Err("Unterminated comment".to_string());
                }
                continue;
            }

            break;
        }
        Ok(())
    }
}

/// Parse a single token.
fn token(input: &str) -> IResult<&str, Token> {
    alt((keyword, operator, number, string_literal, identifier))(input)
}

/// Parse a keyword token with word boundary checking.
fn keyword(input: &str) -> IResult<&str, Token> {
    // Try to match the longest keyword first
    let (rest, word) = recognize(pair(
        take_while1(|c: char| c.is_ascii_alphabetic() || c == '_'),
        take_while(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
    ))(input)?;

    // Map the word to a token
    let token = match word {
        "yang-version" => Token::YangVersion,
        "instance-identifier" => Token::InstanceIdentifier,
        "namespace" => Token::Namespace,
        "organization" => Token::Organization,
        "description" => Token::Description,
        "enumeration" => Token::Enumeration,
        "identityref" => Token::IdentityRef,
        "notification" => Token::Notification,
        "min-elements" => Token::MinElements,
        "max-elements" => Token::MaxElements,
        "if-feature" => Token::IfFeature,
        "ordered-by" => Token::OrderedBy,
        "mandatory" => Token::Mandatory,
        "deviation" => Token::Deviation,
        "extension" => Token::Extension,
        "reference" => Token::Reference,
        "submodule" => Token::Submodule,
        "container" => Token::Container,
        "leaf-list" => Token::LeafList,
        "identity" => Token::Identity,
        "grouping" => Token::Grouping,
        "presence" => Token::Presence,
        "argument" => Token::Argument,
        "revision" => Token::Revision,
        "augment" => Token::Augment,
        "deviate" => Token::Deviate,
        "boolean" => Token::Boolean,
        "contact" => Token::Contact,
        "typedef" => Token::Typedef,
        "leafref" => Token::LeafRef,
        "pattern" => Token::Pattern,
        "feature" => Token::Feature,
        "default" => Token::Default,
        "include" => Token::Include,
        "prefix" => Token::Prefix,
        "import" => Token::Import,
        "module" => Token::Module,
        "choice" => Token::Choice,
        "config" => Token::Config,
        "status" => Token::Status,
        "string" => Token::String,
        "action" => Token::Action,
        "binary" => Token::Binary,
        "unique" => Token::Unique,
        "length" => Token::Length,
        "output" => Token::Output,
        "uint64" => Token::Uint64,
        "uint32" => Token::Uint32,
        "uint16" => Token::Uint16,
        "uint8" => Token::Uint8,
        "int64" => Token::Int64,
        "int32" => Token::Int32,
        "int16" => Token::Int16,
        "int8" => Token::Int8,
        "range" => Token::Range,
        "units" => Token::Units,
        "union" => Token::Union,
        "input" => Token::Input,
        "empty" => Token::Empty,
        "enum" => Token::Enum,
        "bits" => Token::Bits,
        "leaf" => Token::Leaf,
        "list" => Token::List,
        "case" => Token::Case,
        "uses" => Token::Uses,
        "type" => Token::Type,
        "must" => Token::Must,
        "when" => Token::When,
        "base" => Token::Base,
        "bit" => Token::Bit,
        "key" => Token::Key,
        "rpc" => Token::Rpc,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        }
    };

    Ok((rest, token))
}

/// Parse an operator or delimiter token.
fn operator(input: &str) -> IResult<&str, Token> {
    alt((
        value(Token::LeftBrace, char('{')),
        value(Token::RightBrace, char('}')),
        value(Token::Semicolon, char(';')),
        value(Token::Plus, char('+')),
        value(Token::Colon, char(':')),
        value(Token::Slash, char('/')),
        value(Token::DoubleDot, tag("..")),
        value(Token::Dot, char('.')),
        value(Token::Pipe, char('|')),
    ))(input)
}

/// Parse a number token.
/// Only matches if the number is not immediately followed by an alphabetic character.
fn number(input: &str) -> IResult<&str, Token> {
    let (rest, num_str) = recognize(pair(
        nom::combinator::opt(char('-')),
        take_while1(|c: char| c.is_ascii_digit()),
    ))(input)?;

    // Check that the number is not followed by an alphabetic character
    // This prevents matching "10M" as a number
    if let Some(next_char) = rest.chars().next() {
        if next_char.is_ascii_alphabetic() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Digit,
            )));
        }
    }

    Ok((rest, Token::Number(num_str.parse().unwrap())))
}

/// Parse a string literal token (single or double quoted).
/// Supports multi-line strings.
fn string_literal(input: &str) -> IResult<&str, Token> {
    alt((
        // Double-quoted strings (can span multiple lines)
        delimited(
            char('"'),
            map(
                recognize(many0(alt((
                    take_while1(|c| c != '"' && c != '\\'),
                    recognize(pair(char('\\'), nom::character::complete::anychar)),
                )))),
                |s: &str| Token::StringLiteral(unescape_string(s)),
            ),
            char('"'),
        ),
        // Single-quoted strings (can span multiple lines)
        delimited(
            char('\''),
            map(
                recognize(many0(alt((
                    take_while1(|c| c != '\'' && c != '\\'),
                    recognize(pair(char('\\'), nom::character::complete::anychar)),
                )))),
                |s: &str| Token::StringLiteral(unescape_string(s)),
            ),
            char('\''),
        ),
    ))(input)
}

/// Parse an identifier token.
/// YANG identifiers can start with letters, underscores, or digits (for enum values like "10M").
fn identifier(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_'),
            take_while(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'),
        )),
        |s: &str| Token::Identifier(s.to_string()),
    )(input)
}

/// Unescape a string literal.
fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '\'' => result.push('\''),
                    '/' => result.push('/'),
                    'b' => result.push('\u{0008}'),
                    'f' => result.push('\u{000C}'),
                    _ => {
                        result.push('\\');
                        result.push(next);
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}
