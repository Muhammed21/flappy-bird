# Layers

Inspired by Horizon's engine / platform / Horizon split. A crate may depend only on crates of a
**lower** layer. `cargo xtask layers` checks it, and CI runs it.

| Layer | Crates | May depend on |
|---|---|---|
| core | `flappy-core` | nothing in the workspace |
| replay | `flappy-replay` | core |
| storage | `flappy-storage` | core, replay |
| platform | `flappy-web-storage` | core, replay, storage |
| app | `flappy-client` | every layer above |
| tool | `flappy-bench`, `xtask` | anything; no shipped crate depends on a tool |

Normal and build dependencies count; dev-dependencies don't, so tests may use anything.

When lower code needs something from above (time, randomness, input, storage), the lower layer
offers an extension point: a trait or a plain value passed in, naming no type from above.

`flappy-core` stays pure: no clock, no I/O, no global randomness, no threads. That keeps it
deterministic (replays and score validation depend on it) and runnable on `wasm32-unknown-unknown`.
`clippy.toml` bans the std calls that would break this.

A new crate gets a row here and in `LAYERS` (`xtask/src/layers.rs`), or `cargo xtask layers` fails.

## `unsafe`

Denied in every crate. Only `flappy-web-storage` allows it: calling JavaScript (`localStorage`)
from wasm is an FFI call. Every `unsafe` block there carries a `SAFETY:` comment
(`clippy::undocumented_unsafe_blocks`), and `cargo xtask unsafe` fails if any other crate turns
the lint off, in its sources or its `Cargo.toml`.
