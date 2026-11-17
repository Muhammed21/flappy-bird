use flappy_client::{MAX_TICKS_PER_FRAME, RESTART_DELAY_TICKS, Session, TICK_SECONDS};
use flappy_core::{Config, Input, Phase, World};

const SEED: u64 = 7;

fn session() -> Session {
    Session::new(Config::default(), SEED)
}

fn world() -> World {
    World::new(Config::default(), SEED)
}

fn ticks(count: u32) -> f64 {
    f64::from(count) * TICK_SECONDS
}

fn idle_frames(session: Session, count: usize) -> Session {
    (0..count).fold(session, |session, _| session.frame(TICK_SECONDS, false))
}

/// The session on the very tick its game ends.
fn game_over() -> Option<Session> {
    std::iter::successors(Some(session().frame(TICK_SECONDS, true)), |session| {
        Some(session.clone().frame(TICK_SECONDS, false))
    })
    .take(1000)
    .find(|session| session.world().phase() == Phase::Over)
}

#[test]
fn a_frame_shorter_than_a_tick_leaves_the_world_as_it_was() {
    let session = session().frame(TICK_SECONDS / 2.0, false);

    assert_eq!(session.world(), &world());
}

#[test]
fn a_flap_pressed_between_two_ticks_is_kept_for_the_next_tick() {
    let pressed = session().frame(TICK_SECONDS / 2.0, true);
    let ticked = pressed.frame(TICK_SECONDS / 2.0, false);

    assert_eq!(ticked.world(), &world().step(Input::Flap));
}

#[test]
fn a_flap_applies_to_the_first_tick_only_when_a_frame_runs_several() {
    let session = session().frame(ticks(3), true);

    let expected = world()
        .step(Input::Flap)
        .step(Input::Idle)
        .step(Input::Idle);
    assert_eq!(session.world(), &expected);
}

#[test]
fn the_previous_world_is_one_tick_behind_for_interpolation() {
    let session = session().frame(ticks(3), true);

    assert_eq!(
        session.previous(),
        &world().step(Input::Flap).step(Input::Idle)
    );
}

#[test]
fn alpha_is_how_far_the_clock_is_into_the_next_tick() {
    let session = session().frame(TICK_SECONDS * 1.5, false);

    assert!((session.alpha() - 0.5).abs() < 1e-4);
}

#[test]
fn a_long_pause_runs_at_most_a_capped_number_of_ticks() {
    let session = session().frame(TICK_SECONDS, true).frame(10.0, false);

    let expected = (0..MAX_TICKS_PER_FRAME).fold(world().step(Input::Flap), |world, _| {
        world.step(Input::Idle)
    });
    assert_eq!(session.world(), &expected);
}

#[test]
fn a_broken_frame_time_runs_no_tick() {
    let session = session()
        .frame(f64::NAN, true)
        .frame(-1.0, false)
        .frame(f64::INFINITY, false);

    assert_eq!(session.world(), &world());
}

#[test]
fn a_flap_right_after_game_over_is_ignored_so_mashing_does_not_restart() {
    let over = game_over().unwrap();

    let mashed = idle_frames(over.frame(TICK_SECONDS, true), RESTART_DELAY_TICKS as usize);

    assert_eq!(mashed.world().phase(), Phase::Over);
}

#[test]
fn a_flap_after_the_restart_delay_starts_a_new_game_on_a_new_course() {
    let over = idle_frames(game_over().unwrap(), RESTART_DELAY_TICKS as usize);

    let restarted = over.frame(TICK_SECONDS, true);

    assert_eq!(restarted.world(), &World::new(Config::default(), SEED + 1));
}

#[test]
fn time_beyond_the_catch_up_cap_is_dropped_rather_than_run_later() {
    let paused = session().frame(TICK_SECONDS, true).frame(10.0, false);

    let next = paused.clone().frame(0.0, false);

    assert_eq!(next.world(), paused.world());
}

#[test]
fn once_the_game_is_over_the_frozen_bird_does_not_wobble() {
    let over = game_over().unwrap();

    let later = over.frame(TICK_SECONDS, false);

    assert_eq!(later.previous(), later.world());
}

#[test]
fn a_game_in_progress_has_no_replay_yet() {
    assert_eq!(session().frame(TICK_SECONDS, true).replay(), None);
}

#[test]
fn a_finished_game_hands_over_a_replay_that_proves_its_score() {
    let waited = idle_frames(session(), 30);
    let over = std::iter::successors(Some(waited.frame(TICK_SECONDS, true)), |session| {
        Some(session.clone().frame(TICK_SECONDS, false))
    })
    .take(1000)
    .find(|session| session.world().phase() == Phase::Over)
    .unwrap();

    let replay = over.replay().unwrap();

    assert_eq!(replay.seed(), SEED);
    assert_eq!(
        replay.verify(),
        Ok(flappy_replay::Verified {
            score: over.world().score()
        })
    );
}

#[test]
fn a_new_game_starts_a_new_recording() {
    let over = idle_frames(game_over().unwrap(), RESTART_DELAY_TICKS as usize);

    let restarted = over.frame(TICK_SECONDS, true);

    assert_eq!(restarted.replay(), None);
}
