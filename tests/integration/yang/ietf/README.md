# IETF Standard YANG Models

This directory holds IETF standard YANG models that are imported as
dependencies by vendor-specific models (e.g. Juniper models in `../juniper/`).

## Populating this directory

YANG models are **not** checked into this repository. Obtain them from the
[YangModels/yang](https://github.com/YangModels/yang) repository.

```bash
# Clone the community YANG model collection
git clone --depth 1 https://github.com/YangModels/yang.git /tmp/yang-models

# Copy IETF standard models
cp /tmp/yang-models/standard/ietf/RFC/ietf-interfaces*.yang tests/integration/yang/ietf/
cp /tmp/yang-models/standard/ietf/RFC/ietf-inet-types*.yang tests/integration/yang/ietf/
cp /tmp/yang-models/standard/ietf/RFC/ietf-yang-types*.yang tests/integration/yang/ietf/
cp /tmp/yang-models/standard/ietf/RFC/ietf-ip*.yang         tests/integration/yang/ietf/
cp /tmp/yang-models/standard/ietf/RFC/iana-if-type*.yang    tests/integration/yang/ietf/
```

## Commonly needed modules

| Module | RFC | Purpose |
|---|---|---|
| `ietf-interfaces` | RFC 8343 | Interface data model (imported by most vendor interface modules) |
| `ietf-inet-types` | RFC 6991 | Common IP address / prefix types |
| `ietf-yang-types` | RFC 6991 | Common YANG derived types (counter, date-and-time, etc.) |
| `ietf-ip` | RFC 8344 | IPv4/IPv6 address configuration |
| `iana-if-type` | — | Interface type identities |

## Notes

- These models serve as import dependencies for vendor YANG models. They are
  passed to `rustconf::RustconfBuilder` via `search_path()` so the parser can
  resolve `import` statements.
- If a required IETF model is missing, the build script will report a parse
  error for the vendor model that imports it.
- Models should be sourced from the same IETF RFC versions expected by the
  vendor models you are testing against.
