use crate::course::{Course, CourseConfig, Gap};

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
    /// The bird's fixed horizontal position on screen; the pipes move, not the bird.
    pub bird_x: f32,
    pub view_width: f32,
    /// Distance the pipes move left each tick.
    pub scroll_speed: f32,
    pub course: CourseConfig,
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
            bird_x: 80.0,
            view_width: 288.0,
            scroll_speed: 2.0,
            course: CourseConfig {
                first_pipe_x: 400.0,
                spacing: 160.0,
                pipe_width: 52.0,
                gap_height: 110.0,
                gap_center_min: 130.0,
                gap_center_max: 300.0,
            },
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

/// A pipe as it stands on screen: `x` is its left edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pipe {
    pub x: f32,
    pub width: f32,
    pub gap: Gap,
}

impl Pipe {
    fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Whether a circle overlaps or touches the pipe above or below its gap.
    fn hits(&self, x: f32, y: f32, radius: f32) -> bool {
        let dx = x.clamp(self.x, self.right()) - x;
        let dy_top = (y - self.gap.top).max(0.0);
        let dy_bottom = (self.gap.bottom - y).max(0.0);
        dx * dx + dy_top * dy_top <= radius * radius
            || dx * dx + dy_bottom * dy_bottom <= radius * radius
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct World {
    config: Config,
    course: Course,
    phase: Phase,
    bird: Bird,
    /// Ticks the course has scrolled since the first flap.
    ticks: u32,
    /// Pipes passed, which is also the index of the next pipe to pass.
    score: u32,
}

impl World {
    #[must_use]
    pub fn new(config: Config, seed: u64) -> Self {
        Self {
            config,
            course: Course::new(config.course, seed),
            phase: Phase::Ready,
            bird: Bird {
                y: config.start_y,
                velocity: 0.0,
            },
            ticks: 0,
            score: 0,
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

    #[must_use]
    pub fn score(&self) -> u32 {
        self.score
    }

    /// How far the course has scrolled since the first flap.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "exact for the first 16 million ticks"
    )]
    pub fn distance(&self) -> f32 {
        self.ticks as f32 * self.config.scroll_speed
    }

    /// The pipes on screen, left to right. Allocates nothing.
    pub fn pipes(&self) -> impl Iterator<Item = Pipe> + '_ {
        let first = (0..self.score)
            .rev()
            .take_while(|index| self.pipe(*index).right() >= 0.0)
            .last()
            .unwrap_or(self.score);
        (first..)
            .map(|index| self.pipe(index))
            .take_while(|pipe| pipe.x <= self.config.view_width)
    }

    /// The world one tick later.
    #[must_use]
    pub fn step(&self, input: Input) -> Self {
        match (self.phase, input) {
            (Phase::Over, _) | (Phase::Ready, Input::Idle) => self.clone(),
            (Phase::Ready | Phase::Playing, _) => self.fly(input),
        }
    }

    fn pipe(&self, index: u32) -> Pipe {
        Pipe {
            x: self.course.pipe_x(index) - self.distance(),
            width: self.course.config().pipe_width,
            gap: self.course.gap(index),
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
        let scrolled = Self {
            ticks: self.ticks + 1,
            ..self.clone()
        };
        let scored = scrolled.passing_score();

        if y >= floor {
            return scored.with(
                Phase::Over,
                Bird {
                    y: floor,
                    velocity: 0.0,
                },
            );
        }
        let bird = if y <= ceiling {
            Bird {
                y: ceiling,
                velocity: 0.0,
            }
        } else {
            Bird { y, velocity }
        };
        if scored.hits_pipe(bird) {
            return scored.with(
                Phase::Over,
                Bird {
                    velocity: 0.0,
                    ..bird
                },
            );
        }
        scored.with(Phase::Playing, bird)
    }

    fn passing_score(&self) -> Self {
        let passed = self.pipe(self.score).right() < self.config.bird_x;
        Self {
            score: self.score + u32::from(passed),
            ..self.clone()
        }
    }

    fn hits_pipe(&self, bird: Bird) -> bool {
        self.pipes()
            .any(|pipe| pipe.hits(self.config.bird_x, bird.y, self.config.bird_radius))
    }

    fn with(&self, phase: Phase, bird: Bird) -> Self {
        Self {
            phase,
            bird,
            ..self.clone()
        }
    }
}
