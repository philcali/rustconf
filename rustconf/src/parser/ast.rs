//! Abstract Syntax Tree (AST) types for YANG specifications.

/// YANG version enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YangVersion {
    /// YANG version 1.0 (RFC 6020)
    V1_0,
    /// YANG version 1.1 (RFC 7950)
    V1_1,
}

/// Module header containing version and namespace information.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleHeader {
    pub yang_version: YangVersion,
    pub name: String,
    pub namespace: String,
    pub prefix: String,
}

/// A parsed YANG module.
#[derive(Debug, Clone, PartialEq)]
pub struct YangModule {
    pub name: String,
    pub namespace: String,
    pub prefix: String,
    pub yang_version: Option<YangVersion>,
    pub imports: Vec<Import>,
    pub typedefs: Vec<TypeDef>,
    pub groupings: Vec<Grouping>,
    pub data_nodes: Vec<DataNode>,
    pub rpcs: Vec<Rpc>,
    pub notifications: Vec<Notification>,
}

/// Module import statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub module: String,
    pub prefix: String,
    pub revision: Option<String>,
}

/// Type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    pub type_spec: TypeSpec,
    pub units: Option<String>,
    pub default: Option<String>,
    pub description: Option<String>,
}

/// Grouping definition for reusable data node collections.
#[derive(Debug, Clone, PartialEq)]
pub struct Grouping {
    pub name: String,
    pub description: Option<String>,
    pub data_nodes: Vec<DataNode>,
}

/// Data node variants in the YANG data tree.
#[derive(Debug, Clone, PartialEq)]
pub enum DataNode {
    Container(Container),
    List(List),
    Leaf(Leaf),
    LeafList(LeafList),
    Choice(Choice),
    Case(Case),
    Uses(Uses),
}

/// Uses statement for grouping expansion.
#[derive(Debug, Clone, PartialEq)]
pub struct Uses {
    pub name: String,
    pub description: Option<String>,
}

/// Container node.
#[derive(Debug, Clone, PartialEq)]
pub struct Container {
    pub name: String,
    pub description: Option<String>,
    pub config: bool,
    pub mandatory: bool,
    pub children: Vec<DataNode>,
}

/// List node.
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub name: String,
    pub description: Option<String>,
    pub config: bool,
    pub keys: Vec<String>,
    pub children: Vec<DataNode>,
}

/// Leaf node.
#[derive(Debug, Clone, PartialEq)]
pub struct Leaf {
    pub name: String,
    pub description: Option<String>,
    pub type_spec: TypeSpec,
    pub mandatory: bool,
    pub default: Option<String>,
    pub config: bool,
}

/// Leaf-list node.
#[derive(Debug, Clone, PartialEq)]
pub struct LeafList {
    pub name: String,
    pub description: Option<String>,
    pub type_spec: TypeSpec,
    pub config: bool,
}

/// Choice node for mutually exclusive options.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub name: String,
    pub description: Option<String>,
    pub mandatory: bool,
    pub cases: Vec<Case>,
}

/// Case within a choice.
#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    pub name: String,
    pub description: Option<String>,
    pub data_nodes: Vec<DataNode>,
}

/// RPC operation definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Rpc {
    pub name: String,
    pub description: Option<String>,
    pub input: Option<Vec<DataNode>>,
    pub output: Option<Vec<DataNode>>,
}

/// Notification definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    pub name: String,
    pub description: Option<String>,
    pub data_nodes: Vec<DataNode>,
}

/// YANG type specification.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpec {
    Int8 {
        range: Option<RangeConstraint>,
    },
    Int16 {
        range: Option<RangeConstraint>,
    },
    Int32 {
        range: Option<RangeConstraint>,
    },
    Int64 {
        range: Option<RangeConstraint>,
    },
    Uint8 {
        range: Option<RangeConstraint>,
    },
    Uint16 {
        range: Option<RangeConstraint>,
    },
    Uint32 {
        range: Option<RangeConstraint>,
    },
    Uint64 {
        range: Option<RangeConstraint>,
    },
    String {
        length: Option<LengthConstraint>,
        pattern: Option<PatternConstraint>,
    },
    Boolean,
    Enumeration {
        values: Vec<EnumValue>,
    },
    Union {
        types: Vec<TypeSpec>,
    },
    LeafRef {
        path: String,
    },
    Empty,
    Binary {
        length: Option<LengthConstraint>,
    },
    /// Reference to a typedef that needs to be resolved
    TypedefRef {
        name: String,
    },
}

/// Range constraint for numeric types.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeConstraint {
    pub ranges: Vec<Range>,
}

impl RangeConstraint {
    /// Create a new range constraint with the given ranges.
    pub fn new(ranges: Vec<Range>) -> Self {
        Self { ranges }
    }

    /// Validate that a value falls within the allowed ranges.
    pub fn validate(&self, value: i64) -> bool {
        self.ranges.iter().any(|range| range.contains(value))
    }
}

/// A single range with min and max values.
#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub min: i64,
    pub max: i64,
}

impl Range {
    /// Create a new range with the given min and max values.
    pub fn new(min: i64, max: i64) -> Self {
        Self { min, max }
    }

    /// Check if a value is within this range (inclusive).
    pub fn contains(&self, value: i64) -> bool {
        value >= self.min && value <= self.max
    }
}

/// Length constraint for string and binary types.
#[derive(Debug, Clone, PartialEq)]
pub struct LengthConstraint {
    pub lengths: Vec<LengthRange>,
}

impl LengthConstraint {
    /// Create a new length constraint with the given length ranges.
    pub fn new(lengths: Vec<LengthRange>) -> Self {
        Self { lengths }
    }

    /// Validate that a length falls within the allowed ranges.
    pub fn validate(&self, length: u64) -> bool {
        self.lengths.iter().any(|range| range.contains(length))
    }
}

/// A single length range.
#[derive(Debug, Clone, PartialEq)]
pub struct LengthRange {
    pub min: u64,
    pub max: u64,
}

impl LengthRange {
    /// Create a new length range with the given min and max values.
    pub fn new(min: u64, max: u64) -> Self {
        Self { min, max }
    }

    /// Check if a length is within this range (inclusive).
    pub fn contains(&self, length: u64) -> bool {
        length >= self.min && length <= self.max
    }
}

/// Pattern constraint for string types (regular expression).
#[derive(Debug, Clone, PartialEq)]
pub struct PatternConstraint {
    pub pattern: String,
}

impl PatternConstraint {
    /// Create a new pattern constraint with the given regex pattern.
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }

    /// Validate that a string matches the pattern.
    ///
    /// Note: This is a placeholder that always returns true.
    /// Actual regex validation will be implemented when needed.
    pub fn validate(&self, _value: &str) -> bool {
        // TODO: Implement actual regex validation using regex crate
        // For now, we just store the pattern and defer validation
        true
    }
}

/// Enumeration value.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub name: String,
    pub value: Option<i32>,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_constraint_validation() {
        let constraint = RangeConstraint::new(vec![Range::new(1, 10), Range::new(20, 30)]);

        // Values within ranges
        assert!(constraint.validate(1));
        assert!(constraint.validate(5));
        assert!(constraint.validate(10));
        assert!(constraint.validate(20));
        assert!(constraint.validate(25));
        assert!(constraint.validate(30));

        // Values outside ranges
        assert!(!constraint.validate(0));
        assert!(!constraint.validate(11));
        assert!(!constraint.validate(15));
        assert!(!constraint.validate(19));
        assert!(!constraint.validate(31));
    }

    #[test]
    fn test_range_contains() {
        let range = Range::new(10, 20);

        assert!(range.contains(10));
        assert!(range.contains(15));
        assert!(range.contains(20));
        assert!(!range.contains(9));
        assert!(!range.contains(21));
    }

    #[test]
    fn test_length_constraint_validation() {
        let constraint =
            LengthConstraint::new(vec![LengthRange::new(1, 10), LengthRange::new(20, 30)]);

        // Lengths within ranges
        assert!(constraint.validate(1));
        assert!(constraint.validate(5));
        assert!(constraint.validate(10));
        assert!(constraint.validate(20));
        assert!(constraint.validate(25));
        assert!(constraint.validate(30));

        // Lengths outside ranges
        assert!(!constraint.validate(0));
        assert!(!constraint.validate(11));
        assert!(!constraint.validate(15));
        assert!(!constraint.validate(19));
        assert!(!constraint.validate(31));
    }

    #[test]
    fn test_length_range_contains() {
        let range = LengthRange::new(5, 15);

        assert!(range.contains(5));
        assert!(range.contains(10));
        assert!(range.contains(15));
        assert!(!range.contains(4));
        assert!(!range.contains(16));
    }

    #[test]
    fn test_pattern_constraint_creation() {
        let pattern = PatternConstraint::new("[0-9]+".to_string());
        assert_eq!(pattern.pattern, "[0-9]+");

        // Placeholder validation always returns true
        assert!(pattern.validate("123"));
        assert!(pattern.validate("abc"));
    }

    #[test]
    fn test_yang_version_debug() {
        let v1_0 = YangVersion::V1_0;
        let v1_1 = YangVersion::V1_1;

        assert_eq!(format!("{:?}", v1_0), "V1_0");
        assert_eq!(format!("{:?}", v1_1), "V1_1");
    }

    #[test]
    fn test_module_header_creation() {
        let header = ModuleHeader {
            yang_version: YangVersion::V1_1,
            name: "test-module".to_string(),
            namespace: "urn:test:module".to_string(),
            prefix: "test".to_string(),
        };

        assert_eq!(header.yang_version, YangVersion::V1_1);
        assert_eq!(header.name, "test-module");
        assert_eq!(header.namespace, "urn:test:module");
        assert_eq!(header.prefix, "test");
    }
}
