use flappy_client::{MAX_NOSE_DOWN, MAX_NOSE_UP, bird_tilt};
use flappy_core::Config;

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

#[test]
fn a_climbing_bird_points_its_nose_up() {
    let config = Config::default();

    assert!(close(bird_tilt(config.flap_velocity, &config), MAX_NOSE_UP));
}

#[test]
fn a_bird_at_full_fall_speed_dives_nose_down() {
    let config = Config::default();

    assert!(close(
        bird_tilt(config.max_fall_speed, &config),
        MAX_NOSE_DOWN
    ));
}

#[test]
fn the_nose_dips_gradually_as_the_bird_falls_faster() {
    let config = Config::default();
    let halfway = f32::midpoint(MAX_NOSE_UP, MAX_NOSE_DOWN);

    assert!(close(
        bird_tilt(config.max_fall_speed / 2.0, &config),
        halfway
    ));
}

#[test]
fn the_tilt_never_goes_past_its_limits() {
    let config = Config::default();

    assert!(close(bird_tilt(-100.0, &config), MAX_NOSE_UP));
    assert!(close(bird_tilt(100.0, &config), MAX_NOSE_DOWN));
}

#[test]
fn nose_up_turns_against_the_clock_and_nose_down_with_it_on_a_y_down_screen() {
    let config = Config::default();

    assert!(bird_tilt(config.flap_velocity, &config) < 0.0);
    assert!(bird_tilt(config.max_fall_speed, &config) > 0.0);
}
