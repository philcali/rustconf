//! Unit tests for the visitor pattern
//! Task 23.5: Add tests for visitor pattern
//! Requirements: 5.1, 5.2

#[cfg(test)]
mod tests {
    use crate::parser::{
        walk_data_node, walk_data_nodes, Case, Choice, Container, DataNode, DataNodeVisitor, Leaf,
        LeafList, List, TypeSpec, Uses,
    };

    // ========== Test Visitor Implementations ==========

    /// A simple visitor that counts the number of each node type
    struct NodeCounter {
        leaf_count: usize,
        leaf_list_count: usize,
        container_count: usize,
        list_count: usize,
        choice_count: usize,
        case_count: usize,
        uses_count: usize,
    }

    impl NodeCounter {
        fn new() -> Self {
            Self {
                leaf_count: 0,
                leaf_list_count: 0,
                container_count: 0,
                list_count: 0,
                choice_count: 0,
                case_count: 0,
                uses_count: 0,
            }
        }

        fn total_count(&self) -> usize {
            self.leaf_count
                + self.leaf_list_count
                + self.container_count
                + self.list_count
                + self.choice_count
                + self.case_count
                + self.uses_count
        }
    }

    impl DataNodeVisitor for NodeCounter {
        type Error = std::convert::Infallible;

        fn visit_leaf(&mut self, _leaf: &Leaf) -> Result<(), Self::Error> {
            self.leaf_count += 1;
            Ok(())
        }

        fn visit_leaf_list(&mut self, _leaf_list: &LeafList) -> Result<(), Self::Error> {
            self.leaf_list_count += 1;
            Ok(())
        }

        fn visit_container(&mut self, container: &Container) -> Result<(), Self::Error> {
            self.container_count += 1;
            // Continue visiting children
            for child in &container.children {
                walk_data_node(child, self)?;
            }
            Ok(())
        }

        fn visit_list(&mut self, list: &List) -> Result<(), Self::Error> {
            self.list_count += 1;
            // Continue visiting children
            for child in &list.children {
                walk_data_node(child, self)?;
            }
            Ok(())
        }

        fn visit_choice(&mut self, choice: &Choice) -> Result<(), Self::Error> {
            self.choice_count += 1;
            // Continue visiting cases
            for case in &choice.cases {
                self.visit_case(case)?;
            }
            Ok(())
        }

        fn visit_case(&mut self, case: &Case) -> Result<(), Self::Error> {
            self.case_count += 1;
            // Continue visiting children
            for child in &case.data_nodes {
                walk_data_node(child, self)?;
            }
            Ok(())
        }

        fn visit_uses(&mut self, _uses: &Uses) -> Result<(), Self::Error> {
            self.uses_count += 1;
            Ok(())
        }
    }

    /// A visitor that collects leaf names
    struct LeafNameCollector {
        names: Vec<String>,
    }

    impl LeafNameCollector {
        fn new() -> Self {
            Self { names: Vec::new() }
        }
    }

    impl DataNodeVisitor for LeafNameCollector {
        type Error = std::convert::Infallible;

        fn visit_leaf(&mut self, leaf: &Leaf) -> Result<(), Self::Error> {
            self.names.push(leaf.name.clone());
            Ok(())
        }

        fn visit_container(&mut self, container: &Container) -> Result<(), Self::Error> {
            for child in &container.children {
                walk_data_node(child, self)?;
            }
            Ok(())
        }

        fn visit_list(&mut self, list: &List) -> Result<(), Self::Error> {
            for child in &list.children {
                walk_data_node(child, self)?;
            }
            Ok(())
        }

        fn visit_choice(&mut self, choice: &Choice) -> Result<(), Self::Error> {
            for case in &choice.cases {
                self.visit_case(case)?;
            }
            Ok(())
        }

        fn visit_case(&mut self, case: &Case) -> Result<(), Self::Error> {
            for child in &case.data_nodes {
                walk_data_node(child, self)?;
            }
            Ok(())
        }
    }

    /// A visitor that can fail with an error
    struct FailingVisitor {
        fail_on_leaf: Option<String>,
    }

    impl FailingVisitor {
        fn new(fail_on_leaf: Option<String>) -> Self {
            Self { fail_on_leaf }
        }
    }

    impl DataNodeVisitor for FailingVisitor {
        type Error = String;

        fn visit_leaf(&mut self, leaf: &Leaf) -> Result<(), Self::Error> {
            if let Some(ref fail_name) = self.fail_on_leaf {
                if &leaf.name == fail_name {
                    return Err(format!("Failed on leaf: {}", leaf.name));
                }
            }
            Ok(())
        }

        fn visit_container(&mut self, container: &Container) -> Result<(), Self::Error> {
            for child in &container.children {
                walk_data_node(child, self)?;
            }
            Ok(())
        }

        fn visit_list(&mut self, list: &List) -> Result<(), Self::Error> {
            for child in &list.children {
                walk_data_node(child, self)?;
            }
            Ok(())
        }
    }

    // ========== Helper Functions ==========

    fn create_test_leaf(name: &str) -> Leaf {
        Leaf {
            name: name.to_string(),
            description: None,
            type_spec: TypeSpec::String {
                length: None,
                pattern: None,
            },
            mandatory: false,
            default: None,
            config: true,
        }
    }

    fn create_test_leaf_list(name: &str) -> LeafList {
        LeafList {
            name: name.to_string(),
            description: None,
            type_spec: TypeSpec::String {
                length: None,
                pattern: None,
            },
            config: true,
        }
    }

    fn create_test_container(name: &str, children: Vec<DataNode>) -> Container {
        Container {
            name: name.to_string(),
            description: None,
            config: true,
            mandatory: false,
            children,
        }
    }

    fn create_test_list(name: &str, children: Vec<DataNode>) -> List {
        List {
            name: name.to_string(),
            description: None,
            config: true,
            keys: vec!["id".to_string()],
            children,
        }
    }

    // ========== Visitor Pattern Tests ==========

    #[test]
    fn test_visit_single_leaf() {
        let leaf = create_test_leaf("test-leaf");
        let node = DataNode::Leaf(leaf);

        let mut counter = NodeCounter::new();
        walk_data_node(&node, &mut counter).unwrap();

        assert_eq!(counter.leaf_count, 1);
        assert_eq!(counter.total_count(), 1);
    }

    #[test]
    fn test_visit_single_leaf_list() {
        let leaf_list = create_test_leaf_list("test-leaf-list");
        let node = DataNode::LeafList(leaf_list);

        let mut counter = NodeCounter::new();
        walk_data_node(&node, &mut counter).unwrap();

        assert_eq!(counter.leaf_list_count, 1);
        assert_eq!(counter.total_count(), 1);
    }

    #[test]
    fn test_visit_container_with_children() {
        let container = create_test_container(
            "test-container",
            vec![
                DataNode::Leaf(create_test_leaf("leaf1")),
                DataNode::Leaf(create_test_leaf("leaf2")),
                DataNode::LeafList(create_test_leaf_list("leaf-list1")),
            ],
        );
        let node = DataNode::Container(container);

        let mut counter = NodeCounter::new();
        walk_data_node(&node, &mut counter).unwrap();

        assert_eq!(counter.container_count, 1);
        assert_eq!(counter.leaf_count, 2);
        assert_eq!(counter.leaf_list_count, 1);
        assert_eq!(counter.total_count(), 4);
    }

    #[test]
    fn test_visit_nested_containers() {
        let inner_container = create_test_container(
            "inner",
            vec![
                DataNode::Leaf(create_test_leaf("inner-leaf1")),
                DataNode::Leaf(create_test_leaf("inner-leaf2")),
            ],
        );

        let outer_container = create_test_container(
            "outer",
            vec![
                DataNode::Leaf(create_test_leaf("outer-leaf")),
                DataNode::Container(inner_container),
            ],
        );

        let node = DataNode::Container(outer_container);

        let mut counter = NodeCounter::new();
        walk_data_node(&node, &mut counter).unwrap();

        assert_eq!(counter.container_count, 2);
        assert_eq!(counter.leaf_count, 3);
        assert_eq!(counter.total_count(), 5);
    }

    #[test]
    fn test_visit_list_with_children() {
        let list = create_test_list(
            "test-list",
            vec![
                DataNode::Leaf(create_test_leaf("id")),
                DataNode::Leaf(create_test_leaf("name")),
            ],
        );
        let node = DataNode::List(list);

        let mut counter = NodeCounter::new();
        walk_data_node(&node, &mut counter).unwrap();

        assert_eq!(counter.list_count, 1);
        assert_eq!(counter.leaf_count, 2);
        assert_eq!(counter.total_count(), 3);
    }

    #[test]
    fn test_visit_choice_with_cases() {
        let case1 = Case {
            name: "case1".to_string(),
            description: None,
            data_nodes: vec![DataNode::Leaf(create_test_leaf("case1-leaf"))],
        };

        let case2 = Case {
            name: "case2".to_string(),
            description: None,
            data_nodes: vec![
                DataNode::Leaf(create_test_leaf("case2-leaf1")),
                DataNode::Leaf(create_test_leaf("case2-leaf2")),
            ],
        };

        let choice = Choice {
            name: "test-choice".to_string(),
            description: None,
            mandatory: false,
            cases: vec![case1, case2],
        };

        let node = DataNode::Choice(choice);

        let mut counter = NodeCounter::new();
        walk_data_node(&node, &mut counter).unwrap();

        assert_eq!(counter.choice_count, 1);
        assert_eq!(counter.case_count, 2);
        assert_eq!(counter.leaf_count, 3);
        assert_eq!(counter.total_count(), 6);
    }

    #[test]
    fn test_visit_uses_node() {
        let uses = Uses {
            name: "test-grouping".to_string(),
            description: None,
        };
        let node = DataNode::Uses(uses);

        let mut counter = NodeCounter::new();
        walk_data_node(&node, &mut counter).unwrap();

        assert_eq!(counter.uses_count, 1);
        assert_eq!(counter.total_count(), 1);
    }

    #[test]
    fn test_walk_multiple_nodes() {
        let nodes = vec![
            DataNode::Leaf(create_test_leaf("leaf1")),
            DataNode::Leaf(create_test_leaf("leaf2")),
            DataNode::Container(create_test_container(
                "container1",
                vec![DataNode::Leaf(create_test_leaf("leaf3"))],
            )),
        ];

        let mut counter = NodeCounter::new();
        walk_data_nodes(&nodes, &mut counter).unwrap();

        assert_eq!(counter.leaf_count, 3);
        assert_eq!(counter.container_count, 1);
        assert_eq!(counter.total_count(), 4);
    }

    #[test]
    fn test_walk_empty_node_list() {
        let nodes: Vec<DataNode> = vec![];

        let mut counter = NodeCounter::new();
        walk_data_nodes(&nodes, &mut counter).unwrap();

        assert_eq!(counter.total_count(), 0);
    }

    #[test]
    fn test_leaf_name_collector() {
        let container = create_test_container(
            "test-container",
            vec![
                DataNode::Leaf(create_test_leaf("alpha")),
                DataNode::Leaf(create_test_leaf("beta")),
                DataNode::Container(create_test_container(
                    "inner",
                    vec![DataNode::Leaf(create_test_leaf("gamma"))],
                )),
            ],
        );

        let node = DataNode::Container(container);

        let mut collector = LeafNameCollector::new();
        walk_data_node(&node, &mut collector).unwrap();

        assert_eq!(collector.names.len(), 3);
        assert_eq!(collector.names, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_visitor_error_handling() {
        let container = create_test_container(
            "test-container",
            vec![
                DataNode::Leaf(create_test_leaf("leaf1")),
                DataNode::Leaf(create_test_leaf("fail-here")),
                DataNode::Leaf(create_test_leaf("leaf3")),
            ],
        );

        let node = DataNode::Container(container);

        let mut visitor = FailingVisitor::new(Some("fail-here".to_string()));
        let result = walk_data_node(&node, &mut visitor);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed on leaf: fail-here");
    }

    #[test]
    fn test_visitor_no_error_when_condition_not_met() {
        let container = create_test_container(
            "test-container",
            vec![
                DataNode::Leaf(create_test_leaf("leaf1")),
                DataNode::Leaf(create_test_leaf("leaf2")),
            ],
        );

        let node = DataNode::Container(container);

        let mut visitor = FailingVisitor::new(Some("non-existent".to_string()));
        let result = walk_data_node(&node, &mut visitor);

        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_nested_structure() {
        // Create a complex nested structure
        let inner_list = create_test_list(
            "inner-list",
            vec![
                DataNode::Leaf(create_test_leaf("list-key")),
                DataNode::Leaf(create_test_leaf("list-value")),
            ],
        );

        let choice = Choice {
            name: "protocol-choice".to_string(),
            description: None,
            mandatory: false,
            cases: vec![
                Case {
                    name: "tcp".to_string(),
                    description: None,
                    data_nodes: vec![DataNode::Leaf(create_test_leaf("tcp-port"))],
                },
                Case {
                    name: "udp".to_string(),
                    description: None,
                    data_nodes: vec![DataNode::Leaf(create_test_leaf("udp-port"))],
                },
            ],
        };

        let outer_container = create_test_container(
            "config",
            vec![
                DataNode::Leaf(create_test_leaf("enabled")),
                DataNode::List(inner_list),
                DataNode::Choice(choice),
                DataNode::LeafList(create_test_leaf_list("tags")),
            ],
        );

        let node = DataNode::Container(outer_container);

        let mut counter = NodeCounter::new();
        walk_data_node(&node, &mut counter).unwrap();

        assert_eq!(counter.container_count, 1);
        assert_eq!(counter.list_count, 1);
        assert_eq!(counter.choice_count, 1);
        assert_eq!(counter.case_count, 2);
        assert_eq!(counter.leaf_count, 5); // enabled, list-key, list-value, tcp-port, udp-port
        assert_eq!(counter.leaf_list_count, 1);
        assert_eq!(counter.total_count(), 11);
    }

    #[test]
    fn test_visitor_with_default_implementations() {
        // Create a visitor that only overrides visit_leaf
        struct MinimalVisitor {
            leaf_count: usize,
        }

        impl DataNodeVisitor for MinimalVisitor {
            type Error = std::convert::Infallible;

            fn visit_leaf(&mut self, _leaf: &Leaf) -> Result<(), Self::Error> {
                self.leaf_count += 1;
                Ok(())
            }

            // Use default implementations for other methods
        }

        let container = create_test_container(
            "test",
            vec![
                DataNode::Leaf(create_test_leaf("leaf1")),
                DataNode::Leaf(create_test_leaf("leaf2")),
            ],
        );

        let node = DataNode::Container(container);

        let mut visitor = MinimalVisitor { leaf_count: 0 };
        walk_data_node(&node, &mut visitor).unwrap();

        // Default container implementation should visit children
        assert_eq!(visitor.leaf_count, 2);
    }
}
