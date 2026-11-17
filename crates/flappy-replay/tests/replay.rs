mod support;

use flappy_core::{Input, RULES_VERSION};
use flappy_replay::{MAX_TICKS, Recorder, Replay, ReplayError, Verified};
use proptest::prelude::*;
use support::{SEED, autopilot, play};

fn replay(flaps: Vec<u32>, ticks: u32) -> Result<Replay, ReplayError> {
    Replay::new(RULES_VERSION, SEED, flaps, ticks)
}

#[test]
fn an_honest_game_replays_to_the_score_it_reached() {
    let pilot_for_five_pipes = |world: &flappy_core::World| {
        if world.score() < 5 {
            autopilot(world)
        } else {
            Input::Idle
        }
    };
    let (world, replay) = play(SEED, pilot_for_five_pipes, 20_000);

    assert_eq!(world.score(), 5);
    assert_eq!(
        replay.unwrap().verify(),
        Ok(Verified {
            score: world.score()
        })
    );
}

#[test]
fn a_replay_records_nothing_before_the_first_flap() {
    let recorder = Recorder::new(SEED)
        .record(Input::Idle)
        .record(Input::Idle)
        .record(Input::Flap);

    assert_eq!(recorder.finish(), replay(vec![0], 1));
}

#[test]
fn a_replay_must_start_with_the_flap_that_starts_the_game() {
    assert_eq!(
        replay(vec![3, 10], 40),
        Err(ReplayError::DoesNotStartWithAFlap)
    );
    assert_eq!(replay(vec![], 40), Err(ReplayError::DoesNotStartWithAFlap));
}

#[test]
fn flaps_must_be_in_order_and_on_distinct_ticks() {
    assert_eq!(replay(vec![0, 9, 5], 40), Err(ReplayError::FlapsOutOfOrder));
    assert_eq!(replay(vec![0, 5, 5], 40), Err(ReplayError::FlapsOutOfOrder));
}

#[test]
fn a_flap_must_happen_during_the_game() {
    assert_eq!(replay(vec![0, 40], 40), Err(ReplayError::FlapAfterTheEnd));
}

#[test]
fn a_replay_has_a_bounded_length() {
    assert_eq!(replay(vec![0], 0), Err(ReplayError::BadLength));
    assert_eq!(replay(vec![0], MAX_TICKS + 1), Err(ReplayError::BadLength));
    assert!(replay(vec![0], MAX_TICKS).is_ok());
}

#[test]
fn a_replay_made_under_other_rules_is_refused() {
    assert_eq!(
        Replay::new(RULES_VERSION + 1, SEED, vec![0], 10),
        Err(ReplayError::OtherRules {
            found: RULES_VERSION + 1
        })
    );
}

#[test]
fn a_replay_that_stops_before_the_game_ends_is_unfinished() {
    let replay = replay(vec![0], 5).unwrap();

    assert_eq!(replay.verify(), Err(ReplayError::Unfinished));
}

#[test]
fn a_replay_that_goes_on_after_the_game_ended_is_refused() {
    let (_, honest) = play(SEED, |_| Input::Idle, 1000);
    let honest = honest.unwrap();
    let padded = replay(honest.flaps().to_vec(), honest.ticks() + 1).unwrap();

    assert!(honest.verify().is_ok());
    assert_eq!(padded.verify(), Err(ReplayError::GoesOnAfterTheEnd));
}

#[test]
fn a_replay_keeps_its_seed_flaps_and_length() {
    let replay = Replay::new(RULES_VERSION, 9, vec![0, 4], 7).unwrap();

    assert_eq!(
        (
            replay.rules_version(),
            replay.seed(),
            replay.flaps(),
            replay.ticks()
        ),
        (RULES_VERSION, 9, &[0, 4][..], 7)
    );
}

proptest! {
    #[test]
    fn any_finished_game_replays_to_its_own_score(seed: u64, height in 50_u16..390) {
        let pilot = |world: &flappy_core::World| {
            let bird = world.bird();
            if bird.y > f32::from(height) && bird.velocity > 0.0 { Input::Flap } else { Input::Idle }
        };
        let (world, replay) = play(seed, pilot, 20_000);

        prop_assert_eq!(replay.unwrap().verify(), Ok(Verified { score: world.score() }));
    }
}

#[test]
fn a_game_may_last_up_to_an_hour() {
    let an_hour = 60 * 60 * 60;

    assert!(replay(vec![0], an_hour).is_ok());
    assert_eq!(replay(vec![0], an_hour + 1), Err(ReplayError::BadLength));
}

#[test]
fn every_refusal_explains_itself_in_its_own_words() {
    let errors = [
        ReplayError::OtherRules { found: 2 },
        ReplayError::BadLength,
        ReplayError::DoesNotStartWithAFlap,
        ReplayError::FlapsOutOfOrder,
        ReplayError::FlapAfterTheEnd,
        ReplayError::Unfinished,
        ReplayError::GoesOnAfterTheEnd,
    ];

    let messages: std::collections::BTreeSet<String> =
        errors.iter().map(ToString::to_string).collect();

    assert_eq!(messages.len(), errors.len());
    assert!(messages.iter().all(|message| !message.is_empty()));
}
