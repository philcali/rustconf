#[cfg(test)]
mod tests {
    use crate::parser::lexer::{Lexer, Token};
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
