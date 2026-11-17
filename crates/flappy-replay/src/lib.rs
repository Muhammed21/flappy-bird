//! A game's replay: its seed, the ticks on which the player flapped, and its length.
//!
//! A score is never trusted as stored: [`Replay::verify`] plays the game again under the
//! current rules and returns the score it reaches. A replay edited by hand is refused or
//! scores what its inputs actually earn.

use flappy_core::{Config, Input, Phase, RULES_VERSION, World};

/// One hour at 60 ticks per second: longer replays are refused before being simulated.
pub const MAX_TICKS: u32 = 60 * 60 * 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayError {
    OtherRules { found: u32 },
    BadLength,
    DoesNotStartWithAFlap,
    FlapsOutOfOrder,
    FlapAfterTheEnd,
    Unfinished,
    GoesOnAfterTheEnd,
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OtherRules { found } => write!(
                f,
                "replay made under rules {found}, these are rules {RULES_VERSION}"
            ),
            Self::BadLength => write!(f, "replay length outside 1..={MAX_TICKS} ticks"),
            Self::DoesNotStartWithAFlap => write!(f, "replay does not start with a flap"),
            Self::FlapsOutOfOrder => write!(f, "replay flaps are not in strictly increasing order"),
            Self::FlapAfterTheEnd => write!(f, "replay has a flap after its last tick"),
            Self::Unfinished => write!(f, "replay stops before the game ends"),
            Self::GoesOnAfterTheEnd => write!(f, "replay goes on after the game ended"),
        }
    }
}

impl std::error::Error for ReplayError {}

/// What a replay proves, once played again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verified {
    pub score: u32,
}

/// A well-formed replay: built only through [`Replay::new`], which checks its shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replay {
    rules_version: u32,
    seed: u64,
    flaps: Vec<u32>,
    ticks: u32,
}

impl Replay {
    /// Tick 0 is the flap that starts the game; `ticks` counts every tick up to the crash.
    ///
    /// # Errors
    /// When the replay was made under other rules or isn't shaped like a game.
    pub fn new(
        rules_version: u32,
        seed: u64,
        flaps: Vec<u32>,
        ticks: u32,
    ) -> Result<Self, ReplayError> {
        if rules_version != RULES_VERSION {
            return Err(ReplayError::OtherRules {
                found: rules_version,
            });
        }
        if ticks == 0 || ticks > MAX_TICKS {
            return Err(ReplayError::BadLength);
        }
        if flaps.first() != Some(&0) {
            return Err(ReplayError::DoesNotStartWithAFlap);
        }
        if flaps.windows(2).any(|pair| pair.first() >= pair.get(1)) {
            return Err(ReplayError::FlapsOutOfOrder);
        }
        if flaps.last().is_some_and(|last| *last >= ticks) {
            return Err(ReplayError::FlapAfterTheEnd);
        }
        Ok(Self {
            rules_version,
            seed,
            flaps,
            ticks,
        })
    }

    #[must_use]
    pub fn rules_version(&self) -> u32 {
        self.rules_version
    }

    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub fn flaps(&self) -> &[u32] {
        &self.flaps
    }

    #[must_use]
    pub fn ticks(&self) -> u32 {
        self.ticks
    }

    /// Plays the game again and returns the score it reaches.
    ///
    /// # Errors
    /// When the game doesn't end exactly on the replay's last tick.
    pub fn verify(&self) -> Result<Verified, ReplayError> {
        let mut flaps = self.flaps.iter().peekable();
        let mut world = World::new(Config::default(), self.seed);
        for tick in 0..self.ticks {
            if world.phase() == Phase::Over {
                return Err(ReplayError::GoesOnAfterTheEnd);
            }
            let flap = flaps.next_if_eq(&&tick).is_some();
            world = world.step(if flap { Input::Flap } else { Input::Idle });
        }
        if world.phase() != Phase::Over {
            return Err(ReplayError::Unfinished);
        }
        Ok(Verified {
            score: world.score(),
        })
    }
}

/// Builds a replay while a game is played: feed it every input given to `World::step`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorder {
    seed: u64,
    flaps: Vec<u32>,
    ticks: u32,
}

impl Recorder {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            flaps: Vec::new(),
            ticks: 0,
        }
    }

    /// Takes the recorder by value so that recording never copies the flaps.
    #[must_use]
    pub fn record(self, input: Input) -> Self {
        let started = !self.flaps.is_empty();
        match (started, input) {
            (false, Input::Idle) => self,
            (_, Input::Flap) => {
                let mut flaps = self.flaps;
                flaps.push(self.ticks);
                Self {
                    flaps,
                    ticks: self.ticks + 1,
                    seed: self.seed,
                }
            }
            (true, Input::Idle) => Self {
                ticks: self.ticks + 1,
                ..self
            },
        }
    }

    /// # Errors
    /// When nothing was played yet.
    pub fn finish(&self) -> Result<Replay, ReplayError> {
        Replay::new(RULES_VERSION, self.seed, self.flaps.clone(), self.ticks)
    }
}
