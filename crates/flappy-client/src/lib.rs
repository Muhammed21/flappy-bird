//! The client's logic, without macroquad: fixed timestep, input buffering, restart and the
//! letterboxed view. `main.rs` only reads the keyboard and draws.

mod records;
mod session;
mod view;

pub use records::Records;
pub use session::{MAX_TICKS_PER_FRAME, RESTART_DELAY_TICKS, Session, TICK_SECONDS};
pub use view::{Area, VIEW_HEIGHT, VIEW_WIDTH, visible_area};
