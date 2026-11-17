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
