//! The save in the browser's `localStorage`, for the wasm build.
//!
//! Calling JavaScript from wasm is an FFI call, so this is the one crate that allows `unsafe`
//! (`cargo xtask unsafe` checks that no other crate does). The JS half is `web/storage.js`.
//! Every pointer handed to JS describes a live Rust slice for the whole call, and JS copies
//! what it reads before returning; it writes only into the buffer Rust sized for it.
//! Native builds compile this crate empty.

#![allow(unsafe_code)]

#[cfg(target_arch = "wasm32")]
mod bridge;

#[cfg(target_arch = "wasm32")]
pub use bridge::WebStore;
