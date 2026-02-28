//! Unit tests for server-side notification publisher generation (Task 10.1)

use crate::generator::server_notifications::ServerNotificationGenerator;
use crate::generator::GeneratorConfig;
use crate::parser::{DataNode, Leaf, Notification, TypeSpec, YangModule};

#[test]
fn test_generate_notification_publisher_empty_module() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_generate_notification_publisher_struct() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "system-restart".to_string(),
            description: Some("System is restarting".to_string()),
            data_nodes: vec![],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check NotificationPublisher struct
    assert!(result.contains("pub struct NotificationPublisher {"));
    assert!(result.contains("subscribers: Arc<RwLock<Vec<Arc<dyn NotificationSubscriber>>>>,"));

    // Check constructor
    assert!(result.contains("pub fn new() -> Self {"));

    // Check subscribe method
    assert!(result
        .contains("pub async fn subscribe(&self, subscriber: Arc<dyn NotificationSubscriber>)"));

    // Check subscriber_count method
    assert!(result.contains("pub async fn subscriber_count(&self) -> usize {"));

    // Check Default impl
    assert!(result.contains("impl Default for NotificationPublisher {"));
}

#[test]
fn test_generate_notification_subscriber_trait() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "alarm".to_string(),
            description: Some("System alarm".to_string()),
            data_nodes: vec![],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check NotificationSubscriber trait
    assert!(result.contains("pub trait NotificationSubscriber: Send + Sync {"));
    assert!(result.contains("async fn on_notification("));
    assert!(result.contains("notification_name: &str,"));
    assert!(result.contains("payload: Vec<u8>,"));
    assert!(result.contains(") -> Result<(), String>;"));
}

#[test]
fn test_generate_publish_method_for_notification() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "link-up".to_string(),
            description: Some("Link is up".to_string()),
            data_nodes: vec![DataNode::Leaf(Leaf {
                name: "interface".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: false,
            })],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check publish method
    assert!(result.contains("pub async fn publish_link_up(&self, notification: notifications::LinkUp) -> Result<(), String> {"));

    // Check serialization
    assert!(result.contains("serde_json::to_vec(&notification)"));

    // Check notification delivery
    assert!(result.contains("subscriber.on_notification(notification_name, payload.clone()).await"));
}

#[test]
fn test_generate_notification_types() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "interface-state-change".to_string(),
            description: Some("Interface state has changed".to_string()),
            data_nodes: vec![
                DataNode::Leaf(Leaf {
                    name: "interface-name".to_string(),
                    description: Some("Name of the interface".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
                DataNode::Leaf(Leaf {
                    name: "new-state".to_string(),
                    description: Some("New state of the interface".to_string()),
                    type_spec: TypeSpec::String {
                        length: None,
                        pattern: None,
                    },
                    mandatory: true,
                    default: None,
                    config: false,
                }),
            ],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check notification module
    assert!(result.contains("pub mod notifications {"));

    // Check notification struct
    assert!(result.contains("pub struct InterfaceStateChange {"));

    // Check fields
    assert!(result.contains("pub interface_name: String,"));
    assert!(result.contains("pub new_state: String,"));

    // Check derive attributes
    assert!(result.contains("#[derive("));
    assert!(result.contains("Serialize"));
    assert!(result.contains("Deserialize"));
}

#[test]
fn test_generate_multiple_publish_methods() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![
            Notification {
                name: "link-up".to_string(),
                description: Some("Link is up".to_string()),
                data_nodes: vec![],
            },
            Notification {
                name: "link-down".to_string(),
                description: Some("Link is down".to_string()),
                data_nodes: vec![],
            },
        ],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check both publish methods
    assert!(
        result.contains("pub async fn publish_link_up(&self, notification: notifications::LinkUp)")
    );
    assert!(result
        .contains("pub async fn publish_link_down(&self, notification: notifications::LinkDown)"));
}

#[test]
fn test_notification_with_description() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "alarm".to_string(),
            description: Some("Critical system alarm\nRequires immediate attention".to_string()),
            data_nodes: vec![],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check rustdoc comments
    assert!(result.contains("/// Critical system alarm"));
    assert!(result.contains("/// Requires immediate attention"));
}

#[test]
fn test_notification_without_description() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "event".to_string(),
            description: None,
            data_nodes: vec![],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check default rustdoc comment
    assert!(result.contains("/// Notification payload for event."));
}

#[test]
fn test_subscriber_management_methods() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "event".to_string(),
            description: None,
            data_nodes: vec![],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check subscriber management methods
    assert!(result
        .contains("pub async fn subscribe(&self, subscriber: Arc<dyn NotificationSubscriber>)"));
    assert!(result.contains("pub async fn subscriber_count(&self) -> usize {"));
    assert!(result.contains("pub async fn clear_subscribers(&self) {"));
}

#[test]
fn test_concurrent_subscriber_support() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "event".to_string(),
            description: None,
            data_nodes: vec![],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check that subscribers are stored in Arc<RwLock<>> for concurrent access
    assert!(result.contains("Arc<RwLock<Vec<Arc<dyn NotificationSubscriber>>>>,"));

    // Check that subscribe uses write lock
    assert!(result.contains("let mut subs = self.subscribers.write().await;"));

    // Check that subscriber_count uses read lock
    assert!(result.contains("self.subscribers.read().await.len()"));
}

#[test]
fn test_notification_serialization_according_to_yang() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "alarm".to_string(),
            description: Some("System alarm".to_string()),
            data_nodes: vec![DataNode::Leaf(Leaf {
                name: "severity".to_string(),
                description: None,
                type_spec: TypeSpec::String {
                    length: None,
                    pattern: None,
                },
                mandatory: true,
                default: None,
                config: false,
            })],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check that serialization uses serde_json according to YANG schema
    assert!(result.contains("// Serialize notification to JSON according to YANG schema"));
    assert!(result.contains("serde_json::to_vec(&notification)"));
}

#[test]
fn test_delivery_failure_handling() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "event".to_string(),
            description: None,
            data_nodes: vec![],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check that delivery failures are handled gracefully
    assert!(result.contains("// Deliver to all subscribers, handling failures gracefully"));
    assert!(result.contains("let mut delivery_errors = Vec::new();"));
    assert!(result.contains("delivery_errors.push(error_msg);"));

    // Check that failures are logged
    assert!(result.contains("eprintln!"));

    // Check that summary is logged
    assert!(result.contains("if !delivery_errors.is_empty() {"));
    assert!(result.contains("Notification delivery completed with {} failures"));
}

#[test]
fn test_transport_delivery_mechanism() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "status-update".to_string(),
            description: Some("Status update notification".to_string()),
            data_nodes: vec![],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check that notifications are delivered through subscriber interface
    assert!(result.contains("subscriber.on_notification(notification_name, payload.clone()).await"));

    // Check that notification name is passed
    assert!(result.contains("let notification_name = \"status-update\";"));
}

#[test]
fn test_concurrent_notification_delivery() {
    let config = GeneratorConfig::default();
    let generator = ServerNotificationGenerator::new(&config);

    let module = YangModule {
        name: "test".to_string(),
        namespace: "urn:test".to_string(),
        prefix: "t".to_string(),
        yang_version: None,
        imports: vec![],
        typedefs: vec![],
        groupings: vec![],
        data_nodes: vec![],
        rpcs: vec![],
        notifications: vec![Notification {
            name: "event".to_string(),
            description: None,
            data_nodes: vec![],
        }],
    };

    let result = generator.generate_notification_publisher(&module).unwrap();

    // Check that delivery iterates over all subscribers
    assert!(result.contains("for (idx, subscriber) in subscribers.iter().enumerate() {"));

    // Check that payload is cloned for each subscriber
    assert!(result.contains("payload.clone()"));
}
