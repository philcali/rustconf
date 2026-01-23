//! Naming convention conversion utilities for YANG to Rust identifier mapping.

/// Convert a YANG identifier to snake_case for Rust fields and functions.
///
/// YANG typically uses kebab-case (e.g., "interface-name"), which needs to be
/// converted to snake_case (e.g., "interface_name") for Rust.
///
/// # Examples
///
/// ```
/// # use rustconf::generator::naming::to_snake_case;
/// assert_eq!(to_snake_case("interface-name"), "interface_name");
/// assert_eq!(to_snake_case("IPAddress"), "ip_address");
/// assert_eq!(to_snake_case("already_snake"), "already_snake");
/// assert_eq!(to_snake_case("HTTPSConnection"), "https_connection");
/// ```
pub fn to_snake_case(identifier: &str) -> String {
    if identifier.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(identifier.len() + 4);
    let mut chars = identifier.chars().peekable();
    let mut prev_was_upper = false;
    let mut prev_was_delimiter = true;

    while let Some(ch) = chars.next() {
        match ch {
            // Convert delimiters to underscores
            '-' | ' ' | '.' => {
                if !result.is_empty() && !prev_was_delimiter {
                    result.push('_');
                    prev_was_delimiter = true;
                }
                prev_was_upper = false;
            }
            // Keep underscores as-is
            '_' => {
                if !result.is_empty() && !prev_was_delimiter {
                    result.push('_');
                    prev_was_delimiter = true;
                }
                prev_was_upper = false;
            }
            // Handle uppercase letters
            c if c.is_uppercase() => {
                let next_is_lower = chars.peek().is_some_and(|&n| n.is_lowercase());

                // Add underscore before uppercase if:
                // - Not at start
                // - Previous wasn't uppercase, OR
                // - Previous was uppercase but next is lowercase (e.g., "HTTPSConnection" -> "https_connection")
                if !result.is_empty() && !prev_was_delimiter && (!prev_was_upper || next_is_lower) {
                    result.push('_');
                }

                result.push(c.to_lowercase().next().unwrap());
                prev_was_upper = true;
                prev_was_delimiter = false;
            }
            // Handle other characters
            c => {
                result.push(c);
                prev_was_upper = false;
                prev_was_delimiter = false;
            }
        }
    }

    result
}

/// Convert a YANG identifier to PascalCase for Rust types.
///
/// YANG typically uses kebab-case (e.g., "interface-config"), which needs to be
/// converted to PascalCase (e.g., "InterfaceConfig") for Rust types.
///
/// # Examples
///
/// ```
/// # use rustconf::generator::naming::to_pascal_case;
/// assert_eq!(to_pascal_case("interface-config"), "InterfaceConfig");
/// assert_eq!(to_pascal_case("ip-address"), "IpAddress");
/// assert_eq!(to_pascal_case("already_snake"), "AlreadySnake");
/// assert_eq!(to_pascal_case("PascalCase"), "PascalCase");
/// ```
pub fn to_pascal_case(identifier: &str) -> String {
    if identifier.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(identifier.len());
    let mut capitalize_next = true;

    for ch in identifier.chars() {
        match ch {
            // Delimiters trigger capitalization of next character
            '-' | '_' | ' ' | '.' => {
                capitalize_next = true;
            }
            // Capitalize or keep as-is
            c => {
                if capitalize_next {
                    result.extend(c.to_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c);
                }
            }
        }
    }

    result
}

/// Escape Rust keywords by appending an underscore.
///
/// If the identifier is a Rust keyword, append an underscore to make it valid.
/// Otherwise, return the identifier unchanged.
///
/// # Examples
///
/// ```
/// # use rustconf::generator::naming::escape_keyword;
/// assert_eq!(escape_keyword("type"), "type_");
/// assert_eq!(escape_keyword("match"), "match_");
/// assert_eq!(escape_keyword("interface"), "interface");
/// assert_eq!(escape_keyword("self"), "self_");
/// ```
pub fn escape_keyword(identifier: &str) -> String {
    if is_rust_keyword(identifier) {
        format!("{}_", identifier)
    } else {
        identifier.to_string()
    }
}

/// Check if an identifier is a Rust keyword.
fn is_rust_keyword(identifier: &str) -> bool {
    // Rust keywords from the Rust reference
    // https://doc.rust-lang.org/reference/keywords.html
    static KEYWORDS: &[&str] = &[
        // Strict keywords (always reserved)
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
        "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
        "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
        "where", "while", // Keywords reserved for future use
        "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
        "unsized", "virtual", "yield",
        // Keywords in specific contexts (we escape these to be safe)
        "async", "await", "dyn", "try", // Edition-specific keywords
        "union",
    ];

    KEYWORDS.contains(&identifier)
}

/// Convert a YANG identifier to a safe Rust field name.
///
/// This combines snake_case conversion with keyword escaping.
///
/// # Examples
///
/// ```
/// # use rustconf::generator::naming::to_field_name;
/// assert_eq!(to_field_name("interface-type"), "interface_type");
/// assert_eq!(to_field_name("type"), "type_");
/// assert_eq!(to_field_name("match-rule"), "match_rule");
/// ```
pub fn to_field_name(identifier: &str) -> String {
    let snake = to_snake_case(identifier);
    escape_keyword(&snake)
}

/// Convert a YANG identifier to a safe Rust type name.
///
/// This combines PascalCase conversion with keyword escaping (though keywords
/// are less likely to conflict with type names).
///
/// # Examples
///
/// ```
/// # use rustconf::generator::naming::to_type_name;
/// assert_eq!(to_type_name("interface-config"), "InterfaceConfig");
/// assert_eq!(to_type_name("ip-address"), "IpAddress");
/// ```
pub fn to_type_name(identifier: &str) -> String {
    let pascal = to_pascal_case(identifier);
    escape_keyword(&pascal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case_basic() {
        assert_eq!(to_snake_case("interface-name"), "interface_name");
        assert_eq!(to_snake_case("ip-address"), "ip_address");
        assert_eq!(to_snake_case("mtu-size"), "mtu_size");
    }

    #[test]
    fn test_to_snake_case_already_snake() {
        assert_eq!(to_snake_case("already_snake"), "already_snake");
        assert_eq!(to_snake_case("some_field"), "some_field");
    }

    #[test]
    fn test_to_snake_case_pascal_case() {
        assert_eq!(to_snake_case("InterfaceName"), "interface_name");
        assert_eq!(to_snake_case("IPAddress"), "ip_address");
        assert_eq!(to_snake_case("HTTPSConnection"), "https_connection");
    }

    #[test]
    fn test_to_snake_case_mixed() {
        assert_eq!(to_snake_case("Interface-Name"), "interface_name");
        assert_eq!(to_snake_case("IP-Address"), "ip_address");
    }

    #[test]
    fn test_to_snake_case_empty() {
        assert_eq!(to_snake_case(""), "");
    }

    #[test]
    fn test_to_snake_case_single_char() {
        assert_eq!(to_snake_case("a"), "a");
        assert_eq!(to_snake_case("A"), "a");
    }

    #[test]
    fn test_to_pascal_case_basic() {
        assert_eq!(to_pascal_case("interface-config"), "InterfaceConfig");
        assert_eq!(to_pascal_case("ip-address"), "IpAddress");
        assert_eq!(to_pascal_case("mtu-size"), "MtuSize");
    }

    #[test]
    fn test_to_pascal_case_snake_case() {
        assert_eq!(to_pascal_case("interface_config"), "InterfaceConfig");
        assert_eq!(to_pascal_case("ip_address"), "IpAddress");
    }

    #[test]
    fn test_to_pascal_case_already_pascal() {
        assert_eq!(to_pascal_case("InterfaceConfig"), "InterfaceConfig");
        assert_eq!(to_pascal_case("IpAddress"), "IpAddress");
    }

    #[test]
    fn test_to_pascal_case_empty() {
        assert_eq!(to_pascal_case(""), "");
    }

    #[test]
    fn test_to_pascal_case_single_char() {
        assert_eq!(to_pascal_case("a"), "A");
        assert_eq!(to_pascal_case("A"), "A");
    }

    #[test]
    fn test_escape_keyword_keywords() {
        assert_eq!(escape_keyword("type"), "type_");
        assert_eq!(escape_keyword("match"), "match_");
        assert_eq!(escape_keyword("self"), "self_");
        assert_eq!(escape_keyword("Self"), "Self_");
        assert_eq!(escape_keyword("impl"), "impl_");
        assert_eq!(escape_keyword("trait"), "trait_");
        assert_eq!(escape_keyword("async"), "async_");
        assert_eq!(escape_keyword("await"), "await_");
    }

    #[test]
    fn test_escape_keyword_non_keywords() {
        assert_eq!(escape_keyword("interface"), "interface");
        assert_eq!(escape_keyword("config"), "config");
        assert_eq!(escape_keyword("name"), "name");
    }

    #[test]
    fn test_to_field_name() {
        assert_eq!(to_field_name("interface-type"), "interface_type");
        assert_eq!(to_field_name("type"), "type_");
        assert_eq!(to_field_name("match-rule"), "match_rule");
        assert_eq!(to_field_name("IP-Address"), "ip_address");
    }

    #[test]
    fn test_to_type_name() {
        assert_eq!(to_type_name("interface-config"), "InterfaceConfig");
        assert_eq!(to_type_name("ip-address"), "IpAddress");
        assert_eq!(to_type_name("type-def"), "TypeDef");
    }

    #[test]
    fn test_is_rust_keyword() {
        // Strict keywords
        assert!(is_rust_keyword("type"));
        assert!(is_rust_keyword("match"));
        assert!(is_rust_keyword("impl"));
        assert!(is_rust_keyword("trait"));
        assert!(is_rust_keyword("self"));
        assert!(is_rust_keyword("Self"));

        // Reserved keywords
        assert!(is_rust_keyword("async"));
        assert!(is_rust_keyword("await"));
        assert!(is_rust_keyword("abstract"));

        // Non-keywords
        assert!(!is_rust_keyword("interface"));
        assert!(!is_rust_keyword("config"));
        assert!(!is_rust_keyword("name"));
    }

    #[test]
    fn test_to_snake_case_with_numbers() {
        assert_eq!(to_snake_case("interface1"), "interface1");
        assert_eq!(to_snake_case("ipv4-address"), "ipv4_address");
        assert_eq!(to_snake_case("ipv6-address"), "ipv6_address");
        assert_eq!(to_snake_case("port8080"), "port8080");
        assert_eq!(to_snake_case("http2-connection"), "http2_connection");
        assert_eq!(to_snake_case("IPv4Address"), "i_pv4_address");
        assert_eq!(to_snake_case("HTTP2Connection"), "http2_connection");
    }

    #[test]
    fn test_to_pascal_case_with_numbers() {
        assert_eq!(to_pascal_case("interface1"), "Interface1");
        assert_eq!(to_pascal_case("ipv4-address"), "Ipv4Address");
        assert_eq!(to_pascal_case("ipv6-address"), "Ipv6Address");
        assert_eq!(to_pascal_case("http2-connection"), "Http2Connection");
        assert_eq!(to_pascal_case("port-8080"), "Port8080");
    }

    #[test]
    fn test_to_snake_case_with_special_chars() {
        // Dots as delimiters
        assert_eq!(to_snake_case("interface.name"), "interface_name");
        assert_eq!(to_snake_case("config.item"), "config_item");

        // Spaces as delimiters
        assert_eq!(to_snake_case("interface name"), "interface_name");

        // Multiple consecutive delimiters
        assert_eq!(to_snake_case("interface--name"), "interface_name");
        assert_eq!(to_snake_case("interface__name"), "interface_name");
        assert_eq!(to_snake_case("interface-.name"), "interface_name");

        // Leading delimiters are stripped
        assert_eq!(to_snake_case("-interface"), "interface");
        assert_eq!(to_snake_case("_interface"), "interface");

        // Trailing delimiters result in trailing underscore
        assert_eq!(to_snake_case("interface-"), "interface_");
        assert_eq!(to_snake_case("interface_"), "interface_");
    }

    #[test]
    fn test_to_pascal_case_with_special_chars() {
        // Dots as delimiters
        assert_eq!(to_pascal_case("interface.name"), "InterfaceName");
        assert_eq!(to_pascal_case("config.item"), "ConfigItem");

        // Spaces as delimiters
        assert_eq!(to_pascal_case("interface name"), "InterfaceName");

        // Multiple consecutive delimiters
        assert_eq!(to_pascal_case("interface--name"), "InterfaceName");
        assert_eq!(to_pascal_case("interface__name"), "InterfaceName");

        // Leading/trailing delimiters
        assert_eq!(to_pascal_case("-interface"), "Interface");
        assert_eq!(to_pascal_case("interface-"), "Interface");
    }

    #[test]
    fn test_escape_keyword_all_strict_keywords() {
        // Test a comprehensive set of strict keywords
        let keywords = vec![
            "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
            "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
            "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
            "unsafe", "use", "where", "while",
        ];

        for keyword in keywords {
            assert_eq!(escape_keyword(keyword), format!("{}_", keyword));
        }
    }

    #[test]
    fn test_escape_keyword_reserved_keywords() {
        // Test reserved keywords
        let reserved = vec![
            "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
            "unsized", "virtual", "yield", "async", "await", "dyn", "try", "union",
        ];

        for keyword in reserved {
            assert_eq!(escape_keyword(keyword), format!("{}_", keyword));
        }
    }

    #[test]
    fn test_to_field_name_with_keywords() {
        // Test field names that become keywords after conversion
        assert_eq!(to_field_name("Type"), "type_");
        assert_eq!(to_field_name("Match"), "match_");
        assert_eq!(to_field_name("IMPL"), "impl_");
    }

    #[test]
    fn test_to_snake_case_consecutive_uppercase() {
        // Test handling of consecutive uppercase letters
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("XMLParser"), "xml_parser");
        assert_eq!(to_snake_case("URLPath"), "url_path");
        assert_eq!(to_snake_case("IOError"), "io_error");
        assert_eq!(to_snake_case("HTTPSConnection"), "https_connection");
    }

    #[test]
    fn test_to_snake_case_mixed_delimiters() {
        // Test mixed delimiter types
        assert_eq!(
            to_snake_case("interface-name_config"),
            "interface_name_config"
        );
        assert_eq!(to_snake_case("http-server.port"), "http_server_port");
        assert_eq!(to_snake_case("IP-Address_v4"), "ip_address_v4");
    }

    #[test]
    fn test_to_pascal_case_mixed_delimiters() {
        // Test mixed delimiter types
        assert_eq!(
            to_pascal_case("interface-name_config"),
            "InterfaceNameConfig"
        );
        assert_eq!(to_pascal_case("http-server.port"), "HttpServerPort");
        assert_eq!(to_pascal_case("ip-address_v4"), "IpAddressV4");
    }
}
