#![allow(dead_code)]

use flappy_core::{Config, Input, Phase, World};
use flappy_replay::{Recorder, Replay, ReplayError};

pub const SEED: u64 = 42;

/// A simple pilot that aims for the bottom of the next gap: good enough to score.
pub fn autopilot(world: &World) -> Input {
    let config = Config::default();
    let bird = world.bird();
    let target = world
        .pipes()
        .find(|pipe| pipe.x + pipe.width >= config.bird_x - config.bird_radius)
        .map_or(config.ground_y - 100.0, |pipe| pipe.gap.bottom - 30.0);
    if bird.y > target && bird.velocity >= 0.0 {
        Input::Flap
    } else {
        Input::Idle
    }
}

/// Plays from the first flap until the game ends (or `max_ticks`), recording every input.
pub fn play(
    seed: u64,
    pilot: impl Fn(&World) -> Input,
    max_ticks: usize,
) -> (World, Result<Replay, ReplayError>) {
    let start = (World::new(Config::default(), seed), Recorder::new(seed));
    let (world, recorder) = (0..max_ticks).fold(start, |(world, recorder), tick| {
        if world.phase() == Phase::Over {
            return (world, recorder);
        }
        let input = if tick == 0 {
            Input::Flap
        } else {
            pilot(&world)
        };
        (world.step(input), recorder.record(input))
    });
    (world, recorder.finish())
}

/// Flaps on the given ticks, idles otherwise, for `ticks` ticks.
pub fn inputs(flaps: &[u32], ticks: u32) -> Vec<Input> {
    (0..ticks)
        .map(|tick| {
            if flaps.contains(&tick) {
                Input::Flap
            } else {
                Input::Idle
            }
        })
        .collect()
}
