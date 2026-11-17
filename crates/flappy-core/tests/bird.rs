mod support;

use flappy_core::{Bird, Input, Phase, World};
use support::{SEED, config, idle, run};

fn world() -> World {
    World::new(config(), SEED)
}

fn playing() -> World {
    world().step(Input::Flap)
}

#[test]
fn a_new_game_waits_for_the_first_flap_with_the_bird_at_its_start_height() {
    let world = world();

    assert_eq!(world.phase(), Phase::Ready);
    assert_eq!(
        world.bird(),
        Bird {
            y: 50.0,
            velocity: 0.0
        }
    );
}

#[test]
fn the_bird_hovers_while_the_game_waits() {
    let world = run(world(), &idle(30));

    assert_eq!(world.phase(), Phase::Ready);
    assert_eq!(
        world.bird(),
        Bird {
            y: 50.0,
            velocity: 0.0
        }
    );
}

#[test]
fn the_first_flap_starts_the_game_and_lifts_the_bird() {
    let world = world().step(Input::Flap);

    assert_eq!(world.phase(), Phase::Playing);
    assert_eq!(
        world.bird(),
        Bird {
            y: 45.0,
            velocity: -5.0
        }
    );
}

#[test]
fn gravity_pulls_the_bird_down_a_little_more_each_tick() {
    let world = run(playing(), &idle(2));

    assert_eq!(
        world.bird(),
        Bird {
            y: 38.0,
            velocity: -3.0
        }
    );
}

#[test]
fn the_bird_never_falls_faster_than_its_maximum_fall_speed() {
    let world = run(playing(), &idle(12));

    assert_eq!(
        world.bird(),
        Bird {
            y: 53.0,
            velocity: 3.0
        }
    );
}

#[test]
fn a_flap_replaces_the_fall_speed_instead_of_adding_to_it() {
    let falling = run(playing(), &idle(10));

    let world = falling.step(Input::Flap);

    assert_eq!(
        world.bird(),
        Bird {
            y: 42.0,
            velocity: -5.0
        }
    );
}

#[test]
fn the_bird_cannot_fly_above_the_top_of_the_screen() {
    let world = run(playing(), &[Input::Flap; 20]);

    assert_eq!(
        world.bird(),
        Bird {
            y: 5.0,
            velocity: 0.0
        }
    );
}

#[test]
fn touching_the_ground_ends_the_game_with_the_bird_resting_on_it() {
    let world = run(playing(), &idle(40));

    assert_eq!(world.phase(), Phase::Over);
    assert_eq!(
        world.bird(),
        Bird {
            y: 95.0,
            velocity: 0.0
        }
    );
}

#[test]
fn nothing_moves_once_the_game_is_over() {
    let over = run(playing(), &idle(40));

    let later = run(over.clone(), &[Input::Flap, Input::Idle, Input::Flap]);

    assert_eq!(later, over);
}

#[test]
fn the_game_ends_on_the_exact_tick_the_bird_touches_the_ground() {
    let just_above = run(playing(), &idle(25));
    let touching = just_above.step(Input::Idle);

    assert_eq!(just_above.phase(), Phase::Playing);
    assert_eq!(touching.phase(), Phase::Over);
    assert_eq!(
        touching.bird(),
        Bird {
            y: 95.0,
            velocity: 0.0
        }
    );
}

#[test]
fn reaching_the_top_of_the_screen_exactly_stops_the_climb() {
    let world = run(playing(), &[Input::Flap; 8]);

    assert_eq!(
        world.bird(),
        Bird {
            y: 5.0,
            velocity: 0.0
        }
    );
}
