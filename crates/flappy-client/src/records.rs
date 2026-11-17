use flappy_storage::{Best, Store};

use crate::Session;

/// The best score, kept in a [`Store`]. Each finished game is judged once, by replaying it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Records {
    best: Best,
    judged: Option<u64>,
}

impl Records {
    #[must_use]
    pub fn load(store: &dyn Store) -> Self {
        Self {
            best: Best::load(store.read().as_deref()),
            judged: None,
        }
    }

    #[must_use]
    pub fn best_score(&self) -> Option<u32> {
        self.best.score()
    }

    /// Judges the session's game once it is over. A new best is saved; when the store refuses
    /// it, the best still holds for this session.
    #[must_use]
    pub fn after_frame(self, session: &Session, store: &dyn Store) -> Self {
        let Some(replay) = session.replay() else {
            return self;
        };
        if self.judged == Some(replay.seed()) {
            return self;
        }
        let (best, to_save) = self.best.offer(&replay);
        if let Some(text) = to_save {
            let _ = store.write(&text);
        }
        Self {
            best,
            judged: Some(replay.seed()),
        }
    }
}
