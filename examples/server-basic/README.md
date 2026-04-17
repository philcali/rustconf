# Server-Side Generation Example

This example demonstrates how to use rustconf's server-side code generation to create RESTCONF server handlers from YANG schemas.

## What it shows

1. **Stub handler for testing** — Use the generated `StubDeviceManagementHandler` as-is to test client code without real hardware. Every call is logged for inspection.

2. **Request routing** — The generated `RestconfRouter` parses RESTCONF URL paths, deserializes request bodies, dispatches to the correct handler method, and serializes responses.

3. **Custom handler** — Override specific trait methods with real logic while delegating unimplemented operations to the stub. This lets you incrementally build out a production server.

## How it works

The `build.rs` enables server generation:

```rust
rustconf::RustconfBuilder::new()
    .yang_file("yang/device-management.yang")
    .enable_server_generation(true)
    .generate()
    .expect("Failed to generate bindings");
```

This produces a `server/` subdirectory alongside the standard client code:

```
src/generated/
├── mod.rs
├── types.rs
├── operations.rs
├── validation.rs
└── server/
    ├── mod.rs
    ├── handlers.rs    # DeviceManagementHandler trait
    ├── stubs.rs       # StubDeviceManagementHandler
    ├── router.rs      # RestconfRouter
    └── registry.rs    # HandlerRegistry
```

## Running

```sh
cargo run -p server-basic-example
```
