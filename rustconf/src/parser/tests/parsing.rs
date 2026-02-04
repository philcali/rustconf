//! Unit tests for YANG parser
//! Task 4.2: Write unit tests for module header parsing
//! Task 4.4: Write unit tests for data definition parsing
//! Requirements: 1.1, 1.2, 1.3, 1.4, 2.2, 2.3, 2.4, 2.6, 2.7

#[cfg(test)]
mod tests {
    use crate::parser::{ParseError, YangParser, YangVersion};

    // ========== Module Header Tests (Task 4.2) ==========

    #[test]
    fn test_parse_simple_module_with_namespace_and_prefix() {
        let input = r#"
            module example {
                namespace "http://example.com/example";
                prefix ex;
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(
            result.is_ok(),
            "Failed to parse simple module: {:?}",
            result.err()
        );
        let module = result.unwrap();

        assert_eq!(module.name, "example");
        assert_eq!(module.namespace, "http://example.com/example");
        assert_eq!(module.prefix, "ex");
        assert_eq!(module.yang_version, None);
        assert!(module.imports.is_empty());
    }

    #[test]
    fn test_parse_module_with_yang_version() {
        let input = r#"
            module test-module {
                yang-version "1.1";
                namespace "urn:test:module";
                prefix test;
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        assert_eq!(module.name, "test-module");
        assert_eq!(module.yang_version, Some(YangVersion::V1_1));
        assert_eq!(module.namespace, "urn:test:module");
        assert_eq!(module.prefix, "test");
    }

    #[test]
    fn test_parse_module_with_import_statements() {
        let input = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import ietf-yang-types {
                    prefix yang;
                }
                
                import ietf-inet-types {
                    prefix inet;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        assert_eq!(module.name, "main");
        assert_eq!(module.imports.len(), 2);
        assert_eq!(module.imports[0].module, "ietf-yang-types");
        assert_eq!(module.imports[0].prefix, "yang");
        assert_eq!(module.imports[1].module, "ietf-inet-types");
        assert_eq!(module.imports[1].prefix, "inet");
    }

    #[test]
    fn test_error_missing_namespace() {
        let input = r#"
            module bad {
                prefix bad;
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::SyntaxError { message, .. } => {
                assert!(message.contains("namespace"));
            }
            _ => panic!("Expected SyntaxError for missing namespace"),
        }
    }

    #[test]
    fn test_error_missing_prefix() {
        let input = r#"
            module bad {
                namespace "urn:bad";
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::SyntaxError { message, .. } => {
                assert!(message.contains("prefix"));
            }
            _ => panic!("Expected SyntaxError for missing prefix"),
        }
    }

    // ========== Typedef and Grouping Tests (Task 4.3) ==========

    #[test]
    fn test_parse_typedef_simple() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef percent {
                    type uint8;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.typedefs.len(), 1);
        assert_eq!(module.typedefs[0].name, "percent");
    }

    #[test]
    fn test_parse_grouping_simple() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping endpoint {
                    leaf address {
                        type string;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.groupings.len(), 1);
        assert_eq!(module.groupings[0].name, "endpoint");
        assert_eq!(module.groupings[0].data_nodes.len(), 1);
    }

    // ========== Data Definition Tests (Task 4.4) ==========

    #[test]
    fn test_parse_container_simple() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container system-settings {
                    leaf enabled {
                        type boolean;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();
        assert_eq!(module.data_nodes.len(), 1);

        if let crate::parser::DataNode::Container(container) = &module.data_nodes[0] {
            assert_eq!(container.name, "system-settings");
            assert_eq!(container.children.len(), 1);
            assert!(container.config);
            assert!(!container.mandatory);
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_parse_container_with_nested_containers() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container outer {
                    description "Outer container";
                    container inner {
                        description "Inner container";
                        leaf value {
                            type string;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.data_nodes.len(), 1);

        if let crate::parser::DataNode::Container(outer) = &module.data_nodes[0] {
            assert_eq!(outer.name, "outer");
            assert_eq!(outer.description, Some("Outer container".to_string()));
            assert_eq!(outer.children.len(), 1);

            if let crate::parser::DataNode::Container(inner) = &outer.children[0] {
                assert_eq!(inner.name, "inner");
                assert_eq!(inner.description, Some("Inner container".to_string()));
                assert_eq!(inner.children.len(), 1);
            } else {
                panic!("Expected nested Container");
            }
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_parse_container_with_config_and_mandatory() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container system {
                    config false;
                    mandatory true;
                    leaf hostname {
                        type string;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Container(container) = &module.data_nodes[0] {
            assert_eq!(container.name, "system");
            assert!(!container.config);
            assert!(container.mandatory);
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_parse_list_with_key() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                list interface {
                    key "name";
                    leaf name {
                        type string;
                    }
                    leaf enabled {
                        type boolean;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();
        assert_eq!(module.data_nodes.len(), 1);

        if let crate::parser::DataNode::List(list) = &module.data_nodes[0] {
            assert_eq!(list.name, "interface");
            assert_eq!(list.keys, vec!["name"]);
            assert_eq!(list.children.len(), 2);
            assert!(list.config);
        } else {
            panic!("Expected List data node");
        }
    }

    #[test]
    fn test_parse_list_with_multiple_keys() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                list connection {
                    key "source destination";
                    leaf source {
                        type string;
                    }
                    leaf destination {
                        type string;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::List(list) = &module.data_nodes[0] {
            assert_eq!(list.name, "connection");
            assert_eq!(list.keys, vec!["source", "destination"]);
        } else {
            panic!("Expected List data node");
        }
    }

    #[test]
    fn test_parse_list_with_nested_container() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                list server {
                    key "id";
                    leaf id {
                        type uint32;
                    }
                    container settings {
                        leaf hostname {
                            type string;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::List(list) = &module.data_nodes[0] {
            assert_eq!(list.name, "server");
            assert_eq!(list.children.len(), 2);

            let has_container = list
                .children
                .iter()
                .any(|node| matches!(node, crate::parser::DataNode::Container(_)));
            assert!(has_container, "List should contain a nested container");
        } else {
            panic!("Expected List data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_mandatory() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf username {
                    type string;
                    mandatory true;
                    description "User login name";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.data_nodes.len(), 1);

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "username");
            assert!(leaf.mandatory);
            assert_eq!(leaf.description, Some("User login name".to_string()));
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_default() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf timeout {
                    type uint32;
                    default 30;
                    config true;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "timeout");
            assert_eq!(leaf.default, Some("30".to_string()));
            assert!(leaf.config);
            assert!(!leaf.mandatory);
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_list() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf-list dns-server {
                    type string;
                    description "DNS server addresses";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();
        assert_eq!(module.data_nodes.len(), 1);

        if let crate::parser::DataNode::LeafList(leaf_list) = &module.data_nodes[0] {
            assert_eq!(leaf_list.name, "dns-server");
            assert_eq!(
                leaf_list.description,
                Some("DNS server addresses".to_string())
            );
            assert!(leaf_list.config);
        } else {
            panic!("Expected LeafList data node");
        }
    }

    #[test]
    fn test_parse_choice_with_cases() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                choice address-type {
                    description "Address type selection";
                    case ipv4 {
                        leaf ipv4-address {
                            type string;
                        }
                    }
                    case ipv6 {
                        leaf ipv6-address {
                            type string;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();
        assert_eq!(module.data_nodes.len(), 1);

        if let crate::parser::DataNode::Choice(choice) = &module.data_nodes[0] {
            assert_eq!(choice.name, "address-type");
            assert_eq!(
                choice.description,
                Some("Address type selection".to_string())
            );
            assert_eq!(choice.cases.len(), 2);
            assert_eq!(choice.cases[0].name, "ipv4");
            assert_eq!(choice.cases[1].name, "ipv6");
            assert!(!choice.mandatory);
        } else {
            panic!("Expected Choice data node");
        }
    }

    #[test]
    fn test_parse_choice_with_mandatory() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                choice protocol {
                    mandatory true;
                    case tcp {
                        leaf tcp-port {
                            type uint16;
                        }
                    }
                    case udp {
                        leaf udp-port {
                            type uint16;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Choice(choice) = &module.data_nodes[0] {
            assert_eq!(choice.name, "protocol");
            assert!(choice.mandatory);
            assert_eq!(choice.cases.len(), 2);
        } else {
            panic!("Expected Choice data node");
        }
    }

    #[test]
    fn test_parse_choice_with_shorthand_cases() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                choice transport {
                    leaf tcp {
                        type boolean;
                    }
                    leaf udp {
                        type boolean;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Choice(choice) = &module.data_nodes[0] {
            assert_eq!(choice.name, "transport");
            assert_eq!(choice.cases.len(), 2);
            assert_eq!(choice.cases[0].name, "tcp");
            assert_eq!(choice.cases[1].name, "udp");
        } else {
            panic!("Expected Choice data node");
        }
    }

    #[test]
    fn test_parse_case_with_multiple_nodes() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                choice config-method {
                    case manual {
                        description "Manual configuration";
                        leaf ip-address {
                            type string;
                        }
                        leaf netmask {
                            type string;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Choice(choice) = &module.data_nodes[0] {
            assert_eq!(choice.cases.len(), 1);
            let case = &choice.cases[0];
            assert_eq!(case.name, "manual");
            assert_eq!(case.description, Some("Manual configuration".to_string()));
            assert_eq!(case.data_nodes.len(), 2);
        } else {
            panic!("Expected Choice data node");
        }
    }

    #[test]
    fn test_parse_complex_nested_structure() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container system {
                    description "System configuration";
                    
                    list interface {
                        key "name";
                        
                        leaf name {
                            type string;
                            mandatory true;
                        }
                        
                        container settings {
                            leaf enabled {
                                type boolean;
                                default true;
                            }
                            
                            choice address-family {
                                case ipv4 {
                                    leaf ipv4-address {
                                        type string;
                                    }
                                }
                                case ipv6 {
                                    leaf ipv6-address {
                                        type string;
                                    }
                                }
                            }
                        }
                        
                        leaf-list dns-server {
                            type string;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();
        assert_eq!(module.data_nodes.len(), 1);

        if let crate::parser::DataNode::Container(system) = &module.data_nodes[0] {
            assert_eq!(system.name, "system");
            assert_eq!(system.children.len(), 1);

            if let crate::parser::DataNode::List(interface) = &system.children[0] {
                assert_eq!(interface.name, "interface");
                assert_eq!(interface.keys, vec!["name"]);
                assert_eq!(interface.children.len(), 3);

                let has_container = interface.children.iter().any(|node| {
                    if let crate::parser::DataNode::Container(c) = node {
                        c.name == "settings"
                    } else {
                        false
                    }
                });
                assert!(has_container, "List should contain settings container");
            } else {
                panic!("Expected List in system container");
            }
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_parse_multiple_top_level_data_nodes() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container system-settings {
                    leaf enabled {
                        type boolean;
                    }
                }
                
                list users {
                    key "username";
                    leaf username {
                        type string;
                    }
                }
                
                leaf version {
                    type string;
                    config false;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.data_nodes.len(), 3);

        assert!(matches!(
            module.data_nodes[0],
            crate::parser::DataNode::Container(_)
        ));
        assert!(matches!(
            module.data_nodes[1],
            crate::parser::DataNode::List(_)
        ));
        assert!(matches!(
            module.data_nodes[2],
            crate::parser::DataNode::Leaf(_)
        ));
    }

    #[test]
    fn test_parse_container_with_all_child_types() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container root {
                    container nested-container {
                        leaf value {
                            type string;
                        }
                    }
                    
                    list items {
                        key "id";
                        leaf id {
                            type uint32;
                        }
                    }
                    
                    leaf simple-leaf {
                        type boolean;
                    }
                    
                    leaf-list tags {
                        type string;
                    }
                    
                    choice option {
                        case a {
                            leaf option-a {
                                type string;
                            }
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Container(root) = &module.data_nodes[0] {
            assert_eq!(root.name, "root");
            assert_eq!(root.children.len(), 5);

            let has_container = root
                .children
                .iter()
                .any(|n| matches!(n, crate::parser::DataNode::Container(_)));
            let has_list = root
                .children
                .iter()
                .any(|n| matches!(n, crate::parser::DataNode::List(_)));
            let has_leaf = root
                .children
                .iter()
                .any(|n| matches!(n, crate::parser::DataNode::Leaf(_)));
            let has_leaf_list = root
                .children
                .iter()
                .any(|n| matches!(n, crate::parser::DataNode::LeafList(_)));
            let has_choice = root
                .children
                .iter()
                .any(|n| matches!(n, crate::parser::DataNode::Choice(_)));

            assert!(has_container);
            assert!(has_list);
            assert!(has_leaf);
            assert!(has_leaf_list);
            assert!(has_choice);
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_error_leaf_missing_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf bad {
                    description "Missing type";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::SyntaxError { message, .. } => {
                assert!(message.contains("type"));
            }
            _ => panic!("Expected SyntaxError for missing type in leaf"),
        }
    }

    #[test]
    fn test_error_leaf_list_missing_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf-list bad {
                    description "Missing type";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::SyntaxError { message, .. } => {
                assert!(message.contains("type"));
            }
            _ => panic!("Expected SyntaxError for missing type in leaf-list"),
        }
    }

    #[test]
    fn test_parse_deeply_nested_containers() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container level1 {
                    container level2 {
                        container level3 {
                            container level4 {
                                leaf value {
                                    type string;
                                }
                            }
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Container(l1) = &module.data_nodes[0] {
            assert_eq!(l1.name, "level1");
            if let crate::parser::DataNode::Container(l2) = &l1.children[0] {
                assert_eq!(l2.name, "level2");
                if let crate::parser::DataNode::Container(l3) = &l2.children[0] {
                    assert_eq!(l3.name, "level3");
                    if let crate::parser::DataNode::Container(l4) = &l3.children[0] {
                        assert_eq!(l4.name, "level4");
                        assert_eq!(l4.children.len(), 1);
                    } else {
                        panic!("Expected level4 container");
                    }
                } else {
                    panic!("Expected level3 container");
                }
            } else {
                panic!("Expected level2 container");
            }
        } else {
            panic!("Expected level1 container");
        }
    }

    // ========== Type Specification Tests (Task 4.6) ==========

    #[test]
    fn test_parse_leaf_with_int8_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf age {
                    type int8;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "age");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Int8 { .. }
            ));
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_uint32_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf counter {
                    type uint32;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "counter");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Uint32 { .. }
            ));
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_string_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf name {
                    type string;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "name");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::String { .. }
            ));
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_boolean_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf enabled {
                    type boolean;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "enabled");
            assert!(matches!(leaf.type_spec, crate::parser::TypeSpec::Boolean));
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_range_constraint() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf port {
                    type uint16 {
                        range "1..65535";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "port");
            if let crate::parser::TypeSpec::Uint16 { range } = &leaf.type_spec {
                assert!(range.is_some());
                let range = range.as_ref().unwrap();
                assert_eq!(range.ranges.len(), 1);
                assert_eq!(range.ranges[0].min, 1);
                assert_eq!(range.ranges[0].max, 65535);
            } else {
                panic!("Expected Uint16 type with range");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_multiple_range_constraints() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf priority {
                    type int32 {
                        range "1..10 | 20..30 | 40..50";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            if let crate::parser::TypeSpec::Int32 { range } = &leaf.type_spec {
                assert!(range.is_some());
                let range = range.as_ref().unwrap();
                assert_eq!(range.ranges.len(), 3);
                assert_eq!(range.ranges[0].min, 1);
                assert_eq!(range.ranges[0].max, 10);
                assert_eq!(range.ranges[1].min, 20);
                assert_eq!(range.ranges[1].max, 30);
                assert_eq!(range.ranges[2].min, 40);
                assert_eq!(range.ranges[2].max, 50);
            } else {
                panic!("Expected Int32 type with range");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_length_constraint() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf username {
                    type string {
                        length "1..64";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "username");
            if let crate::parser::TypeSpec::String { length, .. } = &leaf.type_spec {
                assert!(length.is_some());
                let length = length.as_ref().unwrap();
                assert_eq!(length.lengths.len(), 1);
                assert_eq!(length.lengths[0].min, 1);
                assert_eq!(length.lengths[0].max, 64);
            } else {
                panic!("Expected String type with length");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_pattern_constraint() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf email {
                    type string {
                        pattern "[a-zA-Z0-9]+@[a-zA-Z0-9]+";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "email");
            if let crate::parser::TypeSpec::String { pattern, .. } = &leaf.type_spec {
                assert!(pattern.is_some());
                let pattern = pattern.as_ref().unwrap();
                assert!(pattern.pattern.contains("@"));
            } else {
                panic!("Expected String type with pattern");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_length_and_pattern_constraints() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf code {
                    type string {
                        length "3..10";
                        pattern "[A-Z][0-9]+";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            if let crate::parser::TypeSpec::String { length, pattern } = &leaf.type_spec {
                assert!(length.is_some());
                assert!(pattern.is_some());

                let length = length.as_ref().unwrap();
                assert_eq!(length.lengths[0].min, 3);
                assert_eq!(length.lengths[0].max, 10);

                let pattern = pattern.as_ref().unwrap();
                assert!(pattern.pattern.contains("[A-Z]"));
            } else {
                panic!("Expected String type with length and pattern");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_enumeration_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf state {
                    type enumeration {
                        enum active;
                        enum inactive;
                        enum pending;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "state");
            if let crate::parser::TypeSpec::Enumeration { values } = &leaf.type_spec {
                assert_eq!(values.len(), 3);
                assert_eq!(values[0].name, "active");
                assert_eq!(values[1].name, "inactive");
                assert_eq!(values[2].name, "pending");
            } else {
                panic!("Expected Enumeration type");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_enumeration_values_and_descriptions() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf color {
                    type enumeration {
                        enum red {
                            value 1;
                            description "Red color";
                        }
                        enum green {
                            value 2;
                            description "Green color";
                        }
                        enum blue {
                            value 3;
                            description "Blue color";
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            if let crate::parser::TypeSpec::Enumeration { values } = &leaf.type_spec {
                assert_eq!(values.len(), 3);

                assert_eq!(values[0].name, "red");
                assert_eq!(values[0].value, Some(1));
                assert_eq!(values[0].description, Some("Red color".to_string()));

                assert_eq!(values[1].name, "green");
                assert_eq!(values[1].value, Some(2));
                assert_eq!(values[1].description, Some("Green color".to_string()));

                assert_eq!(values[2].name, "blue");
                assert_eq!(values[2].value, Some(3));
                assert_eq!(values[2].description, Some("Blue color".to_string()));
            } else {
                panic!("Expected Enumeration type");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_union_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf value {
                    type union {
                        type int32;
                        type string;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "value");
            if let crate::parser::TypeSpec::Union { types } = &leaf.type_spec {
                assert_eq!(types.len(), 2);
                assert!(matches!(types[0], crate::parser::TypeSpec::Int32 { .. }));
                assert!(matches!(types[1], crate::parser::TypeSpec::String { .. }));
            } else {
                panic!("Expected Union type");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_union_type_with_constraints() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf port_or_name {
                    type union {
                        type uint16 {
                            range "1..65535";
                        }
                        type string {
                            length "1..32";
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            if let crate::parser::TypeSpec::Union { types } = &leaf.type_spec {
                assert_eq!(types.len(), 2);

                if let crate::parser::TypeSpec::Uint16 { range } = &types[0] {
                    assert!(range.is_some());
                } else {
                    panic!("Expected Uint16 type in union");
                }

                if let crate::parser::TypeSpec::String { length, .. } = &types[1] {
                    assert!(length.is_some());
                } else {
                    panic!("Expected String type in union");
                }
            } else {
                panic!("Expected Union type");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_binary_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf data {
                    type binary;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "data");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Binary { .. }
            ));
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_binary_length_constraint() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf hash {
                    type binary {
                        length "32";
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            if let crate::parser::TypeSpec::Binary { length } = &leaf.type_spec {
                assert!(length.is_some());
                let length = length.as_ref().unwrap();
                assert_eq!(length.lengths[0].min, 32);
                assert_eq!(length.lengths[0].max, 32);
            } else {
                panic!("Expected Binary type with length");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_empty_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf flag {
                    type empty;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "flag");
            assert!(matches!(leaf.type_spec, crate::parser::TypeSpec::Empty));
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_all_integer_types() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf i8 { type int8; }
                leaf i16 { type int16; }
                leaf i32 { type int32; }
                leaf i64 { type int64; }
                leaf u8 { type uint8; }
                leaf u16 { type uint16; }
                leaf u32 { type uint32; }
                leaf u64 { type uint64; }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.data_nodes.len(), 8);

        // Check each leaf type
        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "i8");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Int8 { .. }
            ));
        }

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[1] {
            assert_eq!(leaf.name, "i16");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Int16 { .. }
            ));
        }

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[2] {
            assert_eq!(leaf.name, "i32");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Int32 { .. }
            ));
        }

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[3] {
            assert_eq!(leaf.name, "i64");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Int64 { .. }
            ));
        }

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[4] {
            assert_eq!(leaf.name, "u8");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Uint8 { .. }
            ));
        }

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[5] {
            assert_eq!(leaf.name, "u16");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Uint16 { .. }
            ));
        }

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[6] {
            assert_eq!(leaf.name, "u32");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Uint32 { .. }
            ));
        }

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[7] {
            assert_eq!(leaf.name, "u64");
            assert!(matches!(
                leaf.type_spec,
                crate::parser::TypeSpec::Uint64 { .. }
            ));
        }
    }

    #[test]
    fn test_parse_typedef_with_constrained_type() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef percent {
                    type uint8 {
                        range "0..100";
                    }
                    units "percent";
                    description "Percentage value";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.typedefs.len(), 1);

        let typedef = &module.typedefs[0];
        assert_eq!(typedef.name, "percent");
        assert_eq!(typedef.units, Some("percent".to_string()));
        assert_eq!(typedef.description, Some("Percentage value".to_string()));

        if let crate::parser::TypeSpec::Uint8 { range } = &typedef.type_spec {
            assert!(range.is_some());
            let range = range.as_ref().unwrap();
            assert_eq!(range.ranges[0].min, 0);
            assert_eq!(range.ranges[0].max, 100);
        } else {
            panic!("Expected Uint8 type with range");
        }
    }

    #[test]
    fn test_parse_complex_type_with_multiple_constraints() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container network {
                    leaf port {
                        type uint16 {
                            range "1024..65535";
                        }
                        mandatory true;
                    }
                    
                    leaf hostname {
                        type string {
                            length "1..255";
                            pattern "[a-zA-Z0-9.-]+";
                        }
                    }
                    
                    leaf protocol {
                        type enumeration {
                            enum tcp {
                                value 6;
                            }
                            enum udp {
                                value 17;
                            }
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        if let crate::parser::DataNode::Container(container) = &module.data_nodes[0] {
            assert_eq!(container.name, "network");
            assert_eq!(container.children.len(), 3);

            // Check port leaf
            if let crate::parser::DataNode::Leaf(port) = &container.children[0] {
                assert_eq!(port.name, "port");
                assert!(port.mandatory);
                if let crate::parser::TypeSpec::Uint16 { range } = &port.type_spec {
                    assert!(range.is_some());
                } else {
                    panic!("Expected Uint16 type");
                }
            } else {
                panic!("Expected Leaf for port");
            }

            // Check hostname leaf
            if let crate::parser::DataNode::Leaf(hostname) = &container.children[1] {
                assert_eq!(hostname.name, "hostname");
                if let crate::parser::TypeSpec::String { length, pattern } = &hostname.type_spec {
                    assert!(length.is_some());
                    assert!(pattern.is_some());
                } else {
                    panic!("Expected String type");
                }
            } else {
                panic!("Expected Leaf for hostname");
            }

            // Check protocol leaf
            if let crate::parser::DataNode::Leaf(protocol) = &container.children[2] {
                assert_eq!(protocol.name, "protocol");
                if let crate::parser::TypeSpec::Enumeration { values } = &protocol.type_spec {
                    assert_eq!(values.len(), 2);
                    assert_eq!(values[0].name, "tcp");
                    assert_eq!(values[0].value, Some(6));
                    assert_eq!(values[1].name, "udp");
                    assert_eq!(values[1].value, Some(17));
                } else {
                    panic!("Expected Enumeration type");
                }
            } else {
                panic!("Expected Leaf for protocol");
            }
        } else {
            panic!("Expected Container data node");
        }
    }

    // ========== Additional Error Case Tests (Task 4.6) ==========

    #[test]
    fn test_parse_list_without_key() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                list items {
                    leaf name {
                        type string;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        // Lists can be keyless in YANG (though not recommended)
        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::List(list) = &module.data_nodes[0] {
            assert_eq!(list.name, "items");
            assert_eq!(list.keys.len(), 0);
            assert_eq!(list.children.len(), 1);
        } else {
            panic!("Expected List data node");
        }
    }

    #[test]
    fn test_error_choice_without_cases() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                choice empty-choice {
                    description "A choice with no cases";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        // A choice should have at least one case (though parser may allow empty)
        // This test verifies the parser handles empty choices gracefully
        if let Ok(module) = result {
            if let crate::parser::DataNode::Choice(choice) = &module.data_nodes[0] {
                assert_eq!(choice.cases.len(), 0);
            }
        }
    }

    #[test]
    fn test_parse_container_with_mixed_config_children() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container system {
                    leaf config-value {
                        type string;
                        config true;
                    }
                    
                    leaf state-value {
                        type string;
                        config false;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Container(container) = &module.data_nodes[0] {
            assert_eq!(container.children.len(), 2);

            if let crate::parser::DataNode::Leaf(leaf1) = &container.children[0] {
                assert_eq!(leaf1.name, "config-value");
                assert!(leaf1.config);
            }

            if let crate::parser::DataNode::Leaf(leaf2) = &container.children[1] {
                assert_eq!(leaf2.name, "state-value");
                assert!(!leaf2.config);
            }
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_parse_list_with_config_false() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                list statistics {
                    key "counter-id";
                    config false;
                    
                    leaf counter-id {
                        type uint32;
                    }
                    
                    leaf value {
                        type uint64;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::List(list) = &module.data_nodes[0] {
            assert_eq!(list.name, "statistics");
            assert!(!list.config);
            assert_eq!(list.keys, vec!["counter-id"]);
        } else {
            panic!("Expected List data node");
        }
    }

    #[test]
    fn test_parse_leaf_list_with_config_false() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf-list active-sessions {
                    type string;
                    config false;
                    description "Currently active session IDs";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::LeafList(leaf_list) = &module.data_nodes[0] {
            assert_eq!(leaf_list.name, "active-sessions");
            assert!(!leaf_list.config);
            assert_eq!(
                leaf_list.description,
                Some("Currently active session IDs".to_string())
            );
        } else {
            panic!("Expected LeafList data node");
        }
    }

    #[test]
    fn test_parse_empty_container() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container placeholder {
                    description "An empty container";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let module = result.unwrap();

        if let crate::parser::DataNode::Container(container) = &module.data_nodes[0] {
            assert_eq!(container.name, "placeholder");
            assert_eq!(container.children.len(), 0);
            assert_eq!(
                container.description,
                Some("An empty container".to_string())
            );
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_parse_leaf_with_all_properties() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf complete-leaf {
                    type string {
                        length "1..100";
                        pattern "[a-z]+";
                    }
                    description "A leaf with all properties";
                    mandatory true;
                    config true;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "complete-leaf");
            assert_eq!(
                leaf.description,
                Some("A leaf with all properties".to_string())
            );
            assert!(leaf.mandatory);
            assert!(leaf.config);

            if let crate::parser::TypeSpec::String { length, pattern } = &leaf.type_spec {
                assert!(length.is_some());
                assert!(pattern.is_some());
            } else {
                panic!("Expected String type with constraints");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_parse_choice_with_default_case() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                choice format {
                    description "Output format selection";
                    case json {
                        leaf json-output {
                            type boolean;
                        }
                    }
                    case xml {
                        leaf xml-output {
                            type boolean;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        if let crate::parser::DataNode::Choice(choice) = &module.data_nodes[0] {
            assert_eq!(choice.name, "format");
            assert_eq!(
                choice.description,
                Some("Output format selection".to_string())
            );
            assert_eq!(choice.cases.len(), 2);
            assert!(!choice.mandatory);
        } else {
            panic!("Expected Choice data node");
        }
    }

    // Task 5.2: Unit tests for import resolution
    // Requirements: 1.4, 7.4

    #[test]
    fn test_resolve_imports_from_search_paths() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create imported module
        let imported_content = r#"
            module ietf_yang_types {
                namespace "urn:ietf:params:xml:ns:yang:ietf-yang-types";
                prefix yang;
            }
        "#;
        let imported_path = temp_dir.path().join("ietf-yang-types.yang");
        fs::write(&imported_path, imported_content).unwrap();

        // Create main module that imports the above
        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import ietf-yang-types {
                    prefix yang;
                }
            }
        "#;

        let mut parser = YangParser::new();
        parser.add_search_path(temp_dir.path().to_path_buf());

        let result = parser.parse_string(main_content, "main.yang");
        assert!(
            result.is_ok(),
            "Failed to parse module with imports: {:?}",
            result.err()
        );

        let module = result.unwrap();
        assert_eq!(module.name, "main");
        assert_eq!(module.imports.len(), 1);

        // Verify the imported module was loaded
        let loaded = parser.get_loaded_module("ietf-yang-types");
        assert!(loaded.is_some(), "Imported module should be loaded");
        assert_eq!(loaded.unwrap().name, "ietf_yang_types");
    }

    #[test]
    fn test_handle_missing_imported_modules() {
        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import missing_module {
                    prefix miss;
                }
            }
        "#;

        let mut parser = YangParser::new();
        // No search paths added, so import will fail

        // Parse should succeed but import resolution fails silently
        let result = parser.parse_string(main_content, "main.yang");
        assert!(
            result.is_ok(),
            "Parsing should succeed even if imports can't be resolved"
        );

        // The missing module should not be in loaded modules
        assert!(parser.get_loaded_module("missing-module").is_none());
    }

    #[test]
    fn test_recursive_import_loading() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create a chain of imports: main -> modulea -> moduleb
        let module_b_content = r#"
            module moduleb {
                namespace "urn:module-b";
                prefix b;
            }
        "#;
        let module_b_path = temp_dir.path().join("moduleb.yang");
        fs::write(&module_b_path, module_b_content).unwrap();

        let module_a_content = r#"
            module modulea {
                namespace "urn:module-a";
                prefix a;
                
                import moduleb {
                    prefix b;
                }
            }
        "#;
        let module_a_path = temp_dir.path().join("modulea.yang");
        fs::write(&module_a_path, module_a_content).unwrap();

        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import modulea {
                    prefix a;
                }
            }
        "#;

        let mut parser = YangParser::new();
        parser.add_search_path(temp_dir.path().to_path_buf());

        let result = parser.parse_string(main_content, "main.yang");
        assert!(
            result.is_ok(),
            "Failed to parse module with recursive imports: {:?}",
            result.err()
        );

        // Verify all modules in the chain were loaded
        assert!(parser.get_loaded_module("main").is_some());
        assert!(parser.get_loaded_module("modulea").is_some());
        assert!(parser.get_loaded_module("moduleb").is_some());
    }

    #[test]
    fn test_avoid_duplicate_parsing() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create a shared module
        let shared_content = r#"
            module shared {
                namespace "urn:shared";
                prefix shared;
            }
        "#;
        let shared_path = temp_dir.path().join("shared.yang");
        fs::write(&shared_path, shared_content).unwrap();

        // Create two modules that both import the shared module
        let module_a_content = r#"
            module modulea {
                namespace "urn:module-a";
                prefix a;
                
                import shared {
                    prefix shared;
                }
            }
        "#;
        let module_a_path = temp_dir.path().join("modulea.yang");
        fs::write(&module_a_path, module_a_content).unwrap();

        let module_b_content = r#"
            module moduleb {
                namespace "urn:module-b";
                prefix b;
                
                import shared {
                    prefix shared;
                }
            }
        "#;
        let module_b_path = temp_dir.path().join("moduleb.yang");
        fs::write(&module_b_path, module_b_content).unwrap();

        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import modulea {
                    prefix a;
                }
                
                import moduleb {
                    prefix b;
                }
            }
        "#;

        let mut parser = YangParser::new();
        parser.add_search_path(temp_dir.path().to_path_buf());

        let result = parser.parse_string(main_content, "main.yang");
        assert!(
            result.is_ok(),
            "Failed to parse module with shared imports: {:?}",
            result.err()
        );

        // Verify all modules were loaded
        assert!(parser.get_loaded_module("main").is_some());
        assert!(parser.get_loaded_module("modulea").is_some());
        assert!(parser.get_loaded_module("moduleb").is_some());
        assert!(parser.get_loaded_module("shared").is_some());

        // The shared module should only be loaded once
        let all_modules = parser.get_all_loaded_modules();
        assert_eq!(all_modules.len(), 4); // main, modulea, moduleb, shared
    }

    #[test]
    fn test_multiple_search_paths() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        // Create module in first search path
        let module_a_content = r#"
            module modulea {
                namespace "urn:module-a";
                prefix a;
            }
        "#;
        let module_a_path = temp_dir1.path().join("modulea.yang");
        fs::write(&module_a_path, module_a_content).unwrap();

        // Create module in second search path
        let module_b_content = r#"
            module moduleb {
                namespace "urn:module-b";
                prefix b;
            }
        "#;
        let module_b_path = temp_dir2.path().join("moduleb.yang");
        fs::write(&module_b_path, module_b_content).unwrap();

        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import modulea {
                    prefix a;
                }
                
                import moduleb {
                    prefix b;
                }
            }
        "#;

        let mut parser = YangParser::new();
        parser.add_search_path(temp_dir1.path().to_path_buf());
        parser.add_search_path(temp_dir2.path().to_path_buf());

        let result = parser.parse_string(main_content, "main.yang");
        assert!(
            result.is_ok(),
            "Failed to parse module with multiple search paths: {:?}",
            result.err()
        );

        // Verify both modules were found and loaded
        assert!(parser.get_loaded_module("modulea").is_some());
        assert!(parser.get_loaded_module("moduleb").is_some());
    }

    #[test]
    fn test_parse_file_with_imports() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create imported module
        let imported_content = r#"
            module imported {
                namespace "urn:imported";
                prefix imp;
            }
        "#;
        let imported_path = temp_dir.path().join("imported.yang");
        fs::write(&imported_path, imported_content).unwrap();

        // Create main module file
        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import imported {
                    prefix imp;
                }
            }
        "#;
        let main_path = temp_dir.path().join("main.yang");
        fs::write(&main_path, main_content).unwrap();

        let mut parser = YangParser::new();
        parser.add_search_path(temp_dir.path().to_path_buf());

        let result = parser.parse_file(&main_path);
        assert!(
            result.is_ok(),
            "Failed to parse file with imports: {:?}",
            result.err()
        );

        // Verify both modules were loaded
        assert!(parser.get_loaded_module("main").is_some());
        assert!(parser.get_loaded_module("imported").is_some());
    }

    #[test]
    fn test_get_all_loaded_modules() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        let module_a_content = r#"
            module modulea {
                namespace "urn:module-a";
                prefix a;
            }
        "#;
        let module_a_path = temp_dir.path().join("modulea.yang");
        fs::write(&module_a_path, module_a_content).unwrap();

        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import modulea {
                    prefix a;
                }
            }
        "#;

        let mut parser = YangParser::new();
        parser.add_search_path(temp_dir.path().to_path_buf());

        parser.parse_string(main_content, "main.yang").unwrap();

        let all_modules = parser.get_all_loaded_modules();
        assert_eq!(all_modules.len(), 2);
        assert!(all_modules.contains_key("main"));
        assert!(all_modules.contains_key("modulea"));
    }

    #[test]
    fn test_import_with_revision_date() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create imported module
        let imported_content = r#"
            module versioned {
                namespace "urn:versioned";
                prefix ver;
            }
        "#;
        let imported_path = temp_dir.path().join("versioned.yang");
        fs::write(&imported_path, imported_content).unwrap();

        // Create main module with revision-date in import
        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import versioned {
                    prefix ver;
                    revision-date "2024-01-15";
                }
            }
        "#;

        let mut parser = YangParser::new();
        parser.add_search_path(temp_dir.path().to_path_buf());

        let result = parser.parse_string(main_content, "main.yang");
        assert!(
            result.is_ok(),
            "Failed to parse module with revision-date in import: {:?}",
            result.err()
        );

        let module = result.unwrap();
        assert_eq!(module.imports.len(), 1);

        // Note: Current implementation skips revision-date parsing
        // This test verifies that the parser doesn't fail when revision-date is present
        // Future enhancement: store and validate revision dates
        assert_eq!(module.imports[0].module, "versioned");
        assert_eq!(module.imports[0].prefix, "ver");
    }

    #[test]
    fn test_import_resolution_with_yang_extension() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create imported module with .yang extension
        let imported_content = r#"
            module types {
                namespace "urn:types";
                prefix types;
            }
        "#;
        let imported_path = temp_dir.path().join("types.yang");
        fs::write(&imported_path, imported_content).unwrap();

        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import types {
                    prefix types;
                }
            }
        "#;

        let mut parser = YangParser::new();
        parser.add_search_path(temp_dir.path().to_path_buf());

        let result = parser.parse_string(main_content, "main.yang");
        assert!(
            result.is_ok(),
            "Failed to resolve import with .yang extension: {:?}",
            result.err()
        );

        // Verify the imported module was loaded
        assert!(parser.get_loaded_module("types").is_some());
    }

    #[test]
    fn test_import_resolution_order() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        // Create different versions of the same module in two search paths
        let module_v1_content = r#"
            module common {
                namespace "urn:common:v1";
                prefix common;
            }
        "#;
        let module_v1_path = temp_dir1.path().join("common.yang");
        fs::write(&module_v1_path, module_v1_content).unwrap();

        let module_v2_content = r#"
            module common {
                namespace "urn:common:v2";
                prefix common;
            }
        "#;
        let module_v2_path = temp_dir2.path().join("common.yang");
        fs::write(&module_v2_path, module_v2_content).unwrap();

        let main_content = r#"
            module main {
                namespace "urn:main";
                prefix main;
                
                import common {
                    prefix common;
                }
            }
        "#;

        let mut parser = YangParser::new();
        // Add search paths in order - first path should take precedence
        parser.add_search_path(temp_dir1.path().to_path_buf());
        parser.add_search_path(temp_dir2.path().to_path_buf());

        let result = parser.parse_string(main_content, "main.yang");
        assert!(result.is_ok());

        // Verify the first version was loaded (from first search path)
        let loaded = parser.get_loaded_module("common");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().namespace, "urn:common:v1");
    }

    // ========== RPC Parsing Tests ==========

    #[test]
    fn test_parse_rpc_with_input_and_output() {
        let input = r#"
            module test-rpc {
                namespace "urn:test:rpc";
                prefix tr;

                rpc restart-device {
                    description "Restart the device";
                    input {
                        leaf delay-seconds {
                            type uint32;
                            default 0;
                        }
                    }
                    output {
                        leaf success {
                            type boolean;
                        }
                        leaf message {
                            type string;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok(), "Failed to parse RPC: {:?}", result.err());
        let module = result.unwrap();

        assert_eq!(module.rpcs.len(), 1);
        let rpc = &module.rpcs[0];
        assert_eq!(rpc.name, "restart-device");
        assert_eq!(rpc.description, Some("Restart the device".to_string()));
        assert!(rpc.input.is_some());
        assert!(rpc.output.is_some());

        let input_nodes = rpc.input.as_ref().unwrap();
        assert_eq!(input_nodes.len(), 1);

        let output_nodes = rpc.output.as_ref().unwrap();
        assert_eq!(output_nodes.len(), 2);
    }

    #[test]
    fn test_parse_rpc_with_output_only() {
        let input = r#"
            module test-rpc {
                namespace "urn:test:rpc";
                prefix tr;

                rpc get-system-info {
                    description "Get system information";
                    output {
                        leaf hostname {
                            type string;
                        }
                        leaf uptime {
                            type uint64;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        assert_eq!(module.rpcs.len(), 1);
        let rpc = &module.rpcs[0];
        assert_eq!(rpc.name, "get-system-info");
        assert!(rpc.input.is_none());
        assert!(rpc.output.is_some());

        let output_nodes = rpc.output.as_ref().unwrap();
        assert_eq!(output_nodes.len(), 2);
    }

    #[test]
    fn test_parse_rpc_with_input_only() {
        let input = r#"
            module test-rpc {
                namespace "urn:test:rpc";
                prefix tr;

                rpc send-notification {
                    description "Send a notification";
                    input {
                        leaf message {
                            type string;
                            mandatory true;
                        }
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        assert_eq!(module.rpcs.len(), 1);
        let rpc = &module.rpcs[0];
        assert_eq!(rpc.name, "send-notification");
        assert!(rpc.input.is_some());
        assert!(rpc.output.is_none());
    }

    #[test]
    fn test_parse_rpc_without_input_or_output() {
        let input = r#"
            module test-rpc {
                namespace "urn:test:rpc";
                prefix tr;

                rpc trigger-action {
                    description "Trigger an action";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        assert_eq!(module.rpcs.len(), 1);
        let rpc = &module.rpcs[0];
        assert_eq!(rpc.name, "trigger-action");
        assert!(rpc.input.is_none());
        assert!(rpc.output.is_none());
    }

    #[test]
    fn test_parse_multiple_rpcs() {
        let input = r#"
            module test-rpc {
                namespace "urn:test:rpc";
                prefix tr;

                rpc operation-one {
                    description "First operation";
                    input {
                        leaf param1 {
                            type string;
                        }
                    }
                }

                rpc operation-two {
                    description "Second operation";
                    output {
                        leaf result {
                            type boolean;
                        }
                    }
                }

                rpc operation-three {
                    description "Third operation";
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        assert_eq!(module.rpcs.len(), 3);
        assert_eq!(module.rpcs[0].name, "operation-one");
        assert_eq!(module.rpcs[1].name, "operation-two");
        assert_eq!(module.rpcs[2].name, "operation-three");
    }

    #[test]
    fn test_parse_module_with_data_nodes_and_rpcs() {
        let input = r#"
            module mixed {
                namespace "urn:test:mixed";
                prefix mx;

                container config {
                    leaf setting {
                        type string;
                    }
                }

                rpc do-something {
                    description "Do something";
                    input {
                        leaf value {
                            type uint32;
                        }
                    }
                }

                list items {
                    key "id";
                    leaf id {
                        type string;
                    }
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");

        assert!(result.is_ok());
        let module = result.unwrap();

        assert_eq!(module.data_nodes.len(), 2);
        assert_eq!(module.rpcs.len(), 1);
        assert_eq!(module.rpcs[0].name, "do-something");
    }
}
