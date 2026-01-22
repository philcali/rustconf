//! Abstract Syntax Tree (AST) types for YANG specifications.

/// A parsed YANG module.
#[derive(Debug, Clone, PartialEq)]
pub struct YangModule {
    pub name: String,
    pub namespace: String,
    pub prefix: String,
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
        pattern: Option<String>,
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
}

/// Range constraint for numeric types.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeConstraint {
    pub ranges: Vec<Range>,
}

/// A single range with min and max values.
#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub min: i64,
    pub max: i64,
}

/// Length constraint for string and binary types.
#[derive(Debug, Clone, PartialEq)]
pub struct LengthConstraint {
    pub lengths: Vec<LengthRange>,
}

/// A single length range.
#[derive(Debug, Clone, PartialEq)]
pub struct LengthRange {
    pub min: u64,
    pub max: u64,
}

/// Enumeration value.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub name: String,
    pub value: Option<i32>,
    pub description: Option<String>,
}
