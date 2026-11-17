use flappy_client::{Area, VIEW_HEIGHT, VIEW_WIDTH, visible_area};

#[test]
fn a_screen_with_the_view_proportions_shows_exactly_the_view() {
    assert_eq!(
        visible_area(VIEW_WIDTH * 3.0, VIEW_HEIGHT * 3.0),
        Area {
            x: 0.0,
            y: 0.0,
            width: VIEW_WIDTH,
            height: VIEW_HEIGHT
        }
    );
}

#[test]
fn a_wider_screen_shows_the_whole_height_and_bars_on_both_sides() {
    assert_eq!(
        visible_area(VIEW_WIDTH * 4.0, VIEW_HEIGHT * 2.0),
        Area {
            x: -VIEW_WIDTH / 2.0,
            y: 0.0,
            width: VIEW_WIDTH * 2.0,
            height: VIEW_HEIGHT
        }
    );
}

#[test]
fn a_taller_screen_shows_the_whole_width_and_bars_above_and_below() {
    assert_eq!(
        visible_area(VIEW_WIDTH, VIEW_HEIGHT * 2.0),
        Area {
            x: 0.0,
            y: -VIEW_HEIGHT / 2.0,
            width: VIEW_WIDTH,
            height: VIEW_HEIGHT * 2.0
        }
    );
}

#[test]
fn a_screen_without_width_or_height_falls_back_to_the_view() {
    let view = Area {
        x: 0.0,
        y: 0.0,
        width: VIEW_WIDTH,
        height: VIEW_HEIGHT,
    };

    assert_eq!(visible_area(0.0, 0.0), view);
    assert_eq!(visible_area(0.0, 100.0), view);
    assert_eq!(visible_area(100.0, 0.0), view);
}

#[test]
fn the_view_is_as_wide_as_the_game_world() {
    assert!((flappy_core::Config::default().view_width - VIEW_WIDTH).abs() < f32::EPSILON);
}
