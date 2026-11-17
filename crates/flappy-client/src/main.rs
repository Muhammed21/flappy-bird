use macroquad::prelude::{Color, Conf, clear_background, next_frame};

const SKY: Color = Color::new(0.44, 0.77, 0.81, 1.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Flappy Bird".to_owned(),
        window_width: 288 * 2,
        window_height: 512 * 2,
        window_resizable: false,
        high_dpi: true,
        ..Conf::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        clear_background(SKY);
        next_frame().await;
    }
}
