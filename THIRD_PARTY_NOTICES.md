# Third-Party Notices

This project includes third-party dependencies.

## Scope

- Active workspace packages: `anzu-runtime`, `anzu-terminal`, and `anzu-winit`.
- Path dependencies also include `anzu-render-wgpu` and ROM packages.
- Browser distribution outputs may include compiled dependency code in generated wasm/js artifacts.

## Policy

- Source references and documentation links in this repository are used for guidance and attribution.
- Linking to external references does not by itself imply third-party code inclusion.
- If third-party source code is copied into this repository, preserve required notices and license terms with the copied material.

## Rust Dependency License Inventory

The legacy license-check workflow refers to a removed root-level `anzu-engine/` path and does not audit the active workspace. Its former configuration is no longer present in the repository.

Do not treat the legacy workflow as an audit of the active workspace. Add and validate a workspace-level license audit before using this file as release evidence.

## Release Checklist

Before release/distribution:

1. Audit dependency licenses for all active workspace packages and path dependencies.
2. Verify no copied third-party source is missing required attribution.
3. Update this file if dependency/license posture changes materially.
