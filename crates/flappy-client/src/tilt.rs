use flappy_core::Config;

/// The bird's angle in radians while it climbs (negative: nose up).
pub const MAX_NOSE_UP: f32 = -0.4;
/// The bird's angle in radians at full fall speed (nose down).
pub const MAX_NOSE_DOWN: f32 = 1.4;

/// The bird's angle for its vertical speed: nose up while climbing, dipping as it falls faster.
#[must_use]
pub fn bird_tilt(velocity: f32, config: &Config) -> f32 {
    let fall = (velocity / config.max_fall_speed).clamp(0.0, 1.0);
    MAX_NOSE_UP + (MAX_NOSE_DOWN - MAX_NOSE_UP) * fall
}
