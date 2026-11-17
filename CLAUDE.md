# Flappy Bird (Rust, macroquad, desktop + WASM)

Architecture inspired by horizon-rust/horizon. Plan: `plans/architecture.md`. Layers: `docs/layers.md`.

## Commands

- `cargo xtask ci`: fmt, clippy (desktop + wasm32), tests, layer rule, cargo-deny. Run before every commit.
- `cargo xtask wasm`: browser build into `dist/` (serve with `python3 -m http.server -d dist`).
- `cargo run -p flappy-client`: desktop game. `--profile fast` for an optimized, quick-to-link build.

## Rules that bite

- `flappy-core` is pure: no clock, I/O, threads or global RNG. `Instant::now` panics on wasm32.
- `unsafe` is forbidden; `unwrap`/`expect`/`panic!`/indexing are denied outside tests.
- Untrusted data (save files) is parsed and bounded, never unwrapped.
- `gl.js` is copied from the locked `miniquad` crate, not downloaded: no CDN, strict CSP in `web/index.html`.
- TDD: tests first, in `flappy-core` especially, which runs headless.
