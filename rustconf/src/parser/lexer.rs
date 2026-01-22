//! YANG lexer for tokenizing YANG input.
//!
//! This module provides token types and a lexer implementation using nom
//! to tokenize YANG specification files.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, multispace1, one_of},
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
fn number(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            nom::combinator::opt(char('-')),
            take_while1(|c: char| c.is_ascii_digit()),
        )),
        |s: &str| Token::Number(s.parse().unwrap()),
    )(input)
}

/// Parse a string literal token (single or double quoted).
fn string_literal(input: &str) -> IResult<&str, Token> {
    alt((
        delimited(
            char('"'),
            map(
                recognize(many0(alt((
                    take_while1(|c| c != '"' && c != '\\'),
                    recognize(pair(char('\\'), one_of(r#""\/bfnrt"#))),
                )))),
                |s: &str| Token::StringLiteral(unescape_string(s)),
            ),
            char('"'),
        ),
        delimited(
            char('\''),
            map(
                recognize(many0(alt((
                    take_while1(|c| c != '\'' && c != '\\'),
                    recognize(pair(char('\\'), one_of(r#"'\/bfnrt"#))),
                )))),
                |s: &str| Token::StringLiteral(unescape_string(s)),
            ),
            char('\''),
        ),
    ))(input)
}

/// Parse an identifier token.
fn identifier(input: &str) -> IResult<&str, Token> {
    map(
        recognize(pair(
            take_while1(|c: char| c.is_ascii_alphabetic() || c == '_'),
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Property-based test generators

    /// Generate valid YANG keywords
    fn yang_keyword() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("module"),
            Just("namespace"),
            Just("prefix"),
            Just("import"),
            Just("container"),
            Just("list"),
            Just("leaf"),
            Just("leaf-list"),
            Just("type"),
            Just("string"),
            Just("int32"),
            Just("uint32"),
            Just("boolean"),
            Just("config"),
            Just("mandatory"),
            Just("description"),
        ]
    }

    /// Generate valid YANG identifiers
    fn yang_identifier() -> impl Strategy<Value = String> {
        "[a-zA-Z_][a-zA-Z0-9_-]{0,20}".prop_filter("Not a keyword", |s| {
            !matches!(
                s.as_str(),
                "module"
                    | "namespace"
                    | "prefix"
                    | "import"
                    | "container"
                    | "list"
                    | "leaf"
                    | "leaf-list"
                    | "type"
                    | "string"
                    | "int32"
                    | "uint32"
                    | "boolean"
                    | "config"
                    | "mandatory"
                    | "description"
                    | "yang-version"
                    | "organization"
                    | "contact"
                    | "reference"
                    | "revision"
                    | "typedef"
                    | "grouping"
                    | "uses"
                    | "choice"
                    | "case"
                    | "rpc"
                    | "input"
                    | "output"
                    | "notification"
            )
        })
    }

    /// Generate valid string literals
    fn yang_string_literal() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 _-]{0,30}".prop_map(|s| format!("\"{}\"", s))
    }

    /// Generate valid numbers
    fn yang_number() -> impl Strategy<Value = String> {
        (-1000i64..1000i64).prop_map(|n| n.to_string())
    }

    /// Generate valid operators
    fn yang_operator() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("{"),
            Just("}"),
            Just(";"),
            Just(":"),
            Just("."),
            Just(".."),
        ]
    }

    /// Generate a valid YANG token sequence
    fn yang_token_sequence() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                yang_keyword().prop_map(|s| s.to_string()),
                yang_identifier(),
                yang_string_literal(),
                yang_number(),
                yang_operator().prop_map(|s| s.to_string()),
            ],
            1..20,
        )
        .prop_map(|tokens| tokens.join(" "))
    }

    /// Generate a simple valid YANG module structure
    fn simple_yang_module() -> impl Strategy<Value = String> {
        (yang_identifier(), yang_identifier(), yang_string_literal()).prop_map(
            |(name, prefix, namespace)| {
                format!(
                    "module {} {{ namespace {}; prefix {}; }}",
                    name, namespace, prefix
                )
            },
        )
    }

    // Property-based tests

    proptest! {
        /// Property 1: Valid YANG Parsing Success (partial - lexing phase)
        /// For any valid YANG token sequence, lexing should succeed without errors
        /// Validates: Requirements 1.1, 1.2
        #[test]
        fn prop_valid_yang_tokens_lex_successfully(input in yang_token_sequence()) {
            let mut lexer = Lexer::new(&input);
            let result = lexer.tokenize();
            prop_assert!(result.is_ok(), "Lexer should successfully tokenize valid YANG tokens: {:?}", result.err());
        }

        /// Property 1: Valid YANG Parsing Success (partial - lexing phase)
        /// For any simple valid YANG module, lexing should succeed
        /// Validates: Requirements 1.1, 1.2
        #[test]
        fn prop_simple_yang_module_lexes_successfully(input in simple_yang_module()) {
            let mut lexer = Lexer::new(&input);
            let result = lexer.tokenize();
            prop_assert!(result.is_ok(), "Lexer should successfully tokenize simple YANG module: {:?}", result.err());

            let tokens = result.unwrap();
            // Should have at least: module, identifier, {, namespace, string, ;, prefix, identifier, ;, }, EOF
            prop_assert!(tokens.len() >= 11, "Simple module should produce at least 11 tokens, got {}", tokens.len());
            prop_assert_eq!(&tokens[0], &Token::Module);
            prop_assert_eq!(tokens.last().unwrap(), &Token::Eof);
        }

        /// Property: Keywords are correctly identified
        /// For any YANG keyword, it should be lexed as the correct keyword token
        #[test]
        fn prop_keywords_identified_correctly(keyword in yang_keyword()) {
            let mut lexer = Lexer::new(keyword);
            let result = lexer.next_token();
            prop_assert!(result.is_ok());
            let token = result.unwrap();
            prop_assert!(token.is_keyword(), "Token {:?} should be identified as a keyword", token);
        }

        /// Property: Identifiers are correctly lexed
        /// For any valid identifier, it should be lexed as an Identifier token
        #[test]
        fn prop_identifiers_lexed_correctly(id in yang_identifier()) {
            let mut lexer = Lexer::new(&id);
            let result = lexer.next_token();
            prop_assert!(result.is_ok());
            if let Token::Identifier(lexed_id) = result.unwrap() {
                prop_assert_eq!(lexed_id, id);
            } else {
                return Err(TestCaseError::fail("Expected Identifier token"));
            }
        }

        /// Property: Numbers are correctly lexed
        /// For any valid number, it should be lexed as a Number token with correct value
        #[test]
        fn prop_numbers_lexed_correctly(num_str in yang_number()) {
            let mut lexer = Lexer::new(&num_str);
            let result = lexer.next_token();
            prop_assert!(result.is_ok());
            if let Token::Number(n) = result.unwrap() {
                let expected: i64 = num_str.parse().unwrap();
                prop_assert_eq!(n, expected);
            } else {
                return Err(TestCaseError::fail("Expected Number token"));
            }
        }

        /// Property: Whitespace is properly handled
        /// For any token sequence with varying whitespace, lexing should succeed
        #[test]
        fn prop_whitespace_handling(
            tokens in prop::collection::vec(yang_keyword(), 1..10),
            spaces in prop::collection::vec(prop_oneof![Just(" "), Just("\n"), Just("\t")], 1..5)
        ) {
            let input = tokens.iter()
                .zip(spaces.iter().cycle())
                .map(|(t, s)| format!("{}{}", t, s))
                .collect::<String>();

            let mut lexer = Lexer::new(&input);
            let result = lexer.tokenize();
            prop_assert!(result.is_ok(), "Lexer should handle whitespace correctly: {:?}", result.err());
        }
    }

    // Unit tests

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("module namespace prefix");
        assert_eq!(lexer.next_token().unwrap(), Token::Module);
        assert_eq!(lexer.next_token().unwrap(), Token::Namespace);
        assert_eq!(lexer.next_token().unwrap(), Token::Prefix);
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("my-module my_identifier test123");
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("my-module".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("my_identifier".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::Identifier("test123".to_string())
        );
    }

    #[test]
    fn test_string_literals() {
        let mut lexer = Lexer::new(r#""hello world" 'single quoted'"#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::StringLiteral("hello world".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::StringLiteral("single quoted".to_string())
        );
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 -17 0");
        assert_eq!(lexer.next_token().unwrap(), Token::Number(42));
        assert_eq!(lexer.next_token().unwrap(), Token::Number(-17));
        assert_eq!(lexer.next_token().unwrap(), Token::Number(0));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("{ } ; : .. .");
        assert_eq!(lexer.next_token().unwrap(), Token::LeftBrace);
        assert_eq!(lexer.next_token().unwrap(), Token::RightBrace);
        assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
        assert_eq!(lexer.next_token().unwrap(), Token::Colon);
        assert_eq!(lexer.next_token().unwrap(), Token::DoubleDot);
        assert_eq!(lexer.next_token().unwrap(), Token::Dot);
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("module // comment\nnamespace /* block comment */ prefix");
        assert_eq!(lexer.next_token().unwrap(), Token::Module);
        assert_eq!(lexer.next_token().unwrap(), Token::Namespace);
        assert_eq!(lexer.next_token().unwrap(), Token::Prefix);
    }

    #[test]
    fn test_whitespace_handling() {
        let mut lexer = Lexer::new("  module  \n  namespace  \t  prefix  ");
        assert_eq!(lexer.next_token().unwrap(), Token::Module);
        assert_eq!(lexer.next_token().unwrap(), Token::Namespace);
        assert_eq!(lexer.next_token().unwrap(), Token::Prefix);
    }

    #[test]
    fn test_escaped_strings() {
        let mut lexer = Lexer::new(r#""hello\nworld" "tab\there""#);
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::StringLiteral("hello\nworld".to_string())
        );
        assert_eq!(
            lexer.next_token().unwrap(),
            Token::StringLiteral("tab\there".to_string())
        );
    }

    #[test]
    fn test_tokenize_simple_module() {
        let input = r#"
            module example {
                namespace "http://example.com";
                prefix ex;
            }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0], Token::Module);
        assert_eq!(tokens[1], Token::Identifier("example".to_string()));
        assert_eq!(tokens[2], Token::LeftBrace);
        assert_eq!(tokens[3], Token::Namespace);
        assert_eq!(
            tokens[4],
            Token::StringLiteral("http://example.com".to_string())
        );
        assert_eq!(tokens[5], Token::Semicolon);
        assert_eq!(tokens[6], Token::Prefix);
        assert_eq!(tokens[7], Token::Identifier("ex".to_string()));
        assert_eq!(tokens[8], Token::Semicolon);
        assert_eq!(tokens[9], Token::RightBrace);
        assert_eq!(tokens[10], Token::Eof);
    }

    #[test]
    fn test_type_keywords() {
        let mut lexer = Lexer::new("int8 uint32 string boolean");
        assert_eq!(lexer.next_token().unwrap(), Token::Int8);
        assert_eq!(lexer.next_token().unwrap(), Token::Uint32);
        assert_eq!(lexer.next_token().unwrap(), Token::String);
        assert_eq!(lexer.next_token().unwrap(), Token::Boolean);
    }

    #[test]
    fn test_data_definition_keywords() {
        let mut lexer = Lexer::new("container list leaf leaf-list");
        assert_eq!(lexer.next_token().unwrap(), Token::Container);
        assert_eq!(lexer.next_token().unwrap(), Token::List);
        assert_eq!(lexer.next_token().unwrap(), Token::Leaf);
        assert_eq!(lexer.next_token().unwrap(), Token::LeafList);
    }
}
