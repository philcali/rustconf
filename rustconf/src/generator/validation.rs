//! Validation type generation for constrained YANG types.

use crate::parser::{LengthConstraint, PatternConstraint, RangeConstraint, TypeSpec};

/// Generate a validated type wrapper for a constrained type.
pub fn generate_validated_type(
    type_name: &str,
    type_spec: &TypeSpec,
    derive_debug: bool,
    derive_clone: bool,
) -> Option<String> {
    match type_spec {
        TypeSpec::Int8 { range: Some(range) } => Some(generate_range_validated_type(
            type_name,
            "i8",
            range,
            derive_debug,
            derive_clone,
        )),
        TypeSpec::Int16 { range: Some(range) } => Some(generate_range_validated_type(
            type_name,
            "i16",
            range,
            derive_debug,
            derive_clone,
        )),
        TypeSpec::Int32 { range: Some(range) } => Some(generate_range_validated_type(
            type_name,
            "i32",
            range,
            derive_debug,
            derive_clone,
        )),
        TypeSpec::Int64 { range: Some(range) } => Some(generate_range_validated_type(
            type_name,
            "i64",
            range,
            derive_debug,
            derive_clone,
        )),
        TypeSpec::Uint8 { range: Some(range) } => Some(generate_range_validated_type(
            type_name,
            "u8",
            range,
            derive_debug,
            derive_clone,
        )),
        TypeSpec::Uint16 { range: Some(range) } => Some(generate_range_validated_type(
            type_name,
            "u16",
            range,
            derive_debug,
            derive_clone,
        )),
        TypeSpec::Uint32 { range: Some(range) } => Some(generate_range_validated_type(
            type_name,
            "u32",
            range,
            derive_debug,
            derive_clone,
        )),
        TypeSpec::Uint64 { range: Some(range) } => Some(generate_range_validated_type(
            type_name,
            "u64",
            range,
            derive_debug,
            derive_clone,
        )),
        TypeSpec::String { length, pattern } => {
            if length.is_some() || pattern.is_some() {
                Some(generate_string_validated_type(
                    type_name,
                    length.as_ref(),
                    pattern.as_ref(),
                    derive_debug,
                    derive_clone,
                ))
            } else {
                None
            }
        }
        TypeSpec::Binary {
            length: Some(length),
        } => Some(generate_binary_validated_type(
            type_name,
            length,
            derive_debug,
            derive_clone,
        )),
        _ => None,
    }
}

/// Generate a range-validated numeric type.
fn generate_range_validated_type(
    type_name: &str,
    base_type: &str,
    range: &RangeConstraint,
    derive_debug: bool,
    derive_clone: bool,
) -> String {
    let mut output = String::new();

    // Generate rustdoc comment
    output.push_str(&format!(
        "/// Validated {} type with range constraints.\n",
        base_type
    ));
    output.push_str("///\n");
    output.push_str("/// Allowed ranges:\n");
    for r in &range.ranges {
        output.push_str(&format!("/// - {} to {}\n", r.min, r.max));
    }

    // Generate derive attributes
    let mut derives = vec!["PartialEq", "Eq"];
    if derive_debug {
        derives.insert(0, "Debug");
    }
    if derive_clone {
        derives.insert(if derive_debug { 1 } else { 0 }, "Clone");
    }
    output.push_str(&format!("#[derive({})]\n", derives.join(", ")));

    output.push_str(&format!("pub struct {} {{\n", type_name));
    output.push_str(&format!("    value: {},\n", base_type));
    output.push_str("}\n\n");

    // Generate implementation
    output.push_str(&format!("impl {} {{\n", type_name));

    // Generate new method with validation
    output.push_str("    /// Create a new validated value.\n");
    output.push_str("    ///\n");
    output.push_str("    /// # Errors\n");
    output.push_str("    ///\n");
    output.push_str("    /// Returns `ValidationError::OutOfRange` if the value is outside the allowed ranges.\n");
    output.push_str(&format!(
        "    pub fn new(value: {}) -> Result<Self, ValidationError> {{\n",
        base_type
    ));

    // Generate validation logic
    output.push_str("        let valid = ");
    let range_checks: Vec<String> = range
        .ranges
        .iter()
        .map(|r| format!("({}..={}).contains(&value)", r.min, r.max))
        .collect();
    output.push_str(&range_checks.join(" || "));
    output.push_str(";\n\n");

    output.push_str("        if valid {\n");
    output.push_str("            Ok(Self { value })\n");
    output.push_str("        } else {\n");
    output.push_str("            Err(ValidationError::OutOfRange {\n");
    output.push_str("                value: value.to_string(),\n");

    // Generate constraint string
    let constraint_str = range
        .ranges
        .iter()
        .map(|r| format!("{}..{}", r.min, r.max))
        .collect::<Vec<_>>()
        .join(" | ");
    output.push_str(&format!(
        "                constraint: \"{}\".to_string(),\n",
        constraint_str
    ));
    output.push_str("            })\n");
    output.push_str("        }\n");
    output.push_str("    }\n\n");

    // Generate value getter
    output.push_str("    /// Get the inner value.\n");
    output.push_str(&format!("    pub fn value(&self) -> {} {{\n", base_type));
    output.push_str("        self.value\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate TryFrom implementation
    output.push_str(&format!(
        "impl TryFrom<{}> for {} {{\n",
        base_type, type_name
    ));
    output.push_str("    type Error = ValidationError;\n\n");
    output.push_str(&format!(
        "    fn try_from(value: {}) -> Result<Self, Self::Error> {{\n",
        base_type
    ));
    output.push_str("        Self::new(value)\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate Serialize implementation
    output.push_str(&format!("impl serde::Serialize for {} {{\n", type_name));
    output.push_str("    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\n");
    output.push_str("    where\n");
    output.push_str("        S: serde::Serializer,\n");
    output.push_str("    {\n");
    output.push_str("        self.value.serialize(serializer)\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate Deserialize implementation with validation
    output.push_str(&format!(
        "impl<'de> serde::Deserialize<'de> for {} {{\n",
        type_name
    ));
    output.push_str("    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\n");
    output.push_str("    where\n");
    output.push_str("        D: serde::Deserializer<'de>,\n");
    output.push_str("    {\n");
    output.push_str(&format!(
        "        let value = {}::deserialize(deserializer)?;\n",
        base_type
    ));
    output.push_str("        Self::new(value).map_err(serde::de::Error::custom)\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    output
}

/// Generate a string-validated type with length and/or pattern constraints.
fn generate_string_validated_type(
    type_name: &str,
    length: Option<&LengthConstraint>,
    pattern: Option<&PatternConstraint>,
    derive_debug: bool,
    derive_clone: bool,
) -> String {
    let mut output = String::new();

    // Generate rustdoc comment
    output.push_str("/// Validated String type with constraints.\n");
    output.push_str("///\n");
    if let Some(len) = length {
        output.push_str("/// Length constraints:\n");
        for l in &len.lengths {
            output.push_str(&format!("/// - {} to {} characters\n", l.min, l.max));
        }
    }
    if let Some(pat) = pattern {
        output.push_str(&format!("/// Pattern: {}\n", pat.pattern));
    }

    // Generate derive attributes
    let mut derives = vec!["PartialEq", "Eq"];
    if derive_debug {
        derives.insert(0, "Debug");
    }
    if derive_clone {
        derives.insert(if derive_debug { 1 } else { 0 }, "Clone");
    }
    output.push_str(&format!("#[derive({})]\n", derives.join(", ")));

    output.push_str(&format!("pub struct {} {{\n", type_name));
    output.push_str("    value: String,\n");
    output.push_str("}\n\n");

    // Generate implementation
    output.push_str(&format!("impl {} {{\n", type_name));

    // Generate new method with validation
    output.push_str("    /// Create a new validated string.\n");
    output.push_str("    ///\n");
    output.push_str("    /// # Errors\n");
    output.push_str("    ///\n");
    output.push_str(
        "    /// Returns `ValidationError` if the string violates length or pattern constraints.\n",
    );
    output.push_str("    pub fn new(value: String) -> Result<Self, ValidationError> {\n");

    // Generate length validation
    if let Some(len) = length {
        output.push_str("        let length = value.len() as u64;\n");
        output.push_str("        let length_valid = ");
        let length_checks: Vec<String> = len
            .lengths
            .iter()
            .map(|l| format!("({}..={}).contains(&length)", l.min, l.max))
            .collect();
        output.push_str(&length_checks.join(" || "));
        output.push_str(";\n\n");

        output.push_str("        if !length_valid {\n");
        output.push_str("            return Err(ValidationError::InvalidLength {\n");
        output.push_str("                value: value.clone(),\n");

        let constraint_str = len
            .lengths
            .iter()
            .map(|l| format!("{}..{}", l.min, l.max))
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&format!(
            "                constraint: \"{}\".to_string(),\n",
            constraint_str
        ));
        output.push_str("            });\n");
        output.push_str("        }\n\n");
    }

    // Generate pattern validation
    if let Some(pat) = pattern {
        output.push_str("        // Pattern validation\n");
        output.push_str(&format!("        let pattern = regex::Regex::new(r\"{}\").map_err(|_| ValidationError::InvalidPattern {{\n", pat.pattern));
        output.push_str("            value: value.clone(),\n");
        output.push_str(&format!(
            "            pattern: \"{}\".to_string(),\n",
            pat.pattern
        ));
        output.push_str("        })?;\n\n");

        output.push_str("        if !pattern.is_match(&value) {\n");
        output.push_str("            return Err(ValidationError::InvalidPattern {\n");
        output.push_str("                value: value.clone(),\n");
        output.push_str(&format!(
            "                pattern: \"{}\".to_string(),\n",
            pat.pattern
        ));
        output.push_str("            });\n");
        output.push_str("        }\n\n");
    }

    output.push_str("        Ok(Self { value })\n");
    output.push_str("    }\n\n");

    // Generate value getter
    output.push_str("    /// Get the inner value.\n");
    output.push_str("    pub fn value(&self) -> &str {\n");
    output.push_str("        &self.value\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate TryFrom implementation
    output.push_str(&format!("impl TryFrom<String> for {} {{\n", type_name));
    output.push_str("    type Error = ValidationError;\n\n");
    output.push_str("    fn try_from(value: String) -> Result<Self, Self::Error> {\n");
    output.push_str("        Self::new(value)\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate Serialize implementation
    output.push_str(&format!("impl serde::Serialize for {} {{\n", type_name));
    output.push_str("    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\n");
    output.push_str("    where\n");
    output.push_str("        S: serde::Serializer,\n");
    output.push_str("    {\n");
    output.push_str("        self.value.serialize(serializer)\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate Deserialize implementation with validation
    output.push_str(&format!(
        "impl<'de> serde::Deserialize<'de> for {} {{\n",
        type_name
    ));
    output.push_str("    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\n");
    output.push_str("    where\n");
    output.push_str("        D: serde::Deserializer<'de>,\n");
    output.push_str("    {\n");
    output.push_str("        let value = String::deserialize(deserializer)?;\n");
    output.push_str("        Self::new(value).map_err(serde::de::Error::custom)\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    output
}

/// Generate a binary-validated type with length constraints.
fn generate_binary_validated_type(
    type_name: &str,
    length: &LengthConstraint,
    derive_debug: bool,
    derive_clone: bool,
) -> String {
    let mut output = String::new();

    // Generate rustdoc comment
    output.push_str("/// Validated binary type with length constraints.\n");
    output.push_str("///\n");
    output.push_str("/// Length constraints:\n");
    for l in &length.lengths {
        output.push_str(&format!("/// - {} to {} bytes\n", l.min, l.max));
    }

    // Generate derive attributes
    let mut derives = vec!["PartialEq", "Eq"];
    if derive_debug {
        derives.insert(0, "Debug");
    }
    if derive_clone {
        derives.insert(if derive_debug { 1 } else { 0 }, "Clone");
    }
    output.push_str(&format!("#[derive({})]\n", derives.join(", ")));

    output.push_str(&format!("pub struct {} {{\n", type_name));
    output.push_str("    value: Vec<u8>,\n");
    output.push_str("}\n\n");

    // Generate implementation
    output.push_str(&format!("impl {} {{\n", type_name));

    // Generate new method with validation
    output.push_str("    /// Create a new validated binary value.\n");
    output.push_str("    ///\n");
    output.push_str("    /// # Errors\n");
    output.push_str("    ///\n");
    output.push_str("    /// Returns `ValidationError::InvalidLength` if the length is outside the allowed ranges.\n");
    output.push_str("    pub fn new(value: Vec<u8>) -> Result<Self, ValidationError> {\n");

    // Generate length validation
    output.push_str("        let length = value.len() as u64;\n");
    output.push_str("        let length_valid = ");
    let length_checks: Vec<String> = length
        .lengths
        .iter()
        .map(|l| format!("({}..={}).contains(&length)", l.min, l.max))
        .collect();
    output.push_str(&length_checks.join(" || "));
    output.push_str(";\n\n");

    output.push_str("        if !length_valid {\n");
    output.push_str("            return Err(ValidationError::InvalidLength {\n");
    output.push_str("                value: format!(\"{:?}\", value),\n");

    let constraint_str = length
        .lengths
        .iter()
        .map(|l| format!("{}..{}", l.min, l.max))
        .collect::<Vec<_>>()
        .join(" | ");
    output.push_str(&format!(
        "                constraint: \"{}\".to_string(),\n",
        constraint_str
    ));
    output.push_str("            });\n");
    output.push_str("        }\n\n");

    output.push_str("        Ok(Self { value })\n");
    output.push_str("    }\n\n");

    // Generate value getter
    output.push_str("    /// Get the inner value.\n");
    output.push_str("    pub fn value(&self) -> &[u8] {\n");
    output.push_str("        &self.value\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate TryFrom implementation
    output.push_str(&format!("impl TryFrom<Vec<u8>> for {} {{\n", type_name));
    output.push_str("    type Error = ValidationError;\n\n");
    output.push_str("    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {\n");
    output.push_str("        Self::new(value)\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate Serialize implementation
    output.push_str(&format!("impl serde::Serialize for {} {{\n", type_name));
    output.push_str("    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\n");
    output.push_str("    where\n");
    output.push_str("        S: serde::Serializer,\n");
    output.push_str("    {\n");
    output.push_str("        self.value.serialize(serializer)\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    // Generate Deserialize implementation with validation
    output.push_str(&format!(
        "impl<'de> serde::Deserialize<'de> for {} {{\n",
        type_name
    ));
    output.push_str("    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\n");
    output.push_str("    where\n");
    output.push_str("        D: serde::Deserializer<'de>,\n");
    output.push_str("    {\n");
    output.push_str("        let value = Vec::<u8>::deserialize(deserializer)?;\n");
    output.push_str("        Self::new(value).map_err(serde::de::Error::custom)\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    output
}

/// Generate the ValidationError type.
pub fn generate_validation_error(derive_debug: bool, derive_clone: bool) -> String {
    let mut output = String::new();

    output.push_str("/// Validation error for constrained types.\n");

    // Generate derive with or without Debug and Clone
    let mut derives = vec!["PartialEq", "Eq"];
    if derive_debug {
        derives.insert(0, "Debug");
    }
    if derive_clone {
        derives.insert(if derive_debug { 1 } else { 0 }, "Clone");
    }
    output.push_str(&format!("#[derive({})]\n", derives.join(", ")));

    output.push_str("pub enum ValidationError {\n");
    output.push_str("    /// Value is outside the allowed range.\n");
    output.push_str("    OutOfRange {\n");
    output.push_str("        value: String,\n");
    output.push_str("        constraint: String,\n");
    output.push_str("    },\n");
    output.push_str("    /// String length is outside the allowed range.\n");
    output.push_str("    InvalidLength {\n");
    output.push_str("        value: String,\n");
    output.push_str("        constraint: String,\n");
    output.push_str("    },\n");
    output.push_str("    /// String does not match the required pattern.\n");
    output.push_str("    InvalidPattern {\n");
    output.push_str("        value: String,\n");
    output.push_str("        pattern: String,\n");
    output.push_str("    },\n");
    output.push_str("}\n\n");

    output.push_str("impl std::fmt::Display for ValidationError {\n");
    output.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
    output.push_str("        match self {\n");
    output.push_str("            ValidationError::OutOfRange { value, constraint } => {\n");
    output.push_str("                write!(f, \"Value '{}' is outside allowed range: {}\", value, constraint)\n");
    output.push_str("            }\n");
    output.push_str("            ValidationError::InvalidLength { value, constraint } => {\n");
    output.push_str("                write!(f, \"Value '{}' has invalid length, expected: {}\", value, constraint)\n");
    output.push_str("            }\n");
    output.push_str("            ValidationError::InvalidPattern { value, pattern } => {\n");
    output.push_str(
        "                write!(f, \"Value '{}' does not match pattern: {}\", value, pattern)\n",
    );
    output.push_str("            }\n");
    output.push_str("        }\n");
    output.push_str("    }\n");
    output.push_str("}\n\n");

    output.push_str("impl std::error::Error for ValidationError {}\n");

    output
}
