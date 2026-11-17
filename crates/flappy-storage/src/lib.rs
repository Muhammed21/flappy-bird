//! The best game, saved as its replay.
//!
//! A save is untrusted input: its size is bounded before parsing, unknown fields are refused,
//! its shape is checked by [`Replay::new`], and its score is never read from it but recomputed
//! by [`Replay::verify`]. Anything that fails is treated as no save at all.

mod file;
mod save;

pub use file::{FileStore, data_dir};
pub use save::{FORMAT, MAX_SAVE_BYTES, SaveError, decode, encode};

/// Where the save lives: a file on desktop, the browser's local storage on the web.
pub trait Store {
    /// The saved text, or nothing when there is none or it can't be read.
    fn read(&self) -> Option<String>;

    /// # Errors
    /// When the save can't be written; the previous one stays as it was.
    fn write(&self, text: &str) -> std::io::Result<()>;
}

use flappy_replay::Replay;

/// The best finished game so far, with the score it replays to.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Best {
    game: Option<(Replay, u32)>,
}

impl Best {
    /// Reads a save; one that can't be read or doesn't replay to a finished game counts as none.
    #[must_use]
    pub fn load(save: Option<&str>) -> Self {
        let game = save.and_then(|text| decode(text).ok()).and_then(|replay| {
            let verified = replay.verify().ok()?;
            Some((replay, verified.score))
        });
        Self { game }
    }

    #[must_use]
    pub fn score(&self) -> Option<u32> {
        self.game.as_ref().map(|(_, score)| *score)
    }

    /// Judges a game by replaying it. When it beats the best, it becomes the best and the text
    /// to save is returned.
    #[must_use]
    pub fn offer(self, replay: &Replay) -> (Self, Option<String>) {
        let Ok(verified) = replay.verify() else {
            return (self, None);
        };
        if self.score().is_some_and(|best| verified.score <= best) {
            return (self, None);
        }
        let best = Self {
            game: Some((replay.clone(), verified.score)),
        };
        (best, Some(encode(replay)))
    }
}
