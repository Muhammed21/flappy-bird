use flappy_core::{Config, Input, Phase, World};
use flappy_replay::{Recorder, Replay};

/// The game runs at a fixed 60 ticks per second, whatever the display's refresh rate.
pub const TICK_SECONDS: f64 = 1.0 / 60.0;
/// After a long pause (a hidden browser tab, a debugger), catch up a quarter second at most.
pub const MAX_TICKS_PER_FRAME: u32 = 15;
/// Half a second after a crash during which flaps are ignored, so mashing doesn't restart.
pub const RESTART_DELAY_TICKS: u32 = 30;

/// Rounding slack so that a frame of exactly `n` ticks runs `n` ticks.
const TICK_EPSILON: f64 = 1e-9;

/// One sitting at the game: the current world, the one a tick before (to interpolate), the
/// time not yet spent on a tick, and a flap waiting for the next tick.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    config: Config,
    seed: u64,
    world: World,
    previous: World,
    accumulated: f64,
    pending_flap: bool,
    ticks_over: u32,
    recorder: Recorder,
}

impl Session {
    #[must_use]
    pub fn new(config: Config, seed: u64) -> Self {
        let world = World::new(config, seed);
        Self {
            config,
            seed,
            previous: world.clone(),
            world,
            accumulated: 0.0,
            pending_flap: false,
            ticks_over: 0,
            recorder: Recorder::new(seed),
        }
    }

    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

    #[must_use]
    pub fn previous(&self) -> &World {
        &self.previous
    }

    /// The finished game's replay, once it is over.
    #[must_use]
    pub fn replay(&self) -> Option<Replay> {
        match self.world.phase() {
            Phase::Over => self.recorder.finish().ok(),
            Phase::Ready | Phase::Playing => None,
        }
    }

    /// How far the clock is between `previous` and `world`, from 0 to 1.
    #[must_use]
    #[expect(clippy::cast_possible_truncation, reason = "a value between 0 and 1")]
    pub fn alpha(&self) -> f32 {
        (self.accumulated / TICK_SECONDS) as f32
    }

    /// The session after a frame that lasted `seconds`, during which the player flapped or not.
    /// Takes the session by value so that a frame never copies the recording.
    #[must_use]
    pub fn frame(self, seconds: f64, flap: bool) -> Self {
        let seconds = if seconds.is_finite() {
            seconds.max(0.0)
        } else {
            0.0
        };
        let available = self.accumulated + seconds;
        let due = (available / TICK_SECONDS + TICK_EPSILON).floor();
        let ticks = whole_ticks(due.min(f64::from(MAX_TICKS_PER_FRAME)));
        // Past the cap, whole ticks are dropped and only the fraction of the next one is kept.
        let accumulated = (available - f64::from(ticks) * TICK_SECONDS).max(0.0) % TICK_SECONDS;
        let waiting = Self {
            accumulated,
            pending_flap: self.pending_flap || flap,
            ..self
        };
        (0..ticks).fold(waiting, |session, _| session.tick())
    }

    fn tick(self) -> Self {
        let flap = self.pending_flap;
        match self.world.phase() {
            Phase::Over if flap && self.ticks_over >= RESTART_DELAY_TICKS => {
                Self::new(self.config, self.seed.wrapping_add(1))
            }
            Phase::Over => Self {
                previous: self.world.clone(),
                pending_flap: false,
                ticks_over: self.ticks_over + 1,
                ..self
            },
            Phase::Ready | Phase::Playing => {
                let input = if flap { Input::Flap } else { Input::Idle };
                Self {
                    previous: self.world.clone(),
                    world: self.world.step(input),
                    pending_flap: false,
                    recorder: self.recorder.record(input),
                    ..self
                }
            }
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "callers pass a whole number between 0 and MAX_TICKS_PER_FRAME"
)]
fn whole_ticks(due: f64) -> u32 {
    due as u32
}
