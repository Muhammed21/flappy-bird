# Flappy Bird (Rust, macroquad, desktop + WASM)

Architecture inspired by horizon-rust/horizon. Plan: `plans/architecture.md`. Layers: `docs/layers.md`.

## Commands

- `cargo xtask ci`: fmt, clippy (desktop + wasm32), tests, layer rule, unsafe guard, cargo-deny, wasm build. Run before every commit.
- `cargo xtask wasm`: browser build into `dist/` (serve with `python3 -m http.server -d dist`).
- `cargo run -p flappy-client`: desktop game. `--profile fast` for an optimized, quick-to-link build.
- `cargo bench -p flappy-bench`: a tick (~11 ns), verifying a 10-minute replay (~375 µs). Measure before optimizing.

## Rules that bite

- `flappy-core` is pure: no clock, I/O, threads or global RNG. `Instant::now` panics on wasm32.
- `unsafe` is denied everywhere except `flappy-web-storage` (the JS bridge; `cargo xtask unsafe`). `unwrap`/`expect`/`panic!`/indexing are denied outside tests.
- The save is untrusted: size-bounded, unknown fields refused, and its score never read but recomputed by replaying it (`flappy-storage`).
- Changing the rules or `Config::default` changes replays: bump `flappy_core::RULES_VERSION` (a golden test in `crates/flappy-core/tests/course.rs` fails to remind you).
- Desktop runs: set `FLAPPY_HOME` to a scratch folder so the real save is never touched.
- Sounds are generated (`Tone`, `wav`), not shipped. quad-snd's JS imports are allowed undefined one by one in `web/wasm-imports.txt` (`.cargo/config.toml`); never set `RUSTFLAGS` in CI, it would replace that flag.
- Text goes through `text_size`: free-form font sizes make macroquad rebuild its glyph texture (`glBindTexture` errors in the browser).
- An `extern "C"` block for wasm needs `#[link(wasm_import_module = "env")]`, or the wasm link fails (clippy doesn't link, the wasm build in `cargo xtask ci` does).
- `gl.js` is copied from the locked `miniquad` crate, not downloaded: no CDN, strict CSP in `web/index.html`.
- TDD: tests first, in `flappy-core` especially, which runs headless.
