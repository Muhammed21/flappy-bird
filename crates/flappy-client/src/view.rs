/// The game's logical screen, in the same units as `flappy_core`.
pub const VIEW_WIDTH: f32 = 288.0;
pub const VIEW_HEIGHT: f32 = 512.0;

/// A rectangle in view units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

const VIEW: Area = Area {
    x: 0.0,
    y: 0.0,
    width: VIEW_WIDTH,
    height: VIEW_HEIGHT,
};

/// The part of the world a screen of this size shows: the whole view, scaled to fit, centred,
/// plus whatever the screen's proportions add on two sides (drawn as bars).
#[must_use]
pub fn visible_area(screen_width: f32, screen_height: f32) -> Area {
    if !(screen_width > 0.0 && screen_height > 0.0) {
        return VIEW;
    }
    let scale = (screen_width / VIEW_WIDTH).min(screen_height / VIEW_HEIGHT);
    let width = screen_width / scale;
    let height = screen_height / scale;
    Area {
        x: (VIEW_WIDTH - width) / 2.0,
        y: (VIEW_HEIGHT - height) / 2.0,
        width,
        height,
    }
}
