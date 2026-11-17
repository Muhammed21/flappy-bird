/// `SplitMix64` over `(seed, index)`: one independent, well-mixed value per pipe, with no state to
/// carry. Changing it changes every course, so `tests/course.rs` pins its output.
pub(crate) fn mix(seed: u64, index: u64) -> u64 {
    let z = seed.wrapping_add(index.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A value in `[0, 1)` built from the top 24 bits, which `f32` holds exactly.
#[expect(
    clippy::cast_precision_loss,
    reason = "a 24-bit integer is exact in f32"
)]
pub(crate) fn unit(value: u64) -> f32 {
    (value >> 40) as f32 / 16_777_216.0
}
