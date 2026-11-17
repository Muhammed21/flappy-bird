use flappy_client::{SAMPLE_RATE, Tone, wav};

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let field = bytes.get(at..at + 4)?;
    Some(u32::from_le_bytes(field.try_into().ok()?))
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let field = bytes.get(at..at + 2)?;
    Some(u16::from_le_bytes(field.try_into().ok()?))
}

fn chirp() -> Tone {
    Tone {
        start_hz: 400.0,
        end_hz: 800.0,
        seconds: 0.1,
        volume: 0.5,
    }
}

#[test]
fn a_tone_lasts_its_duration() {
    assert_eq!(chirp().samples().len(), 4410);
}

#[test]
fn a_tone_starts_and_ends_in_silence_so_it_does_not_click() {
    let samples = chirp().samples();

    assert_eq!(samples.first(), Some(&0));
    assert!(samples.last().is_some_and(|last| last.unsigned_abs() < 200));
}

#[test]
fn a_tone_never_gets_louder_than_its_volume() {
    let peak = chirp()
        .samples()
        .iter()
        .map(|sample| sample.unsigned_abs())
        .max();

    assert!(peak.is_some_and(|peak| peak <= 16_384 && peak > 10_000));
}

#[test]
fn a_tone_is_audible_rather_than_flat() {
    let samples = chirp().samples();
    let crossings = samples
        .windows(2)
        .filter(|pair| (pair[0] < 0) != (pair[1] < 0))
        .count();

    // A 400 to 800 Hz sweep over 0.1 s crosses zero about 2 x 60 times.
    assert!((100..140).contains(&crossings), "{crossings} crossings");
}

#[test]
fn a_wav_file_has_a_pcm_mono_16_bit_header_and_its_samples() {
    let samples = [0_i16, 1000, -1000];

    let bytes = wav(&samples);

    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(u32_at(&bytes, 4), Some(36 + 6));
    assert_eq!(&bytes[8..16], b"WAVEfmt ");
    assert_eq!(u32_at(&bytes, 16), Some(16));
    assert_eq!(u16_at(&bytes, 20), Some(1));
    assert_eq!(u16_at(&bytes, 22), Some(1));
    assert_eq!(u32_at(&bytes, 24), Some(SAMPLE_RATE));
    assert_eq!(u32_at(&bytes, 28), Some(SAMPLE_RATE * 2));
    assert_eq!(u16_at(&bytes, 32), Some(2));
    assert_eq!(u16_at(&bytes, 34), Some(16));
    assert_eq!(&bytes[36..40], b"data");
    assert_eq!(u32_at(&bytes, 40), Some(6));
    assert_eq!(&bytes[44..], &[0, 0, 0xE8, 0x03, 0x18, 0xFC]);
}

#[test]
fn a_tone_fades_in_over_its_first_milliseconds() {
    let samples = chirp().samples();
    let first_millisecond = samples
        .iter()
        .take(44)
        .map(|sample| sample.unsigned_abs())
        .max();

    assert!(first_millisecond.is_some_and(|peak| peak < 4_000));
}
