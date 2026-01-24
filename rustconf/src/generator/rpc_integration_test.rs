//! Integration tests for RPC code generation (Task 10.1)

#[cfg(test)]
mod tests {
    use crate::generator::{CodeGenerator, GeneratorConfig};
    use crate::parser::{DataNode, Leaf, Rpc, TypeSpec, YangModule};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generated_rpc_code_compiles() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("generated");

        let config = GeneratorConfig {
            output_dir: output_dir.clone(),
            module_name: "rpc_test".to_string(),
            ..Default::default()
        };

        let generator = CodeGenerator::new(config);

        let module = YangModule {
            name: "interface-mgmt".to_string(),
            namespace: "urn:example:interface-mgmt".to_string(),
            prefix: "if-mgmt".to_string(),
            yang_version: None,
            imports: vec![],
            typedefs: vec![],
            groupings: vec![],
            data_nodes: vec![],
            rpcs: vec![
                Rpc {
                    name: "reset-interface".to_string(),
                    description: Some("Reset an interface to default state".to_string()),
                    input: Some(vec![DataNode::Leaf(Leaf {
                        name: "interface-name".to_string(),
                        description: Some("Name of the interface to reset".to_string()),
                        type_spec: TypeSpec::String {
                            length: None,
                            pattern: None,
                        },
                        mandatory: true,
                        default: None,
                        config: true,
                    })]),
                    output: None,
                },
                Rpc {
                    name: "get-statistics".to_string(),
                    description: Some("Get interface statistics".to_string()),
                    input: Some(vec![DataNode::Leaf(Leaf {
                        name: "interface-name".to_string(),
                        description: None,
                        type_spec: TypeSpec::String {
                            length: None,
                            pattern: None,
                        },
                        mandatory: true,
                        default: None,
                        config: true,
                    })]),
                    output: Some(vec![
                        DataNode::Leaf(Leaf {
                            name: "rx-packets".to_string(),
                            description: Some("Received packets".to_string()),
                            type_spec: TypeSpec::Uint64 { range: None },
                            mandatory: true,
                            default: None,
                            config: false,
                        }),
                        DataNode::Leaf(Leaf {
                            name: "tx-packets".to_string(),
                            description: Some("Transmitted packets".to_string()),
                            type_spec: TypeSpec::Uint64 { range: None },
                            mandatory: true,
                            default: None,
                            config: false,
                        }),
                    ]),
                },
            ],
            notifications: vec![],
        };

        let generated = generator.generate(&module).unwrap();
        generator.write_files(&generated).unwrap();

        let content = fs::read_to_string(output_dir.join("rpc_test.rs")).unwrap();

        // Verify the generated code contains expected elements
        assert!(content.contains("pub enum RpcError"));
        assert!(content.contains("pub mod operations"));
        assert!(content.contains("pub struct ResetInterfaceInput"));
        assert!(content.contains("pub struct GetStatisticsInput"));
        assert!(content.contains("pub struct GetStatisticsOutput"));
        assert!(content.contains("pub async fn reset_interface"));
        assert!(content.contains("pub async fn get_statistics"));

        // Verify RPC error variants
        assert!(content.contains("NetworkError(String)"));
        assert!(content.contains("ServerError { code: u16, message: String }"));
        assert!(content.contains("SerializationError(String)"));
        assert!(content.contains("InvalidInput(String)"));
        assert!(content.contains("NotImplemented"));

        // Verify function signatures
        assert!(content.contains("input: ResetInterfaceInput) -> Result<(), RpcError>"));
        assert!(
            content.contains("input: GetStatisticsInput) -> Result<GetStatisticsOutput, RpcError>")
        );

        // Verify rustdoc comments
        assert!(content.contains("/// Reset an interface to default state"));
        assert!(content.contains("/// Get interface statistics"));
        assert!(content.contains("/// # Errors"));
    }
}
