# Design Document: Intermediate Client Crate Architecture

## Overview

This design introduces an intermediate client crate architecture that simplifies rustconf usage by allowing crate authors to create dedicated client libraries. Instead of every end user running build-time code generation, a crate author creates an intermediate crate that performs generation once and publishes the result as a normal Rust library.

The key insights are:
1. Generated code can be treated as source code rather than build artifacts by generating into src/ instead of OUT_DIR
2. Static runtime components (HTTP abstractions, transport adapters, error types) can be extracted into a separate `rustconf-runtime` crate, reducing generated code size and simplifying maintenance

This design splits rustconf into two crates:
- **rustconf**: Build-time code generation (build-dependency only)
- **rustconf-runtime**: Runtime components (normal dependency, no build scripts)

## Architecture

### Current Architecture (Build-Time Generation)

```
End User Project
├── Cargo.toml (rustconf as build-dependency + runtime deps)
├── build.rs (runs rustconf code generation)
├── yang/ (YANG specifications)
└── src/
    └── main.rs (includes generated code via include! macro)
```

Every user must:
1. Add rustconf as build-dependency
2. Add transport dependencies (reqwest, hyper, etc.)
3. Create build.rs to run code generation
4. Manage YANG files
5. Wait for code generation on every clean build

### New Architecture (Intermediate Client Crate)

```
rustconf-runtime (new crate - runtime dependency)
├── src/
│   ├── lib.rs
│   ├── transport.rs (HttpTransport trait, RestconfClient)
│   ├── error.rs (RpcError type)
│   └── adapters/
│       ├── reqwest.rs (feature-gated)
│       └── hyper.rs (feature-gated)

Intermediate Client Crate (created by crate author)
├── Cargo.toml (rustconf as build-dep, rustconf-runtime as dep)
├── build.rs (generates code to src/generated/)
├── yang/ (YANG specifications)
└── src/
    ├── lib.rs (re-exports generated code + rustconf-runtime)
    └── generated/ (generated code committed to git)
        ├── types.rs (YANG types only)
        └── operations.rs (RPC functions only)

End User Project
├── Cargo.toml (only depends on intermediate client crate)
└── src/
    └── main.rs (uses client crate like any library)
```

End users only need to:
1. Add one dependency (the intermediate client crate)
2. Use the API directly (no build scripts, no code generation)

The intermediate client crate transitively depends on rustconf-runtime, so end users automatically get the runtime components.

## Components and Interfaces

### 1. rustconf-runtime Crate (New)

A new runtime-only crate that contains all static components needed by generated code:

**Public API:**

```rust
// src/transport.rs
pub trait HttpTransport: Send + Sync {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, RpcError>;
}

pub struct RestconfClient<T: HttpTransport> {
    base_url: String,
    transport: T,
    interceptors: Vec<Box<dyn RequestInterceptor>>,
}

impl<T: HttpTransport> RestconfClient<T> {
    pub fn new(base_url: impl Into<String>, transport: T) -> Result<Self, RpcError>;
    pub fn with_interceptor(self, interceptor: impl RequestInterceptor + 'static) -> Self;
}

pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub enum HttpMethod {
    GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD,
}

pub trait RequestInterceptor: Send + Sync {
    fn intercept(&self, request: &mut HttpRequest) -> Result<(), RpcError>;
}

// src/error.rs
pub enum RpcError {
    TransportError(String),
    SerializationError(String),
    DeserializationError(String),
    ValidationError(String),
    HttpError { status_code: u16, message: String },
    NotImplemented,
}

pub trait ErrorMapper: Send + Sync {
    fn map_error(&self, response: &HttpResponse) -> RpcError;
}

pub struct DefaultErrorMapper;

// src/adapters/reqwest.rs (feature-gated)
#[cfg(feature = "reqwest")]
pub mod reqwest_adapter {
    pub struct ReqwestTransport { ... }
    impl HttpTransport for ReqwestTransport { ... }
}

// src/adapters/hyper.rs (feature-gated)
#[cfg(feature = "hyper")]
pub mod hyper_adapter {
    pub struct HyperTransport { ... }
    impl HttpTransport for HyperTransport { ... }
}
```

**Dependencies:**
```toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Optional transport dependencies
reqwest = { version = "0.11", features = ["json"], optional = true }
hyper = { version = "0.14", optional = true }
hyper-tls = { version = "0.5", optional = true }

[features]
default = []
reqwest = ["dep:reqwest"]
hyper = ["dep:hyper", "dep:hyper-tls"]
```

### 2. RustconfBuilder Enhancements

The existing `RustconfBuilder` needs minimal changes:

```rust
impl RustconfBuilder {
    /// Set the output directory for generated code.
    /// Can be set to src/generated/ for intermediate crate pattern.
    pub fn output_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_dir = path.into();
        self.config.output_dir = self.output_dir.clone();
        self
    }
    
    /// Enable modular output (multiple files instead of single file).
    /// Recommended for intermediate crate pattern.
    pub fn modular_output(mut self, enable: bool) -> Self {
        self.config.modular_output = enable;
        self
    }
    
    // Existing methods remain unchanged
    pub fn yang_file(mut self, path: impl Into<PathBuf>) -> Self { ... }
    pub fn enable_validation(mut self, enable: bool) -> Self { ... }
    pub fn enable_restful_rpcs(mut self, enable: bool) -> Self { ... }
    pub fn generate(self) -> Result<(), BuildError> { ... }
}
```

### 3. Code Generator Modifications

The `CodeGenerator` will be simplified by removing HTTP abstraction generation (now in rustconf-runtime):

```rust
impl CodeGenerator {
    /// Generate Rust code from a YANG module.
    pub fn generate(&self, module: &YangModule) -> Result<GeneratedCode, GeneratorError> {
        if self.config.modular_output {
            self.generate_modular(module)
        } else {
            self.generate_single_file(module)
        }
    }
    
    fn generate_modular(&self, module: &YangModule) -> Result<GeneratedCode, GeneratorError> {
        let mut files = Vec::new();
        
        // Generate mod.rs with module declarations and re-exports
        files.push(self.generate_mod_file(module)?);
        
        // Generate types.rs with YANG-derived types only
        files.push(self.generate_types_file(module)?);
        
        // Generate operations.rs with RPC functions (uses rustconf-runtime types)
        if self.config.enable_restful_rpcs && (!module.rpcs.is_empty() || !module.data_nodes.is_empty()) {
            files.push(self.generate_operations_file(module)?);
        }
        
        // Generate validation.rs if needed
        if self.config.enable_validation {
            files.push(self.generate_validation_file(module)?);
        }
        
        Ok(GeneratedCode { files })
    }
    
    fn generate_types_file(&self, module: &YangModule) -> Result<GeneratedFile, GeneratorError> {
        let mut content = String::new();
        
        // Add use statements
        content.push_str("use serde::{Deserialize, Serialize};\n");
        if self.config.enable_validation {
            content.push_str("use super::validation::*;\n");
        }
        content.push('\n');
        
        // Generate type definitions (no HTTP abstractions)
        let type_gen = types::TypeGenerator::new(&self.config);
        for typedef in &module.typedefs {
            content.push_str(&type_gen.generate_typedef(typedef)?);
            content.push('\n');
        }
        
        for data_node in &module.data_nodes {
            content.push_str(&type_gen.generate_data_node(data_node, module)?);
            content.push('\n');
        }
        
        Ok(GeneratedFile {
            path: self.config.output_dir.join("types.rs"),
            content,
        })
    }
    
    fn generate_operations_file(&self, module: &YangModule) -> Result<GeneratedFile, GeneratorError> {
        let mut content = String::new();
        
        // Add use statements (imports from rustconf-runtime)
        content.push_str("use rustconf_runtime::{RestconfClient, HttpTransport, RpcError};\n");
        content.push_str("use super::types::*;\n");
        content.push('\n');
        
        // Generate operation functions (no HTTP abstractions, uses rustconf-runtime)
        let ops_gen = operations::OperationsGenerator::new(&self.config);
        content.push_str(&ops_gen.generate_operations_module(module)?);
        
        Ok(GeneratedFile {
            path: self.config.output_dir.join("operations.rs"),
            content,
        })
    }
    
    fn generate_mod_file(&self, module: &YangModule) -> Result<GeneratedFile, GeneratorError> {
        let mut content = String::new();
        
        content.push_str("// This file is automatically generated by rustconf.\n");
        content.push_str("// DO NOT EDIT MANUALLY.\n\n");
        
        // Declare submodules
        content.push_str("pub mod types;\n");
        if self.config.enable_restful_rpcs {
            content.push_str("pub mod operations;\n");
        }
        if self.config.enable_validation {
            content.push_str("pub mod validation;\n");
        }
        content.push('\n');
        
        // Re-export commonly used items
        content.push_str("pub use types::*;\n");
        if self.config.enable_restful_rpcs {
            content.push_str("pub use operations::*;\n");
        }
        content.push('\n');
        
        // Re-export rustconf-runtime items
        content.push_str("pub use rustconf_runtime::{\n");
        content.push_str("    RestconfClient,\n");
        content.push_str("    HttpTransport,\n");
        content.push_str("    HttpRequest,\n");
        content.push_str("    HttpResponse,\n");
        content.push_str("    HttpMethod,\n");
        content.push_str("    RpcError,\n");
        content.push_str("    RequestInterceptor,\n");
        content.push_str("};\n");
        
        Ok(GeneratedFile {
            path: self.config.output_dir.join("mod.rs"),
            content,
        })
    }
}
```

### 4. Generated Code Structure (Revised)

With rustconf-runtime, the generated code is much simpler:

```
src/generated/
├── mod.rs          // Module declarations and re-exports
├── types.rs        // YANG-derived types only
├── operations.rs   // RPC operation functions (uses rustconf-runtime types)
└── validation.rs   // Validation types and logic (if enabled)
```

No more generated HTTP abstractions, transport adapters, or error types. These live in rustconf-runtime.

### 5. Intermediate Crate Structure (Revised)

**Cargo.toml:**
```toml
[package]
name = "my-device-client"
version = "0.1.0"
edition = "2021"

[features]
default = ["rustconf-runtime/reqwest"]

[dependencies]
rustconf-runtime = { version = "0.1" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
rustconf = "0.1"
```

**build.rs:**
```rust
fn main() {
    rustconf::RustconfBuilder::new()
        .yang_file("yang/device-management.yang")
        .search_path("yang/")
        .output_dir("src/generated")
        .enable_validation(true)
        .enable_restful_rpcs(true)
        .modular_output(true)
        .module_name("device_management")
        .generate()
        .expect("Failed to generate RESTCONF bindings");
    
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=yang/");
}
```

**src/lib.rs:**
```rust
//! My Device Client
//!
//! Type-safe Rust bindings for the Device Management RESTCONF API.

// Re-export generated code
pub mod generated;

// Re-export for convenience
pub use generated::*;

// Re-export rustconf-runtime items
pub use rustconf_runtime::{RestconfClient, HttpTransport, RpcError};

// Re-export transport adapters based on rustconf-runtime features
#[cfg(feature = "rustconf-runtime/reqwest")]
pub use rustconf_runtime::reqwest_adapter;

#[cfg(feature = "rustconf-runtime/hyper")]
pub use rustconf_runtime::hyper_adapter;
```

The crate author controls which transport to use by enabling rustconf-runtime features, not by adding separate dependencies. This is much simpler than the previous approach.

## Data Models

### GeneratorConfig

```rust
pub struct GeneratorConfig {
    /// Output directory for generated code
    pub output_dir: PathBuf,
    
    /// Name of the generated module
    pub module_name: String,
    
    /// Enable validation code generation
    pub enable_validation: bool,
    
    /// Enable RESTful RPC implementations
    pub enable_restful_rpcs: bool,
    
    /// Enable XML serialization support
    pub enable_xml: bool,
    
    /// Generate modular output (multiple files) vs single file
    pub modular_output: bool,
    
    /// Derive Debug trait on generated types
    pub derive_debug: bool,
    
    /// Derive Clone trait on generated types
    pub derive_clone: bool,
    
    /// Namespace handling mode
    pub namespace_mode: NamespaceMode,
}
```

### GeneratedCode

```rust
pub struct GeneratedCode {
    /// List of generated files
    pub files: Vec<GeneratedFile>,
}

pub struct GeneratedFile {
    /// Path where the file should be written
    pub path: PathBuf,
    
    /// Content of the generated file
    pub content: String,
}
```

### Intermediate Crate Metadata

While not enforced by rustconf, recommended metadata for intermediate crates:

```toml
[package.metadata.rustconf]
yang_version = "1.1"
yang_modules = ["device-management.yang"]
generated_at = "2024-01-15T10:30:00Z"
rustconf_version = "0.1.0"
```

## Error Handling

### Build-Time Errors

The existing `BuildError` type handles all build-time errors:

```rust
pub enum BuildError {
    ConfigurationError { message: String },
    ParseError(ParseError),
    GeneratorError(GeneratorError),
    IoError(std::io::Error),
}
```

For intermediate crate authors, errors during `cargo build` will be reported clearly:

```
error: Failed to generate RESTCONF bindings
  --> build.rs:5:10
   |
5  |         .generate()
   |          ^^^^^^^^^ YANG parse error: unexpected token at line 42
```

### Runtime Errors

End users of the intermediate crate see the same `RpcError` type:

```rust
pub enum RpcError {
    TransportError(String),
    SerializationError(String),
    DeserializationError(String),
    ValidationError(String),
    HttpError { status_code: u16, message: String },
    NotImplemented,
}
```

## Testing Strategy

### Unit Tests

1. **RustconfBuilder Configuration Tests**
   - Test output_dir accepts src/generated paths
   - Test validation of src/ directory paths
   - Test error handling for invalid paths

2. **Modular Generation Tests**
   - Test that modular_output generates multiple files
   - Test that each file has correct module structure
   - Test that generated mod.rs correctly declares submodules

3. **File Organization Tests**
   - Test types.rs contains only type definitions
   - Test operations.rs contains only operation functions
   - Test transport.rs contains HTTP abstractions
   - Test adapters are feature-gated correctly

4. **Integration Tests**
   - Create a test intermediate crate
   - Generate code to src/generated
   - Compile the intermediate crate
   - Use the intermediate crate from a test project

### Property-Based Tests

Property-based tests will validate universal correctness properties across all inputs. Each property test should run a minimum of 100 iterations and be tagged with its corresponding design property.


## Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### Property 1: Output Directory Flexibility

*For any* valid filesystem path, when set as the output directory via `output_dir()`, rustconf SHALL successfully generate code to that location.

**Validates: Requirements 3.6, 7.1**

### Property 2: Generated Code Completeness

*For any* YANG module, when code is generated, the output SHALL include all types, operations, and validation logic defined in the YANG specification.

**Validates: Requirements 2.3, 2.4, 2.5, 5.2**

### Property 3: Multi-Module Generation

*For any* set of YANG modules specified, when code is generated, the output SHALL include code for all specified modules and their transitive imports.

**Validates: Requirements 5.3, 5.4**

### Property 4: Valid Rust Code Generation

*For any* YANG module, when code is generated to src/, the output SHALL be syntactically valid Rust code that compiles without errors.

**Validates: Requirements 7.2, 7.3, 11.5**

### Property 5: No Include Macro in Source Generation

*For any* code generated to a src/ directory, the generated files SHALL NOT contain `include!` macro invocations.

**Validates: Requirements 7.4**

### Property 6: API Compatibility Across Generation Methods

*For any* YANG module, the public API (types, functions, method signatures) generated to OUT_DIR SHALL be identical to the API generated to src/.

**Validates: Requirements 9.1, 9.2, 9.3, 9.4**

### Property 7: Validation Logic Preservation

*For any* YANG type with constraints, the generated validation logic SHALL be identical whether generated to OUT_DIR or src/, and SHALL validate the same set of valid and invalid inputs.

**Validates: Requirements 10.1, 10.2**

### Property 8: Error Type Consistency

*For any* error condition, the RpcError type and error messages generated to src/ SHALL be identical to those generated to OUT_DIR.

**Validates: Requirements 10.3, 10.4**

### Property 9: Validation Error Messages

*For any* validation failure, the error message SHALL reference the specific YANG constraint that was violated.

**Validates: Requirements 10.5**

### Property 10: Feature-Gated Compilation

*For any* valid feature configuration (including no default features), an intermediate client crate SHALL compile successfully.

**Validates: Requirements 6.5**

### Property 11: Modular File Organization

*For any* YANG module, when modular generation is enabled, the generated code SHALL be organized into separate files (types.rs, operations.rs, validation.rs) with a valid mod.rs that declares all submodules.

**Validates: Requirements 2.1, 7.3**

### Property 12: Runtime Type Imports

*For any* generated code with RESTful RPCs enabled, the operations.rs file SHALL import types from rustconf_runtime (RestconfClient, HttpTransport, RpcError) and SHALL NOT contain definitions of these types.

**Validates: Requirements 12.4**

