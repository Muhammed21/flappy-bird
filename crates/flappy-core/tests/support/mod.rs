#![allow(dead_code)]

use flappy_core::{Config, CourseConfig, Input, World};

pub const SEED: u64 = 7;

/// Simple numbers; the first pipe is far away so bird-only tests never meet one.
pub fn config() -> Config {
    Config {
        gravity: 1.0,
        flap_velocity: -5.0,
        max_fall_speed: 3.0,
        bird_radius: 5.0,
        start_y: 50.0,
        ground_y: 100.0,
        bird_x: 20.0,
        view_width: 100.0,
        scroll_speed: 2.0,
        course: CourseConfig {
            first_pipe_x: 1000.0,
            spacing: 50.0,
            pipe_width: 10.0,
            gap_height: 40.0,
            gap_center_min: 50.0,
            gap_center_max: 50.0,
        },
    }
}

/// A bird that keeps its start height forever (no gravity, a flap sets speed 0),
/// facing pipes from x = 100 whose gaps are centred on `gap_center`.
pub fn hovering(gap_center: f32) -> Config {
    Config {
        gravity: 0.0,
        flap_velocity: 0.0,
        course: CourseConfig {
            first_pipe_x: 100.0,
            gap_center_min: gap_center,
            gap_center_max: gap_center,
            ..config().course
        },
        ..config()
    }
}

pub fn run(world: World, inputs: &[Input]) -> World {
    inputs.iter().fold(world, |world, input| world.step(*input))
}

pub fn idle(ticks: usize) -> Vec<Input> {
    vec![Input::Idle; ticks]
}

/// A game `ticks` ticks after its first flap (the flap is tick 1).
pub fn after(config: Config, ticks: usize) -> World {
    run(World::new(config, SEED).step(Input::Flap), &idle(ticks - 1))
}
