use std::f32::consts::TAU;

/// The game's sounds are generated, not shipped: no audio file, nothing to license.
pub const SAMPLE_RATE: u32 = 44_100;
/// Fade-in so the tone doesn't start with a click.
const ATTACK_SECONDS: f32 = 0.005;

/// A sine sweep from `start_hz` to `end_hz` that fades out over `seconds`; `volume` from 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tone {
    pub start_hz: f32,
    pub end_hz: f32,
    pub seconds: f32,
    pub volume: f32,
}

impl Tone {
    #[must_use]
    pub fn samples(&self) -> Vec<i16> {
        let rate = sample_rate();
        let count = whole(self.seconds * rate);
        let attack = (ATTACK_SECONDS * rate).max(1.0);
        let length = position(count).max(1.0);
        (0..count)
            .scan(0.0_f32, |phase, index| {
                let at = position(index);
                let progress = at / length;
                let envelope = (at / attack).min(1.0) * (1.0 - progress);
                let sample = phase.sin() * envelope * self.volume.clamp(0.0, 1.0);
                let hz = self.start_hz + (self.end_hz - self.start_hz) * progress;
                *phase = (*phase + TAU * hz / rate) % TAU;
                Some(pcm(sample))
            })
            .collect()
    }
}

/// A mono 16-bit PCM WAV file holding `samples`.
#[must_use]
pub fn wav(samples: &[i16]) -> Vec<u8> {
    let data_bytes = u32::try_from(samples.len() * 2).unwrap_or(u32::MAX);
    let header = [
        &b"RIFF"[..],
        &(36 + data_bytes).to_le_bytes(),
        b"WAVEfmt ",
        &16_u32.to_le_bytes(),
        &1_u16.to_le_bytes(),
        &1_u16.to_le_bytes(),
        &SAMPLE_RATE.to_le_bytes(),
        &(SAMPLE_RATE * 2).to_le_bytes(),
        &2_u16.to_le_bytes(),
        &16_u16.to_le_bytes(),
        b"data",
        &data_bytes.to_le_bytes(),
    ];
    header
        .into_iter()
        .flatten()
        .copied()
        .chain(samples.iter().flat_map(|sample| sample.to_le_bytes()))
        .collect()
}

#[expect(clippy::cast_precision_loss, reason = "44100 is exact in f32")]
fn sample_rate() -> f32 {
    SAMPLE_RATE as f32
}

#[expect(
    clippy::cast_precision_loss,
    reason = "sounds last well under 16 million samples"
)]
fn position(index: usize) -> f32 {
    index as f32
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a short, positive sample count"
)]
fn whole(samples: f32) -> usize {
    samples.round().max(0.0) as usize
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "clamped to the i16 range first"
)]
fn pcm(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16
}
