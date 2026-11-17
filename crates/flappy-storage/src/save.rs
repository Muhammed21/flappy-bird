use flappy_replay::{Replay, ReplayError};
use serde::{Deserialize, Serialize};

/// Bumped when the save's layout changes.
pub const FORMAT: u32 = 1;
/// An hour-long game flapping every other tick stays well under this.
pub const MAX_SAVE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveError {
    TooLarge,
    Unreadable,
    OtherFormat { found: u32 },
    NotAGame(ReplayError),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge => write!(f, "save larger than {MAX_SAVE_BYTES} bytes"),
            Self::Unreadable => write!(f, "save is not a readable save"),
            Self::OtherFormat { found } => {
                write!(
                    f,
                    "save in format {found}, this version reads format {FORMAT}"
                )
            }
            Self::NotAGame(error) => write!(f, "save is not a game: {error}"),
        }
    }
}

impl std::error::Error for SaveError {}

#[derive(Deserialize)]
struct Header {
    format: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    format: u32,
    rules: u32,
    seed: u64,
    ticks: u32,
    /// The first flap's tick, then the ticks between each flap and the one before.
    flap_gaps: Vec<u32>,
}

#[must_use]
pub fn encode(replay: &Replay) -> String {
    let flaps = replay.flaps();
    let gaps = flaps
        .first()
        .into_iter()
        .copied()
        .chain(
            flaps
                .iter()
                .zip(flaps.iter().skip(1))
                .map(|(before, after)| after - before),
        )
        .collect();
    let save = Save {
        format: FORMAT,
        rules: replay.rules_version(),
        seed: replay.seed(),
        ticks: replay.ticks(),
        flap_gaps: gaps,
    };
    serde_json::to_string(&save).unwrap_or_default()
}

/// # Errors
/// When the text is too large, isn't a save of this format, or isn't shaped like a game.
pub fn decode(text: &str) -> Result<Replay, SaveError> {
    if text.len() > MAX_SAVE_BYTES {
        return Err(SaveError::TooLarge);
    }
    let header: Header = serde_json::from_str(text).map_err(|_| SaveError::Unreadable)?;
    if header.format != FORMAT {
        return Err(SaveError::OtherFormat {
            found: header.format,
        });
    }
    let save: Save = serde_json::from_str(text).map_err(|_| SaveError::Unreadable)?;
    let (flaps, _) = save
        .flap_gaps
        .iter()
        .try_fold(
            (Vec::with_capacity(save.flap_gaps.len()), 0_u32),
            |(mut flaps, last), gap| {
                let tick = last.checked_add(*gap)?;
                flaps.push(tick);
                Some((flaps, tick))
            },
        )
        .ok_or(SaveError::Unreadable)?;
    Replay::new(save.rules, save.seed, flaps, save.ticks).map_err(SaveError::NotAGame)
}
