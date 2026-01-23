//! Unit tests for typedef and grouping expansion
//! Task 5.3: Implement typedef and grouping expansion
//! Requirements: 1.5

#[cfg(test)]
mod tests {
    use crate::parser::{DataNode, TypeSpec, YangParser};

    #[test]
    fn test_expand_simple_typedef() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef percent {
                    type uint8;
                }
                
                leaf usage {
                    type percent;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let mut module = result.unwrap();

        // Before expansion, the leaf should have a TypedefRef
        if let DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert!(matches!(leaf.type_spec, TypeSpec::TypedefRef { .. }));
        }

        // Expand the module
        let expand_result = parser.expand_module(&mut module);
        assert!(
            expand_result.is_ok(),
            "Failed to expand: {:?}",
            expand_result.err()
        );

        // After expansion, the leaf should have the concrete type
        if let DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "usage");
            assert!(matches!(leaf.type_spec, TypeSpec::Uint8 { .. }));
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_expand_typedef_with_constraints() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef port-number {
                    type uint16 {
                        range "1..65535";
                    }
                }
                
                leaf server-port {
                    type port-number;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());

        let mut module = result.unwrap();
        let expand_result = parser.expand_module(&mut module);
        assert!(expand_result.is_ok());

        // After expansion, the leaf should have uint16 with range constraint
        if let DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "server-port");
            if let TypeSpec::Uint16 { range } = &leaf.type_spec {
                assert!(range.is_some());
            } else {
                panic!("Expected Uint16 type with range constraint");
            }
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_expand_nested_typedef() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef base-type {
                    type uint32;
                }
                
                typedef derived-type {
                    type base-type;
                }
                
                leaf value {
                    type derived-type;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());

        let mut module = result.unwrap();
        let expand_result = parser.expand_module(&mut module);
        assert!(expand_result.is_ok());

        // After expansion, should resolve to uint32
        if let DataNode::Leaf(leaf) = &module.data_nodes[0] {
            assert_eq!(leaf.name, "value");
            assert!(matches!(leaf.type_spec, TypeSpec::Uint32 { .. }));
        } else {
            panic!("Expected Leaf data node");
        }
    }

    #[test]
    fn test_expand_simple_grouping() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping endpoint {
                    leaf address {
                        type string;
                    }
                    leaf port {
                        type uint16;
                    }
                }
                
                container server {
                    uses endpoint;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let mut module = result.unwrap();

        // Before expansion, container should have a Uses node
        if let DataNode::Container(container) = &module.data_nodes[0] {
            assert_eq!(container.children.len(), 1);
            assert!(matches!(container.children[0], DataNode::Uses(_)));
        }

        // Expand the module
        let expand_result = parser.expand_module(&mut module);
        assert!(
            expand_result.is_ok(),
            "Failed to expand: {:?}",
            expand_result.err()
        );

        // After expansion, Uses should be replaced with the grouping's nodes
        if let DataNode::Container(container) = &module.data_nodes[0] {
            assert_eq!(container.name, "server");
            assert_eq!(container.children.len(), 2);

            // Check first leaf
            if let DataNode::Leaf(leaf) = &container.children[0] {
                assert_eq!(leaf.name, "address");
                assert!(matches!(leaf.type_spec, TypeSpec::String { .. }));
            } else {
                panic!("Expected Leaf data node for address");
            }

            // Check second leaf
            if let DataNode::Leaf(leaf) = &container.children[1] {
                assert_eq!(leaf.name, "port");
                assert!(matches!(leaf.type_spec, TypeSpec::Uint16 { .. }));
            } else {
                panic!("Expected Leaf data node for port");
            }
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_expand_grouping_with_typedef() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                typedef ip-address {
                    type string;
                }
                
                grouping network-config {
                    leaf ip {
                        type ip-address;
                    }
                }
                
                container interface {
                    uses network-config;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());

        let mut module = result.unwrap();
        let expand_result = parser.expand_module(&mut module);
        assert!(expand_result.is_ok());

        // After expansion, both grouping and typedef should be resolved
        if let DataNode::Container(container) = &module.data_nodes[0] {
            assert_eq!(container.children.len(), 1);

            if let DataNode::Leaf(leaf) = &container.children[0] {
                assert_eq!(leaf.name, "ip");
                assert!(matches!(leaf.type_spec, TypeSpec::String { .. }));
            } else {
                panic!("Expected Leaf data node");
            }
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_expand_nested_grouping() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping inner {
                    leaf value {
                        type string;
                    }
                }
                
                grouping outer {
                    uses inner;
                    leaf extra {
                        type uint32;
                    }
                }
                
                container root {
                    uses outer;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());

        let mut module = result.unwrap();
        let expand_result = parser.expand_module(&mut module);
        assert!(expand_result.is_ok());

        // After expansion, should have both leaves from nested groupings
        if let DataNode::Container(container) = &module.data_nodes[0] {
            // Debug: print what we actually got
            println!("Container has {} children", container.children.len());
            for (i, child) in container.children.iter().enumerate() {
                println!("Child {}: {:?}", i, child);
            }

            assert_eq!(container.children.len(), 2);

            // First leaf from inner grouping
            if let DataNode::Leaf(leaf) = &container.children[0] {
                assert_eq!(leaf.name, "value");
            } else {
                panic!(
                    "Expected first Leaf data node, got: {:?}",
                    container.children[0]
                );
            }

            // Second leaf from outer grouping
            if let DataNode::Leaf(leaf) = &container.children[1] {
                assert_eq!(leaf.name, "extra");
            } else {
                panic!("Expected second Leaf data node");
            }
        } else {
            panic!("Expected Container data node");
        }
    }

    #[test]
    fn test_expand_multiple_uses() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping common {
                    leaf id {
                        type uint32;
                    }
                }
                
                container first {
                    uses common;
                }
                
                container second {
                    uses common;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());

        let mut module = result.unwrap();
        let expand_result = parser.expand_module(&mut module);
        assert!(expand_result.is_ok());

        // Both containers should have the expanded grouping
        assert_eq!(module.data_nodes.len(), 2);

        for data_node in &module.data_nodes {
            if let DataNode::Container(container) = data_node {
                assert_eq!(container.children.len(), 1);
                if let DataNode::Leaf(leaf) = &container.children[0] {
                    assert_eq!(leaf.name, "id");
                } else {
                    panic!("Expected Leaf data node");
                }
            } else {
                panic!("Expected Container data node");
            }
        }
    }

    #[test]
    fn test_error_undefined_typedef() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                leaf value {
                    type undefined-type;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());

        let mut module = result.unwrap();
        let expand_result = parser.expand_module(&mut module);

        // Should fail with semantic error
        assert!(expand_result.is_err());
        if let Err(e) = expand_result {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("Undefined typedef"));
            assert!(error_msg.contains("undefined-type"));
        }
    }

    #[test]
    fn test_error_undefined_grouping() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                container root {
                    uses undefined-grouping;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok());

        let mut module = result.unwrap();
        let expand_result = parser.expand_module(&mut module);

        // Should fail with semantic error
        assert!(expand_result.is_err());
        if let Err(e) = expand_result {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("Undefined grouping"));
            assert!(error_msg.contains("undefined-grouping"));
        }
    }

    #[test]
    fn test_expand_grouping_in_list() {
        let input = r#"
            module test {
                namespace "urn:test";
                prefix test;
                
                grouping key-value {
                    leaf name {
                        type string;
                    }
                    leaf value {
                        type string;
                    }
                }
                
                list items {
                    key "name";
                    uses key-value;
                }
            }
        "#;

        let mut parser = YangParser::new();
        let result = parser.parse_string(input, "test.yang");
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let mut module = result.unwrap();
        let expand_result = parser.expand_module(&mut module);
        assert!(
            expand_result.is_ok(),
            "Failed to expand: {:?}",
            expand_result.err()
        );

        // List should have expanded grouping nodes
        if let DataNode::List(list) = &module.data_nodes[0] {
            assert_eq!(list.name, "items");
            assert_eq!(list.children.len(), 2);
        } else {
            panic!("Expected List data node");
        }
    }
}
