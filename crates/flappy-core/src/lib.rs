//! The game's rules: physics, pipes, collisions and score.
//!
//! Pure and deterministic: no clock, no window, no file, no global randomness.
//! The same seed and the same inputs always give the same game, which is what
//! replays and score validation rely on.
//!
//! Units: logical pixels (y points down) and ticks of a fixed timestep.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Speed added each tick while falling, in px/tick².
    pub gravity: f32,
    /// Speed a flap sets, in px/tick (negative goes up).
    pub flap_velocity: f32,
    pub max_fall_speed: f32,
    pub bird_radius: f32,
    pub start_y: f32,
    /// Top of the ground: the bird touching it ends the game.
    pub ground_y: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gravity: 0.45,
            flap_velocity: -7.5,
            max_fall_speed: 9.0,
            bird_radius: 12.0,
            start_y: 220.0,
            ground_y: 400.0,
        }
    }
}

/// What the player does during one tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Idle,
    Flap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for the first flap.
    Ready,
    Playing,
    Over,
}

/// The bird's centre height and vertical speed (px/tick, positive falls).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bird {
    pub y: f32,
    pub velocity: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct World {
    config: Config,
    phase: Phase,
    bird: Bird,
}

impl World {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            phase: Phase::Ready,
            bird: Bird {
                y: config.start_y,
                velocity: 0.0,
            },
        }
    }

    #[must_use]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    #[must_use]
    pub fn bird(&self) -> Bird {
        self.bird
    }

    /// The world one tick later.
    #[must_use]
    pub fn step(&self, input: Input) -> Self {
        match (self.phase, input) {
            (Phase::Over, _) | (Phase::Ready, Input::Idle) => self.clone(),
            (Phase::Ready | Phase::Playing, _) => self.fly(input),
        }
    }

    fn fly(&self, input: Input) -> Self {
        let velocity = match input {
            Input::Flap => self.config.flap_velocity,
            Input::Idle => {
                (self.bird.velocity + self.config.gravity).min(self.config.max_fall_speed)
            }
        };
        let y = self.bird.y + velocity;
        let ceiling = self.config.bird_radius;
        let floor = self.config.ground_y - self.config.bird_radius;

        if y >= floor {
            return self.with(
                Phase::Over,
                Bird {
                    y: floor,
                    velocity: 0.0,
                },
            );
        }
        if y <= ceiling {
            return self.with(
                Phase::Playing,
                Bird {
                    y: ceiling,
                    velocity: 0.0,
                },
            );
        }
        self.with(Phase::Playing, Bird { y, velocity })
    }

    fn with(&self, phase: Phase, bird: Bird) -> Self {
        Self {
            config: self.config,
            phase,
            bird,
        }
    }
}
