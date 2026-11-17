use flappy_client::{Area, Session, VIEW_HEIGHT, VIEW_WIDTH, visible_area};
use flappy_core::{Config, Phase};
use macroquad::prelude::{
    BLACK, Camera2D, Color, Conf, KeyCode, MouseButton, Rect, WHITE, clear_background, draw_circle,
    draw_rectangle, draw_text, get_frame_time, is_key_pressed, is_mouse_button_pressed,
    measure_text, next_frame, screen_height, screen_width, set_camera, set_default_camera, vec2,
};

const SKY: Color = Color::new(0.44, 0.77, 0.81, 1.0);
const PIPE: Color = Color::new(0.45, 0.75, 0.18, 1.0);
const GROUND: Color = Color::new(0.87, 0.85, 0.58, 1.0);
const BIRD: Color = Color::new(0.98, 0.82, 0.19, 1.0);

const TAP_TO_FLY: &str = "Espace ou clic pour voler";
const TAP_TO_RETRY: &str = "Espace ou clic pour rejouer";

fn window_conf() -> Conf {
    Conf {
        window_title: "Flappy Bird".to_owned(),
        window_width: 432,
        window_height: 768,
        window_resizable: true,
        high_dpi: true,
        ..Conf::default()
    }
}

fn flapped() -> bool {
    [KeyCode::Space, KeyCode::Up, KeyCode::W]
        .into_iter()
        .any(is_key_pressed)
        || is_mouse_button_pressed(MouseButton::Left)
}

fn seed() -> u64 {
    macroquad::miniquad::date::now().to_bits()
}

/// A camera showing `area` with y pointing down, like the core. `from_display_rect` points it up.
fn view_camera(area: Area) -> Camera2D {
    let camera = Camera2D::from_display_rect(Rect::new(area.x, area.y, area.width, area.height));
    Camera2D {
        zoom: vec2(camera.zoom.x, -camera.zoom.y),
        ..camera
    }
}

fn draw_world(session: &Session, config: &Config, area: Area) {
    let world = session.world();
    let alpha = session.alpha();
    let scroll = match world.phase() {
        Phase::Playing => config.scroll_speed * (1.0 - alpha),
        Phase::Ready | Phase::Over => 0.0,
    };
    let bottom = area.y + area.height;

    clear_background(BLACK);
    draw_rectangle(0.0, 0.0, VIEW_WIDTH, VIEW_HEIGHT, SKY);
    for pipe in world.pipes() {
        let x = pipe.x + scroll;
        draw_rectangle(x, area.y, pipe.width, pipe.gap.top - area.y, PIPE);
        draw_rectangle(
            x,
            pipe.gap.bottom,
            pipe.width,
            config.ground_y - pipe.gap.bottom,
            PIPE,
        );
    }
    draw_rectangle(
        0.0,
        config.ground_y,
        VIEW_WIDTH,
        bottom - config.ground_y,
        GROUND,
    );

    let before = session.previous().bird().y;
    let bird_y = before + (world.bird().y - before) * alpha;
    draw_circle(config.bird_x, bird_y, config.bird_radius, BIRD);

    draw_rectangle(area.x, area.y, -area.x, area.height, BLACK);
    draw_rectangle(VIEW_WIDTH, area.y, -area.x, area.height, BLACK);
    draw_rectangle(area.x, area.y, area.width, -area.y, BLACK);
    draw_rectangle(area.x, VIEW_HEIGHT, area.width, -area.y, BLACK);
}

/// Text is drawn in screen pixels so it stays sharp at any window size.
fn draw_centered(text: &str, view_y: f32, view_size: f32, area: Area) {
    let scale = screen_width() / area.width;
    let size = view_size * scale;
    let width = measure_text(text, None, 1, 1.0).width * size;
    let x = (VIEW_WIDTH / 2.0 - area.x) * scale - width / 2.0;
    let y = (view_y - area.y) * scale;
    draw_text(text, x, y, size, WHITE);
}

fn draw_hud(session: &Session, area: Area) {
    let world = session.world();
    match world.phase() {
        Phase::Ready => draw_centered(TAP_TO_FLY, 300.0, 18.0, area),
        Phase::Playing => draw_centered(&world.score().to_string(), 80.0, 48.0, area),
        Phase::Over => {
            draw_centered(&world.score().to_string(), 80.0, 48.0, area);
            draw_centered(TAP_TO_RETRY, 300.0, 18.0, area);
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let config = Config::default();
    let mut session = Session::new(config, seed());
    loop {
        session = session.frame(f64::from(get_frame_time()), flapped());
        let area = visible_area(screen_width(), screen_height());

        set_camera(&view_camera(area));
        draw_world(&session, &config, area);
        set_default_camera();
        draw_hud(&session, area);

        next_frame().await;
    }
}
