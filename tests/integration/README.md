# Integration Test Harness

This crate validates rustconf-generated RESTCONF client and server code against live network OS emulators. It catches protocol-level mismatches, serialization quirks, and YANG interpretation differences that unit tests cannot detect.

## Quick start

Run the Tier 1 tests (no emulator needed):

```bash
cargo test -p rustconf-integration-tests
```

Run the full suite against a live emulator:

```bash
RUSTCONF_INTEGRATION_TEST=1 cargo test -p rustconf-integration-tests
```

When `RUSTCONF_INTEGRATION_TEST` is not set to `1`, emulator-dependent tests skip gracefully and the build stays green.

## Architecture

```
tests/integration/
├── build.rs              # Generates client/server code from vendor YANG models
├── config/               # Reference TOML configs per emulator
│   ├── crpd.toml
│   └── netopeer2.toml
├── fixtures/             # RFC 7951 JSON fixtures for test state setup
│   ├── interfaces.json
│   └── system.json
├── src/
│   ├── lib.rs            # Re-exports all harness modules
│   ├── config.rs         # HarnessConfig — env var and TOML loading
│   ├── emulators/        # EmulatorConfig trait + vendor implementations
│   │   ├── crpd.rs       # Juniper cRPD defaults
│   │   └── netopeer2.rs  # sysrepo/Netopeer2 defaults
│   ├── error.rs          # HarnessError unified error type
│   ├── fixture.rs        # FixtureManager — apply/teardown test state
│   ├── harness.rs        # TestHarness — container lifecycle + RESTCONF client
│   ├── multi_emulator.rs # MultiEmulatorRunner — run suites across emulators
│   ├── reporter.rs       # ConformanceReporter — text and JUnit XML reports
│   └── generated/        # Build-time generated code (do not edit)
├── tests/
│   ├── common/mod.rs     # skip_unless_integration!() and skip_unless_emulator!() macros
│   ├── client_tests.rs   # Client RESTCONF operations (GET, PUT, PATCH, RPC)
│   ├── error_tests.rs    # Error scenarios (404, malformed body, constraint violations)
│   ├── roundtrip_tests.rs# Serialization round-trips and fixture apply/teardown
│   ├── server_tests.rs   # Server conformance (compare generated server vs emulator)
│   └── multi_emulator_tests.rs  # Multi-emulator report aggregation
└── yang/                 # Vendor YANG models (sourced from YangModels/yang)
    ├── ietf/             # IETF standard models
    └── juniper/          # Juniper-specific models
```

## Key components

### TestHarness

Manages the full emulator lifecycle using `testcontainers-rs`:

1. Starts a container from the emulator's image
2. Maps the RESTCONF port to a random host port
3. Polls the health check endpoint until it returns 200 or times out
4. Provides a configured `RestconfClient` with Basic auth
5. Stops and removes the container on teardown

```rust
let config = JunosCrpdConfig::new();
let harness_config = HarnessConfig::from_env();
let mut harness = TestHarness::new(config, &harness_config);

harness.start().await?;
let client = harness.restconf_client()?;
// ... run tests against client ...
harness.stop().await?;
```

### EmulatorConfig trait

Abstracts vendor-specific details so the harness works with any RESTCONF emulator. Two implementations are provided:

| Emulator | Struct | Image | Port | TLS | YANG models |
|---|---|---|---|---|---|
| Juniper cRPD | `JunosCrpdConfig` | `crpd:23.4R1.10` | 3000 | yes | `yang/juniper/` |
| Netopeer2 | `NetopeerConfig` | `sysrepo/netopeer2:latest` | 6443 | yes | `yang/ietf/` |

Adding a new emulator means implementing `EmulatorConfig` and dropping the vendor's YANG models into `yang/<vendor>/`.

### FixtureManager

Handles test state setup and teardown via RESTCONF:

1. **Apply** — GETs the current state (saved for rollback), then PUTs the fixture data
2. **Teardown** — Restores original state in reverse order (LIFO)

Fixtures are JSON files following RFC 7951 encoding:

```json
{
  "resource_path": "/data/ietf-interfaces:interfaces",
  "data": {
    "ietf-interfaces:interfaces": {
      "interface": [
        {
          "name": "ge-0/0/0",
          "type": "iana-if-type:ethernetCsmacd",
          "enabled": true
        }
      ]
    }
  }
}
```

### ConformanceReporter

Collects test results and produces two report formats:

- **Text report** — Human-readable, grouped by YANG module, with pass/fail/skip markers and failure details
- **JUnit XML** — For CI integration, one `<testsuite>` per YANG module

Example text output:

```
=== Conformance Report: Juniper cRPD 23.4R1 ===

Module: ietf-interfaces (3 tests)
  ✓ GET  /data/ietf-interfaces:interfaces
  ✓ PUT  /data/ietf-interfaces:interfaces/interface=ge-0/0/0
  ✗ PATCH /data/ietf-interfaces:interfaces/interface=ge-0/0/0
      Expected: 200
      Actual:   405
      Request:  PATCH /restconf/data/ietf-interfaces:interfaces/interface=ge-0%2F0%2F0

Summary: 2 passed, 1 failed, 0 skipped (of 3 total)
```

## Configuration

The harness loads configuration from environment variables, TOML files, or both (env vars override TOML values).

### Environment variables

| Variable | Description | Default |
|---|---|---|
| `RUSTCONF_INTEGRATION_TEST` | Set to `1` to enable emulator-dependent tests | unset (tests skip) |
| `RUSTCONF_EMULATOR_TYPE` | Emulator type (`crpd`, `netopeer2`) | `crpd` |
| `RUSTCONF_CONTAINER_IMAGE` | Override container image | per emulator |
| `RUSTCONF_RESTCONF_PORT` | Override RESTCONF port | per emulator |
| `RUSTCONF_USERNAME` | Override username | per emulator |
| `RUSTCONF_PASSWORD` | Override password | per emulator |
| `RUSTCONF_HEALTH_TIMEOUT_SECS` | Health check timeout in seconds | `120` |
| `RUSTCONF_TEST_TIMEOUT_SECS` | Per-test timeout in seconds | `30` |
| `RUSTCONF_SKIP_TLS_VERIFY` | Skip TLS certificate verification (`true`/`false`) | `true` |
| `RUSTCONF_BASE_URL` | Override RESTCONF base URL | computed from container |

### TOML config files

Reference configs live in `config/`. Load one with `HarnessConfig::from_file()`:

```toml
[emulator]
type = "crpd"
image = "crpd:23.4R1.10"
restconf_port = 3000
username = "root"
password = "Juniper1"
base_path = "/restconf"
uses_tls = true
skip_tls_verify = true
health_check_path = "/restconf"
yang_model_dir = "yang/juniper"

[timeouts]
health_check_secs = 120
test_timeout_secs = 30
```

## Test tiers

Tests are split into two tiers based on whether they need a live emulator:

### Tier 1 — Always runs (no emulator)

These run as part of normal `cargo test`:

- Configuration loading and validation
- Fixture JSON parsing round-trips
- Conformance report generation and structure
- Multi-emulator report separation
- Error type construction and display

### Tier 2 — Requires emulator

These run only when `RUSTCONF_INTEGRATION_TEST=1` and a container runtime (Docker or Podman) is available:

- Client RESTCONF operations against the live emulator
- Serialization write-read round-trips
- Error scenario validation (404, malformed body, constraint violations)
- Server conformance comparison (generated server vs emulator)
- Fixture apply-teardown state restoration

## YANG model management

Vendor YANG models are stored in `yang/` and checked into the repo. The `build.rs` script invokes rustconf code generation at build time:

- Models that parse successfully produce generated client/server code in `src/generated/`
- Models with unsupported YANG constructs are logged and skipped — tests for those modules skip at runtime
- Regeneration triggers automatically when YANG files change (`cargo:rerun-if-changed`)

To update models, pull from [YangModels/yang](https://github.com/YangModels/yang) and place them in the appropriate vendor directory.

## CI integration

The harness is designed for CI with graceful degradation:

- **No emulator available** — all Tier 2 tests skip, exit code 0
- **Emulator available** — full suite runs, results output as standard cargo test output and optional JUnit XML
- **Gating** — controlled entirely by `RUSTCONF_INTEGRATION_TEST=1`

To add integration tests to a CI pipeline:

```yaml
integration-tests:
  runs-on: ubuntu-latest
  env:
    RUSTCONF_INTEGRATION_TEST: "1"
  steps:
    - uses: actions/checkout@v4
    - name: Pull emulator image
      run: docker pull crpd:23.4R1.10
    - name: Run integration tests
      run: cargo test -p rustconf-integration-tests
```

## Adding a new emulator

1. Implement the `EmulatorConfig` trait in `src/emulators/<vendor>.rs`
2. Add the vendor's YANG models to `yang/<vendor>/`
3. Create a reference TOML config in `config/<vendor>.toml`
4. Register the new type in `multi_emulator::create_harness()` so it can be selected by name
