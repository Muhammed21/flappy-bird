use crate::random;

/// How pipes are laid out along the course, in course coordinates (distance from the start).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CourseConfig {
    pub first_pipe_x: f32,
    /// From one pipe's left edge to the next one's.
    pub spacing: f32,
    pub pipe_width: f32,
    pub gap_height: f32,
    pub gap_center_min: f32,
    pub gap_center_max: f32,
}

/// The opening a pipe leaves for the bird, from `top` down to `bottom`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap {
    pub top: f32,
    pub bottom: f32,
}

/// Every pipe of one game, derived from its seed: pipe `n` is computed, never stored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Course {
    config: CourseConfig,
    seed: u64,
}

impl Course {
    #[must_use]
    pub fn new(config: CourseConfig, seed: u64) -> Self {
        Self { config, seed }
    }

    #[must_use]
    pub fn config(&self) -> CourseConfig {
        self.config
    }

    /// Left edge of pipe `index`.
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "exact up to 16 million pipes")]
    pub fn pipe_x(&self, index: u32) -> f32 {
        self.config.first_pipe_x + index as f32 * self.config.spacing
    }

    #[must_use]
    pub fn gap(&self, index: u32) -> Gap {
        let range = self.config.gap_center_max - self.config.gap_center_min;
        let centre = self.config.gap_center_min
            + random::unit(random::mix(self.seed, u64::from(index))) * range;
        let half = self.config.gap_height / 2.0;
        Gap {
            top: centre - half,
            bottom: centre + half,
        }
    }
}
