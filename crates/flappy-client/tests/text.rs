use flappy_client::{TextSize, text_size};

#[test]
fn a_size_on_a_step_is_drawn_as_is() {
    assert_eq!(
        text_size(48.0),
        TextSize {
            font_size: 48,
            scale: 1.0
        }
    );
}

#[test]
fn other_sizes_are_rasterized_at_the_next_step_up_and_scaled_down_to_stay_sharp() {
    let size = text_size(40.0);

    assert_eq!(size.font_size, 48);
    assert!((size.scale - 40.0 / 48.0).abs() < 1e-6);
}

#[test]
fn tiny_and_huge_sizes_stay_within_the_rasterized_range() {
    assert_eq!(text_size(3.0).font_size, 16);
    assert_eq!(text_size(1000.0).font_size, 256);
    assert!((text_size(1000.0).scale - 1000.0 / 256.0).abs() < 1e-4);
}

#[test]
fn resizing_the_window_uses_only_a_handful_of_font_sizes() {
    let sizes: std::collections::BTreeSet<u16> = (10..400)
        .map(|size| text_size(f32::from(u16::try_from(size).unwrap())).font_size)
        .collect();

    assert!(sizes.len() <= 16, "{} font sizes", sizes.len());
}

#[test]
fn a_size_of_zero_draws_nothing() {
    assert_eq!(
        text_size(0.0),
        TextSize {
            font_size: 16,
            scale: 0.0
        }
    );
}

#[test]
fn a_broken_size_falls_back_to_the_smallest_step() {
    assert_eq!(
        text_size(f32::NAN),
        TextSize {
            font_size: 16,
            scale: 1.0
        }
    );
    assert_eq!(
        text_size(-5.0),
        TextSize {
            font_size: 16,
            scale: 1.0
        }
    );
}
