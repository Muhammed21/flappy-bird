use flappy_core::{Config, Input, Phase, RULES_VERSION, World};
use flappy_replay::{Recorder, Replay};
use flappy_storage::{Best, MAX_SAVE_BYTES, SaveError, decode, encode};
use proptest::prelude::*;

#[expect(
    clippy::unwrap_used,
    reason = "test helper: a game played to its crash always records"
)]
/// A finished game: flap every `every` ticks until the bird crashes.
fn finished_game(seed: u64, every: u32) -> Replay {
    let start = (World::new(Config::default(), seed), Recorder::new(seed));
    let (_, recorder) = (0..20_000_u32).fold(start, |(world, recorder), tick| {
        if world.phase() == Phase::Over {
            return (world, recorder);
        }
        let input = if tick % every == 0 {
            Input::Flap
        } else {
            Input::Idle
        };
        (world.step(input), recorder.record(input))
    });
    recorder.finish().unwrap()
}

#[expect(
    clippy::unwrap_used,
    reason = "test helper: a game played to its crash always records"
)]
/// A game that scores, by aiming for the bottom of the next gap, then letting go.
fn scoring_game(seed: u64, pipes: u32) -> Replay {
    let config = Config::default();
    let pilot = |world: &World| {
        let target = world
            .pipes()
            .find(|pipe| pipe.x + pipe.width >= config.bird_x - config.bird_radius)
            .map_or(300.0, |pipe| pipe.gap.bottom - 30.0);
        let bird = world.bird();
        if world.phase() == Phase::Ready
            || (world.score() < pipes && bird.y > target && bird.velocity >= 0.0)
        {
            Input::Flap
        } else {
            Input::Idle
        }
    };
    let start = (World::new(config, seed), Recorder::new(seed));
    let (_, recorder) = (0..20_000_u32).fold(start, |(world, recorder), _| {
        if world.phase() == Phase::Over {
            return (world, recorder);
        }
        let input = pilot(&world);
        (world.step(input), recorder.record(input))
    });
    recorder.finish().unwrap()
}

#[test]
fn a_saved_game_reads_back_as_the_same_replay() {
    let replay = scoring_game(1, 3);

    assert_eq!(decode(&encode(&replay)), Ok(replay));
}

#[test]
fn a_save_is_small_json_storing_the_gaps_between_flaps() {
    let replay = Replay::new(RULES_VERSION, 9, vec![0, 30, 45], 80).unwrap();

    assert_eq!(
        encode(&replay),
        format!(
            r#"{{"format":1,"rules":{RULES_VERSION},"seed":9,"ticks":80,"flap_gaps":[0,30,15]}}"#
        )
    );
}

#[test]
fn text_that_is_not_a_save_is_refused() {
    assert_eq!(decode("not json"), Err(SaveError::Unreadable));
    assert_eq!(decode(r#"{"format":1}"#), Err(SaveError::Unreadable));
}

#[test]
fn unknown_fields_are_refused_rather_than_ignored() {
    let save = r#"{"format":1,"rules":1,"seed":9,"ticks":80,"flap_gaps":[0],"score":999}"#;

    assert_eq!(decode(save), Err(SaveError::Unreadable));
}

#[test]
fn a_save_in_another_format_is_refused() {
    let save = r#"{"format":2,"rules":1,"seed":9,"ticks":80,"flap_gaps":[0]}"#;

    assert_eq!(decode(save), Err(SaveError::OtherFormat { found: 2 }));
}

#[test]
fn an_oversized_save_is_refused_before_it_is_parsed() {
    let huge = " ".repeat(MAX_SAVE_BYTES + 1);

    assert_eq!(decode(&huge), Err(SaveError::TooLarge));
}

#[test]
fn a_save_whose_gaps_overflow_is_refused() {
    let save = format!(
        r#"{{"format":1,"rules":1,"seed":9,"ticks":80,"flap_gaps":[0,{max},{max}]}}"#,
        max = u32::MAX
    );

    assert_eq!(decode(&save), Err(SaveError::Unreadable));
}

#[test]
fn a_save_that_is_not_shaped_like_a_game_is_refused() {
    let save = r#"{"format":1,"rules":1,"seed":9,"ticks":80,"flap_gaps":[5]}"#;

    assert_eq!(
        decode(save),
        Err(SaveError::NotAGame(
            flappy_replay::ReplayError::DoesNotStartWithAFlap
        ))
    );
}

#[test]
fn with_nothing_saved_there_is_no_best_score() {
    assert_eq!(Best::load(None).score(), None);
}

#[test]
fn a_saved_best_game_gives_back_its_replayed_score() {
    let replay = scoring_game(1, 3);

    let best = Best::load(Some(&encode(&replay)));

    assert_eq!(best.score(), Some(3));
}

#[test]
fn a_tampered_save_is_ignored_rather_than_trusted() {
    let honest = encode(&scoring_game(1, 3));
    let tampered = honest.replacen("\"seed\":1", "\"seed\":2", 1);

    assert_ne!(tampered, honest);
    assert_eq!(Best::load(Some(&tampered)).score(), None);
}

#[test]
fn an_unfinished_save_is_ignored() {
    let save = r#"{"format":1,"rules":1,"seed":9,"ticks":3,"flap_gaps":[0]}"#;

    assert_eq!(Best::load(Some(save)).score(), None);
}

#[test]
fn a_better_game_becomes_the_best_and_must_be_saved() {
    let better = scoring_game(1, 3);

    let (best, to_save) = Best::load(None).offer(&better);

    assert_eq!(best.score(), Some(3));
    assert_eq!(to_save, Some(encode(&better)));
}

#[test]
fn a_game_that_does_not_beat_the_best_changes_nothing() {
    let (best, _) = Best::load(None).offer(&scoring_game(1, 3));

    let (after, to_save) = best.clone().offer(&scoring_game(2, 2));
    let (tied, tie_save) = best.clone().offer(&scoring_game(3, 3));

    assert_eq!((after.score(), to_save), (Some(3), None));
    assert_eq!((tied.score(), tie_save), (Some(3), None));
}

#[test]
fn an_offered_game_is_judged_by_replaying_it() {
    let unfinished = Replay::new(RULES_VERSION, 9, vec![0], 3).unwrap();

    let (best, to_save) = Best::load(None).offer(&unfinished);

    assert_eq!((best.score(), to_save), (None, None));
}

#[test]
fn a_first_finished_game_is_a_best_even_with_no_points() {
    let (best, to_save) = Best::load(None).offer(&finished_game(1, 50));

    assert_eq!(best.score(), Some(0));
    assert!(to_save.is_some());
}

proptest! {
    #[test]
    fn any_finished_game_survives_a_save_and_load(seed: u64, every in 5_u32..60) {
        let replay = finished_game(seed, every);

        prop_assert_eq!(decode(&encode(&replay)), Ok(replay));
    }

    #[test]
    fn decoding_arbitrary_text_never_panics(text in ".{0,300}") {
        let _ = decode(&text);
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "a native test may time itself")]
fn the_largest_allowed_save_loads_in_linear_time() {
    let gaps = vec!["1"; 200_000].join(",");
    let save =
        format!(r#"{{"format":1,"rules":1,"seed":9,"ticks":200001,"flap_gaps":[0,{gaps}]}}"#);
    assert!(save.len() < MAX_SAVE_BYTES);

    let started = std::time::Instant::now();
    let replay = decode(&save).unwrap();

    assert_eq!(replay.flaps().len(), 200_001);
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
}

#[test]
fn a_save_of_exactly_the_maximum_size_is_still_parsed() {
    assert_eq!(
        decode(&" ".repeat(MAX_SAVE_BYTES)),
        Err(SaveError::Unreadable)
    );
}

#[test]
fn every_refusal_explains_itself_in_its_own_words() {
    let errors = [
        SaveError::TooLarge,
        SaveError::Unreadable,
        SaveError::OtherFormat { found: 2 },
        SaveError::NotAGame(flappy_replay::ReplayError::Unfinished),
    ];

    let messages: std::collections::BTreeSet<String> =
        errors.iter().map(ToString::to_string).collect();

    assert_eq!(messages.len(), errors.len());
    assert!(messages.iter().all(|message| !message.is_empty()));
}
