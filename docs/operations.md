# Operations

## Runtime Monitoring

Primary operational visibility currently comes from browser console logs.

- Startup errors: canvas lookup, window creation, renderer initialization.
- Manifest loader status: success summary or typed warning/error message.
- Render loop failures: surfaced to console and runtime exits event loop.

Recommended checks during runtime verification:
- Confirm no repeated surface/device errors in console.
- Confirm redraw/update/render loop remains active after window resize.
- Confirm manifest fetch resolves from served static root.

## Debugging

Common failure modes:
- `manifest.json` not found or blocked by incorrect serving path.
- Browser not supporting selected GPU backend path.
- Stale generated `static/pkg` files after code changes.

Debug workflow:
1. Rebuild with wasm-pack release output.
2. Hard-refresh browser page.
3. Inspect console messages for startup and render diagnostics.
4. Validate static server root is `anzu-engine/static`.

## Maintenance

- Keep Rust dependencies current with regular `cargo update` reviews.
- Re-run format/lint/build checks before merging.
- Keep docs synchronized whenever runtime contracts or commands change.
- Track milestone acceptance criteria in backlog as implementation evolves.
