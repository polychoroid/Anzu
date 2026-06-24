Anzu Engine — Minimal Browser Scaffold
=====================================

This folder contains a minimal browser-only scaffold for Milestone 1.

What is included
- `src/platform_browser.rs` — tiny adapter that looks up the canvas element.
- `src/lib.rs` — wasm-bindgen start entry that attaches to the canvas and logs startup.
- `static/index.html` — simple HTML that loads the generated `pkg/` bundle.

Build & Run (quick)

1. Build the wasm package (recommended: `wasm-pack`):

```bash
wasm-pack build --target web --out-dir static/pkg
```

2. Serve the `static/` directory with a simple HTTP server and open `static/index.html` in the browser:

```bash
python3 -m http.server --directory static 8000
# then open http://localhost:8000/index.html
```

Alternative: use `cargo build --target wasm32-unknown-unknown` followed by `wasm-bindgen` CLI to generate `pkg/`.

Notes
- This scaffold intentionally avoids rendering or wgpu initialization — it is a minimal platform adapter so you can follow along.
- After you confirm this layout, I can add a small rotating-triangle renderer in Rust that uses `wgpu` and demonstrates Milestone 1 end-to-end.

Trusted references selected
Rendering and GPU standards
GPUWeb spec repository: https://github.com/gpuweb/gpuweb
WGPU project and examples: https://github.com/gfx-rs/wgpu and https://github.com/gfx-rs/wgpu/tree/trunk/examples
Learn WGPU tutorial: https://sotrh.github.io/learn-wgpu/
WebGPU Fundamentals: https://webgpufundamentals.org/
Engine architecture patterns
Bevy (data-oriented and modular patterns): https://bevy.org/learn/quick-start/introduction/ and https://github.com/bevyengine/bevy
Godot (mature scene/resource lifecycle and production structure): https://github.com/godotengine/godot
Simulation and netcode fundamentals
Fix Your Timestep: https://gafferongames.com/post/fix_your_timestep/
Snapshot Interpolation: https://gafferongames.com/post/snapshot_interpolation/
Networking stack references in Rust
Tokio runtime: https://github.com/tokio-rs/tokio
Quinn QUIC transport: https://github.com/quinn-rs/quinn
Security baselines
OWASP Authentication Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html
OWASP Authorization Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html

