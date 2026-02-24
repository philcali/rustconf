# Device Client - Intermediate Client Crate Example

This example demonstrates the **intermediate client crate pattern** for rustconf. Instead of requiring end users to run build-time code generation, this pattern allows you to create a reusable client library that:

- Generates code once during development
- Commits generated code to version control
- Publishes as a normal Rust library
- Eliminates rustconf and YANG dependencies for end users

## Structure

```
intermediate-client/
├── Cargo.toml          # rustconf as build-dep, rustconf-runtime as dep
├── build.rs            # Generates code to src/generated/
├── yang/               # YANG specifications
│   └── device-management.yang
└── src/
    ├── lib.rs          # Re-exports generated code
    └── generated/      # Generated code (committed to git)
        ├── mod.rs
        ├── types.rs
        ├── operations.rs
        └── validation.rs
```

## Building the Intermediate Crate

As the crate author, you run:

```bash
cd examples/intermediate-client
cargo build
```

This generates code to `src/generated/`. You would then commit these files:

```bash
git add src/generated/
git commit -m "Update generated bindings"
```

## Using the Intermediate Crate

End users add your crate as a dependency:

```toml
[dependencies]
device-client = "0.1"
```

And use it like any normal library:

```rust
use device_client::{RestconfClient, reqwest_adapter::ReqwestTransport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = ReqwestTransport::new();
    let client = RestconfClient::new("https://device.example.com", transport)?;
    
    // Get system information
    let info = device_client::get_system_info(&client).await?;
    println!("Hostname: {}", info.hostname.unwrap_or_default());
    println!("Uptime: {} seconds", info.uptime_seconds.unwrap_or(0));
    
    // Restart device with delay
    let input = device_client::RestartDeviceInput {
        delay_seconds: Some(30),
    };
    let result = device_client::restart_device(&client, &input).await?;
    println!("Restart initiated: {}", result.success.unwrap_or(false));
    
    Ok(())
}
```

## Benefits

- **No build.rs for end users**: Faster compilation, simpler setup
- **No rustconf dependency**: Smaller dependency tree
- **No YANG files needed**: Everything is in the published crate
- **Better IDE support**: Generated code is indexed like normal source
- **Version control**: Changes to generated code are visible in diffs

## Publishing

To publish this crate:

1. Update version in `Cargo.toml`
2. Regenerate code if YANG changed: `cargo build`
3. Commit generated code changes
4. Publish: `cargo publish`

End users get a clean, simple dependency with no build-time overhead.
