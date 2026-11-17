//! Inputs for the benchmarks in `benches/`: long, realistic games played by a simple pilot.

use flappy_core::{Config, Input, Phase, World};
use flappy_replay::{Recorder, Replay, ReplayError};

/// Aims for the bottom of the next gap: good enough to pass pipes for a long time.
#[must_use]
pub fn autopilot(world: &World) -> Input {
    let config = Config::default();
    let bird = world.bird();
    let target = world
        .pipes()
        .find(|pipe| pipe.x + pipe.width >= config.bird_x - config.bird_radius)
        .map_or(config.ground_y - 100.0, |pipe| pipe.gap.bottom - 30.0);
    if world.phase() == Phase::Ready || (bird.y > target && bird.velocity >= 0.0) {
        Input::Flap
    } else {
        Input::Idle
    }
}

/// A game flown by the autopilot for `ticks` ticks, then left to crash.
///
/// # Errors
/// When the game never ends, which the autopilot letting go prevents.
pub fn long_game(seed: u64, ticks: u32) -> Result<Replay, ReplayError> {
    let start = (
        World::new(Config::default(), seed),
        Recorder::new(seed),
        0_u32,
    );
    let (_, recorder, _) = std::iter::successors(Some(start), |(world, recorder, tick)| {
        (world.phase() != Phase::Over).then(|| {
            let input = if *tick < ticks {
                autopilot(world)
            } else {
                Input::Idle
            };
            (world.step(input), recorder.clone().record(input), tick + 1)
        })
    })
    .last()
    .unwrap_or((World::new(Config::default(), seed), Recorder::new(seed), 0));
    recorder.finish()
}
