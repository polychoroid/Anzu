# Third-Party Notices

This project includes third-party dependencies.

## Scope

- Runtime crate: `anzu-engine/`
- Distribution outputs may include compiled dependency code (for example wasm/js artifacts).

## Policy

- Source references and documentation links in this repository are used for guidance and attribution.
- Linking to external references does not by itself imply third-party code inclusion.
- If third-party source code is copied into this repository, preserve required notices and license terms with the copied material.

## Rust Dependency License Inventory

Use `cargo-deny` with `anzu-engine/deny.toml` to audit dependency licenses.

Install:

```bash
cargo install cargo-deny
```

Run:

```bash
cd anzu-engine
cargo deny --config deny.toml check licenses
```

## Release Checklist

Before release/distribution:

1. Run dependency license checks.
2. Verify no copied third-party source is missing required attribution.
3. Update this file if dependency/license posture changes materially.
