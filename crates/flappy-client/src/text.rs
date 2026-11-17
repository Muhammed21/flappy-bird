/// Font sizes are rasterized in steps of this many pixels.
const STEP: f32 = 16.0;
const MAX_STEPS: f32 = 16.0;

/// How to draw text of a given pixel size: rasterize at `font_size`, then scale by `scale`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextSize {
    pub font_size: u16,
    pub scale: f32,
}

/// Every new font size makes macroquad rebuild its glyph texture, so text that follows the
/// window's size would rebuild it on each resize. Sizes are rounded up to one of 16 steps
/// (rasterized larger, scaled down, so they stay sharp).
#[must_use]
pub fn text_size(size: f32) -> TextSize {
    if !(size >= 0.0 && size.is_finite()) {
        return TextSize {
            font_size: 16,
            scale: 1.0,
        };
    }
    let steps = (size / STEP).ceil().clamp(1.0, MAX_STEPS);
    let font_size = whole_pixels(steps * STEP);
    TextSize {
        font_size,
        scale: size / f32::from(font_size),
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a whole number between 16 and 256"
)]
fn whole_pixels(size: f32) -> u16 {
    size as u16
}
