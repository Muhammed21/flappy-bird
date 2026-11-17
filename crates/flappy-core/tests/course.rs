use flappy_core::{Course, CourseConfig, Gap};
use proptest::prelude::*;

fn config() -> CourseConfig {
    CourseConfig {
        first_pipe_x: 100.0,
        spacing: 50.0,
        pipe_width: 10.0,
        gap_height: 40.0,
        gap_center_min: 30.0,
        gap_center_max: 70.0,
    }
}

fn gaps(course: &Course, count: u32) -> Vec<Gap> {
    (0..count).map(|index| course.gap(index)).collect()
}

#[test]
fn pipes_start_at_the_first_pipe_and_follow_each_other_evenly() {
    let course = Course::new(config(), 1);

    let xs: Vec<f32> = (0..3).map(|index| course.pipe_x(index)).collect();

    assert_eq!(xs, vec![100.0, 150.0, 200.0]);
}

#[test]
fn a_fixed_gap_range_gives_every_pipe_the_same_gap() {
    let config = CourseConfig {
        gap_center_min: 50.0,
        gap_center_max: 50.0,
        ..config()
    };

    let course = Course::new(config, 1);

    assert_eq!(
        gaps(&course, 5),
        vec![
            Gap {
                top: 30.0,
                bottom: 70.0
            };
            5
        ]
    );
}

#[test]
fn the_same_seed_always_lays_out_the_same_course() {
    assert_eq!(
        gaps(&Course::new(config(), 42), 50),
        gaps(&Course::new(config(), 42), 50)
    );
}

#[test]
fn different_seeds_lay_out_different_courses() {
    assert_ne!(
        gaps(&Course::new(config(), 1), 10),
        gaps(&Course::new(config(), 2), 10)
    );
}

#[test]
fn the_gaps_of_one_course_vary_from_pipe_to_pipe() {
    let gaps = gaps(&Course::new(config(), 42), 10);

    assert!(gaps.windows(2).any(|pair| pair[0] != pair[1]));
}

proptest! {
    #[test]
    fn every_gap_has_the_configured_height_and_a_centre_within_range(seed: u64, index: u32) {
        let gap = Course::new(config(), seed).gap(index);
        let centre = f32::midpoint(gap.top, gap.bottom);

        prop_assert!((gap.bottom - gap.top - 40.0).abs() < 1e-3);
        prop_assert!((30.0..=70.0).contains(&centre));
    }
}

#[test]
fn a_seed_lays_out_the_same_course_in_every_version_so_old_replays_stay_valid() {
    let centres: Vec<f32> = gaps(&Course::new(config(), 42), 5)
        .iter()
        .map(|gap| f32::midpoint(gap.top, gap.bottom))
        .collect();

    assert_eq!(
        centres,
        vec![59.662_598, 36.396_416, 41.144_043, 43.767_624, 31.521_206]
    );
}

#[test]
fn a_scripted_game_ends_the_same_way_under_these_rules() {
    // If this fails, the rules changed: bump flappy_core::RULES_VERSION so that replays
    // recorded under the old rules are refused instead of misjudged, then update this test.
    use flappy_core::{Bird, Config, Input, Phase, RULES_VERSION, World};

    let config = Config::default();
    let pilot = |world: &World| {
        let next_gap = world
            .pipes()
            .find(|pipe| pipe.x + pipe.width >= config.bird_x - config.bird_radius)
            .map_or(300.0, |pipe| pipe.gap.bottom - 30.0);
        let bird = world.bird();
        let flap = world.score() < 3 && bird.y > next_gap && bird.velocity >= 0.0;
        if world.phase() == Phase::Ready || flap {
            Input::Flap
        } else {
            Input::Idle
        }
    };
    let (world, ticks) = (0..5_000_u32)
        .try_fold((World::new(config, 42), 0_u32), |(world, ticks), _| {
            if world.phase() == Phase::Over {
                return Err((world, ticks));
            }
            Ok((world.step(pilot(&world)), ticks + 1))
        })
        .unwrap_or_else(|ended| ended);

    assert_eq!(RULES_VERSION, 1);
    assert_eq!(world.phase(), Phase::Over);
    assert_eq!(
        (world.score(), ticks, world.bird()),
        (
            3,
            387,
            Bird {
                y: 388.0,
                velocity: 0.0
            }
        )
    );
}
