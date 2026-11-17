//! The game's rules: physics, pipes, collisions and score.
//!
//! Pure and deterministic: no clock, no window, no file, no global randomness.
//! The same seed and the same inputs always give the same game, which is what
//! replays and score validation rely on.
//!
//! Units: logical pixels (y points down) and ticks of a fixed timestep.

mod course;
mod random;
mod world;

pub use course::{Course, CourseConfig, Gap};
pub use world::{Bird, Config, Input, Phase, Pipe, World};
