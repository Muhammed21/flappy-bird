//! The client's logic, without macroquad: fixed timestep, input buffering, restart and the
//! letterboxed view. `main.rs` only reads the keyboard and draws.

mod records;
mod session;
mod sound;
mod text;
mod tilt;
mod view;

pub use records::Records;
pub use session::{Events, MAX_TICKS_PER_FRAME, RESTART_DELAY_TICKS, Session, TICK_SECONDS};
pub use sound::{SAMPLE_RATE, Tone, wav};
pub use text::{TextSize, text_size};
pub use tilt::{MAX_NOSE_DOWN, MAX_NOSE_UP, bird_tilt};
pub use view::{Area, VIEW_HEIGHT, VIEW_WIDTH, visible_area};
