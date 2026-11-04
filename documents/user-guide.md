# User Guide

## Current Status

Anzu currently provides a browser prototype, not a complete playable game. The app creates a local `You Touched My Boats` session, loads one GLTF asset, and requests a redraw after each 16 ms tick event.

The ROM maps keyboard input to actions, but action application only writes a log message. The session does not apply published updates to its state. As a result, keyboard input does not move boats or change the displayed state. The display currently projects one rotating asset instance.

## Run the Checked-In Browser Build

The browser page loads generated files from `docs/pkg/`. These files may not match the current Rust source.

1. Start a static file server from the repository root:

```bash
python3 -m http.server 8000 --directory docs
```

2. Open `http://localhost:8000/` in a browser.

## Input Mapping

The browser input adapter maps keyboard events to runtime input signals. The ROM currently maps:

- `W`, `A`, `S`, and `D` to actions for the first player.
- Arrow keys and `Enter` to actions for the second player.
- `Space` to a fire action for the first player.

These actions do not yet change the display. `Escape` maps to an input signal but does not close the app. The `Tab` input branch is unfinished and can panic.

## Known Limits

- The 16 ms interval sends one server tick. It is not an elapsed-time fixed-step scheduler.
- The session does not dequeue update batches or call `State::apply_update`.
- The renderer draws only the first supplied instance and uses a narrow GLTF path.
- The terminal display adapter is unfinished.
- No ROM catalog, manifest loading, remote session, or network transport is available.

See the [API reference](api.md#current-source-interfaces) for the current source boundaries.
