mod support;

use flappy_core::{Bird, Config, Gap, Phase, Pipe, World};
use support::{SEED, after, hovering, idle, run};

fn xs(world: &World) -> Vec<f32> {
    world.pipes().map(|pipe| pipe.x).collect()
}

#[test]
fn pipes_wait_with_the_bird_until_the_first_flap() {
    let world = run(World::new(hovering(50.0), SEED), &idle(30));

    assert_eq!(xs(&world), vec![100.0]);
}

#[test]
fn pipes_scroll_left_at_the_scroll_speed_once_the_game_starts() {
    assert_eq!(xs(&after(hovering(50.0), 1)), vec![98.0]);
}

#[test]
fn the_next_pipe_appears_when_it_reaches_the_right_edge_of_the_view() {
    assert_eq!(xs(&after(hovering(50.0), 25)), vec![50.0, 100.0]);
}

#[test]
fn a_pipe_is_listed_until_its_right_edge_leaves_the_view() {
    assert_eq!(xs(&after(hovering(50.0), 55)), vec![-10.0, 40.0, 90.0]);
    assert_eq!(xs(&after(hovering(50.0), 56)), vec![38.0, 88.0]);
}

#[test]
fn listed_pipes_carry_their_width_and_gap() {
    let pipe = after(hovering(50.0), 1).pipes().next();

    assert_eq!(
        pipe,
        Some(Pipe {
            x: 98.0,
            width: 10.0,
            gap: Gap {
                top: 30.0,
                bottom: 70.0
            },
        })
    );
}

#[test]
fn a_new_game_starts_with_no_score() {
    assert_eq!(World::new(hovering(50.0), SEED).score(), 0);
}

#[test]
fn a_pipe_scores_on_the_exact_tick_its_right_edge_passes_the_bird() {
    assert_eq!(after(hovering(50.0), 45).score(), 0);
    assert_eq!(after(hovering(50.0), 46).score(), 1);
}

#[test]
fn every_pipe_flown_through_scores_one() {
    let world = after(hovering(50.0), 271);

    assert_eq!(world.phase(), Phase::Playing);
    assert_eq!(world.score(), 10);
}

#[test]
fn hitting_a_pipe_ends_the_game_on_the_exact_tick_of_contact() {
    assert_eq!(after(hovering(20.0), 37).phase(), Phase::Playing);
    assert_eq!(after(hovering(20.0), 38).phase(), Phase::Over);
}

#[test]
fn hitting_the_upper_pipe_ends_the_game_too() {
    assert_eq!(after(hovering(80.0), 38).phase(), Phase::Over);
}

#[test]
fn nothing_scrolls_and_the_score_stays_once_the_game_is_over() {
    let over = after(hovering(20.0), 38);

    let later = run(over.clone(), &idle(100));

    assert_eq!(xs(&later), xs(&over));
    assert_eq!(later.score(), over.score());
}

#[test]
fn grazing_the_corner_of_the_upper_pipe_hits_only_once_the_bird_reaches_it() {
    // Gap top at 46, bird at 50 with radius 5: the corner touches when it is 3 px ahead.
    assert_eq!(after(hovering(66.0), 38).phase(), Phase::Playing);
    assert_eq!(after(hovering(66.0), 39).phase(), Phase::Over);
}

#[test]
fn grazing_the_corner_of_the_lower_pipe_hits_only_once_the_bird_reaches_it() {
    // Gap bottom at 54, bird at 50 with radius 5: the corner touches when it is 3 px ahead.
    assert_eq!(after(hovering(34.0), 38).phase(), Phase::Playing);
    assert_eq!(after(hovering(34.0), 39).phase(), Phase::Over);
}

#[test]
fn a_bird_that_hits_a_pipe_stops_where_it_hit() {
    let climbing = Config {
        flap_velocity: -1.0,
        ..hovering(80.0)
    };

    let world = after(climbing, 38);

    assert_eq!(world.phase(), Phase::Over);
    assert_eq!(
        world.bird(),
        Bird {
            y: 12.0,
            velocity: 0.0
        }
    );
}
