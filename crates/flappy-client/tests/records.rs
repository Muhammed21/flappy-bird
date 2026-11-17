use std::cell::RefCell;

use flappy_client::{Records, Session, TICK_SECONDS};
use flappy_core::{Config, Phase};
use flappy_storage::{Best, Store, encode};

/// A store in memory that counts its writes, and can refuse them.
#[derive(Default)]
struct Memory {
    text: RefCell<Option<String>>,
    writes: RefCell<u32>,
    refuses: bool,
}

impl Store for Memory {
    fn read(&self) -> Option<String> {
        self.text.borrow().clone()
    }

    fn write(&self, text: &str) -> std::io::Result<()> {
        *self.writes.borrow_mut() += 1;
        if self.refuses {
            return Err(std::io::Error::other("full"));
        }
        *self.text.borrow_mut() = Some(text.to_owned());
        Ok(())
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test helper: flapping on a rhythm always ends in a crash"
)]
/// A session played to its crash, flapping every `every` frames.
fn crashed(seed: u64, every: usize) -> Session {
    std::iter::successors(
        Some((Session::new(Config::default(), seed), 0)),
        |(session, frame)| {
            Some((
                session.clone().frame(TICK_SECONDS, frame % every == 0),
                frame + 1,
            ))
        },
    )
    .take(5_000)
    .map(|(session, _)| session)
    .find(|session| session.world().phase() == Phase::Over)
    .unwrap()
}

#[test]
fn with_an_empty_store_there_is_no_best_score() {
    assert_eq!(Records::load(&Memory::default()).best_score(), None);
}

#[test]
fn the_best_score_is_read_back_from_the_store() {
    let replay = crashed(1, 20).replay().unwrap();
    let store = Memory::default();
    *store.text.borrow_mut() = Some(encode(&replay));

    let expected = Best::load(Some(&encode(&replay))).score();
    assert_eq!(Records::load(&store).best_score(), expected);
}

#[test]
fn a_finished_game_that_beats_the_best_is_saved() {
    let store = Memory::default();
    let session = crashed(1, 20);

    let records = Records::load(&store).after_frame(&session, &store);

    assert_eq!(records.best_score(), Some(session.world().score()));
    assert_eq!(store.read(), session.replay().map(|replay| encode(&replay)));
}

#[test]
fn a_finished_game_is_judged_once_however_many_frames_show_it() {
    let store = Memory::default();
    let session = crashed(1, 20);

    let _ = Records::load(&store)
        .after_frame(&session, &store)
        .after_frame(&session, &store)
        .after_frame(&session, &store);

    assert_eq!(*store.writes.borrow(), 1);
}

#[test]
fn a_game_in_progress_is_not_judged() {
    let store = Memory::default();
    let playing = Session::new(Config::default(), 1).frame(TICK_SECONDS, true);

    let records = Records::load(&store).after_frame(&playing, &store);

    assert_eq!((records.best_score(), *store.writes.borrow()), (None, 0));
}

#[test]
fn a_game_that_does_not_beat_the_best_is_not_saved() {
    let store = Memory::default();
    let first = crashed(1, 20);
    let records = Records::load(&store).after_frame(&first, &store);

    let _ = records.after_frame(&crashed(2, 20), &store);

    assert_eq!(*store.writes.borrow(), 1);
}

#[test]
fn a_store_that_refuses_to_write_still_keeps_the_best_for_this_session() {
    let store = Memory {
        refuses: true,
        ..Memory::default()
    };
    let session = crashed(1, 20);

    let records = Records::load(&store).after_frame(&session, &store);

    assert_eq!(records.best_score(), Some(session.world().score()));
}
