use flappy_client::{
    Area, Events, MAX_NOSE_DOWN, Records, Session, Tone, VIEW_HEIGHT, VIEW_WIDTH, bird_tilt,
    text_size, visible_area, wav,
};
use flappy_core::{Config, Phase, Pipe};
use flappy_storage::Store;
use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};
use macroquad::prelude::{
    BLACK, Camera2D, Color, Conf, KeyCode, MouseButton, Rect, TextParams, Vec2, WHITE,
    clear_background, draw_circle, draw_rectangle, draw_text_ex, draw_triangle, get_frame_time,
    is_key_pressed, is_mouse_button_pressed, measure_text, next_frame, screen_height, screen_width,
    set_camera, set_default_camera, vec2,
};

const SKY: Color = Color::new(0.44, 0.77, 0.81, 1.0);
const CLOUD: Color = Color::new(0.93, 0.97, 0.98, 1.0);
const PIPE: Color = Color::new(0.45, 0.75, 0.18, 1.0);
const PIPE_SHADE: Color = Color::new(0.33, 0.58, 0.12, 1.0);
const GROUND: Color = Color::new(0.87, 0.85, 0.58, 1.0);
const GROUND_STRIPE: Color = Color::new(0.80, 0.77, 0.48, 1.0);
const GRASS: Color = Color::new(0.52, 0.80, 0.25, 1.0);
const BIRD: Color = Color::new(0.98, 0.82, 0.19, 1.0);
const WING: Color = Color::new(0.95, 0.67, 0.13, 1.0);
const BEAK: Color = Color::new(0.94, 0.42, 0.16, 1.0);

const TAP_TO_FLY: &str = "Espace ou clic pour voler";
const TAP_TO_RETRY: &str = "Espace ou clic pour rejouer";
const BEST: &str = "Meilleur";

const CAP_HEIGHT: f32 = 20.0;
const CAP_OVERHANG: f32 = 4.0;
const STRIPE_WIDTH: f32 = 24.0;
const CLOUD_SPACING: f32 = 120.0;
const CLOUD_PARALLAX: f32 = 0.25;

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

#[cfg(target_arch = "wasm32")]
fn store() -> Box<dyn Store> {
    Box::new(flappy_web_storage::WebStore)
}

#[cfg(not(target_arch = "wasm32"))]
fn store() -> Box<dyn Store> {
    match flappy_storage::data_dir(std::env::consts::OS, |name| std::env::var(name).ok()) {
        Some(dir) => Box::new(flappy_storage::FileStore::new(dir)),
        None => Box::new(Nowhere),
    }
}

/// No place to save: the best score lasts for this session only.
#[cfg(not(target_arch = "wasm32"))]
struct Nowhere;

#[cfg(not(target_arch = "wasm32"))]
impl Store for Nowhere {
    fn read(&self) -> Option<String> {
        None
    }

    fn write(&self, _: &str) -> std::io::Result<()> {
        Err(std::io::Error::other("no data folder"))
    }
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

struct Sounds {
    flap: Sound,
    score: Sound,
    crash: Sound,
}

async fn tone(tone: Tone) -> Option<Sound> {
    load_sound_from_bytes(&wav(&tone.samples())).await.ok()
}

async fn load_sounds() -> Option<Sounds> {
    Some(Sounds {
        flap: tone(Tone {
            start_hz: 500.0,
            end_hz: 900.0,
            seconds: 0.08,
            volume: 0.35,
        })
        .await?,
        score: tone(Tone {
            start_hz: 880.0,
            end_hz: 1320.0,
            seconds: 0.12,
            volume: 0.3,
        })
        .await?,
        crash: tone(Tone {
            start_hz: 300.0,
            end_hz: 80.0,
            seconds: 0.3,
            volume: 0.5,
        })
        .await?,
    })
}

fn play(sound: &Sound) {
    play_sound(
        sound,
        PlaySoundParams {
            looped: false,
            volume: 1.0,
        },
    );
}

fn play_events(sounds: &Sounds, events: Events) {
    [
        (events.flapped, &sounds.flap),
        (events.scored, &sounds.score),
        (events.crashed, &sounds.crash),
    ]
    .into_iter()
    .filter(|(happened, _)| *happened)
    .for_each(|(_, sound)| play(sound));
}

fn draw_clouds(shown_distance: f32, area: Area) {
    let offset = (shown_distance * CLOUD_PARALLAX) % CLOUD_SPACING;
    let heights = [70.0, 40.0, 95.0, 55.0];
    (0_u8..5)
        .zip(heights.into_iter().cycle())
        .for_each(|(index, y)| {
            let x = f32::from(index) * CLOUD_SPACING - offset + area.x.min(0.0);
            draw_circle(x, y, 18.0, CLOUD);
            draw_circle(x + 18.0, y + 4.0, 14.0, CLOUD);
            draw_circle(x - 16.0, y + 6.0, 12.0, CLOUD);
        });
}

fn draw_pipe(pipe: &Pipe, x: f32, config: &Config, area: Area) {
    let shade = pipe.width * 0.2;
    let body = |top: f32, height: f32| {
        draw_rectangle(x, top, pipe.width, height, PIPE);
        draw_rectangle(x + pipe.width - shade, top, shade, height, PIPE_SHADE);
    };
    let cap = |top: f32| {
        let width = pipe.width + 2.0 * CAP_OVERHANG;
        draw_rectangle(x - CAP_OVERHANG, top, width, CAP_HEIGHT, PIPE);
        draw_rectangle(
            x - CAP_OVERHANG + width - shade,
            top,
            shade,
            CAP_HEIGHT,
            PIPE_SHADE,
        );
    };
    body(area.y, pipe.gap.top - area.y);
    cap(pipe.gap.top - CAP_HEIGHT);
    body(pipe.gap.bottom, config.ground_y - pipe.gap.bottom);
    cap(pipe.gap.bottom);
}

fn draw_ground(shown_distance: f32, config: &Config, area: Area) {
    let bottom = area.y + area.height;
    let offset = shown_distance % STRIPE_WIDTH;
    draw_rectangle(
        area.x,
        config.ground_y,
        area.width,
        bottom - config.ground_y,
        GROUND,
    );
    (0_u8..16).for_each(|index| {
        let x = f32::from(index) * STRIPE_WIDTH - offset;
        draw_rectangle(
            x,
            config.ground_y + 8.0,
            STRIPE_WIDTH / 2.0,
            6.0,
            GROUND_STRIPE,
        );
    });
    draw_rectangle(area.x, config.ground_y, area.width, 8.0, GRASS);
}

fn draw_bird(centre: Vec2, angle: f32, radius: f32) {
    let (sin, cos) = angle.sin_cos();
    let at = |dx: f32, dy: f32| centre + vec2(dx * cos - dy * sin, dx * sin + dy * cos);
    draw_circle(centre.x, centre.y, radius, BIRD);
    let wing = at(-radius * 0.35, radius * 0.2);
    draw_circle(wing.x, wing.y, radius * 0.45, WING);
    let eye = at(radius * 0.4, -radius * 0.35);
    draw_circle(eye.x, eye.y, radius * 0.32, WHITE);
    let pupil = at(radius * 0.5, -radius * 0.35);
    draw_circle(pupil.x, pupil.y, radius * 0.14, BLACK);
    draw_triangle(
        at(radius * 0.8, -radius * 0.15),
        at(radius * 1.45, radius * 0.1),
        at(radius * 0.8, radius * 0.35),
        BEAK,
    );
}

fn draw_world(session: &Session, config: &Config, area: Area) {
    let world = session.world();
    let previous = session.previous();
    let alpha = session.alpha();
    let lerp = |from: f32, to: f32| from + (to - from) * alpha;
    let scroll = match world.phase() {
        Phase::Playing => config.scroll_speed * (1.0 - alpha),
        Phase::Ready | Phase::Over => 0.0,
    };
    let shown_distance = world.distance() - scroll;

    clear_background(BLACK);
    draw_rectangle(area.x, area.y, area.width, area.height, SKY);
    draw_clouds(shown_distance, area);
    for pipe in world.pipes() {
        draw_pipe(&pipe, pipe.x + scroll, config, area);
    }
    draw_ground(shown_distance, config, area);

    let velocity = lerp(previous.bird().velocity, world.bird().velocity);
    let tilt = match world.phase() {
        Phase::Ready => 0.0,
        Phase::Playing => bird_tilt(velocity, config),
        Phase::Over => MAX_NOSE_DOWN,
    };
    let centre = vec2(config.bird_x, lerp(previous.bird().y, world.bird().y));
    draw_bird(centre, tilt, config.bird_radius);

    draw_rectangle(area.x, area.y, -area.x, area.height, BLACK);
    draw_rectangle(VIEW_WIDTH, area.y, -area.x, area.height, BLACK);
    draw_rectangle(area.x, area.y, area.width, -area.y, BLACK);
    draw_rectangle(area.x, VIEW_HEIGHT, area.width, -area.y, BLACK);
}

/// Text is drawn in screen pixels so it stays sharp at any window size.
fn draw_centered(text: &str, view_y: f32, view_size: f32, area: Area) {
    let scale = screen_width() / area.width;
    let size = text_size(view_size * scale);
    let width = measure_text(text, None, size.font_size, size.scale).width;
    let x = (VIEW_WIDTH / 2.0 - area.x) * scale - width / 2.0;
    let y = (view_y - area.y) * scale;
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font_size: size.font_size,
            font_scale: size.scale,
            color: WHITE,
            ..TextParams::default()
        },
    );
}

fn draw_hud(session: &Session, records: &Records, area: Area) {
    let world = session.world();
    let best = records.best_score().map(|best| format!("{BEST} : {best}"));
    match world.phase() {
        Phase::Ready => draw_centered(TAP_TO_FLY, 300.0, 18.0, area),
        Phase::Playing => draw_centered(&world.score().to_string(), 80.0, 48.0, area),
        Phase::Over => {
            draw_centered(&world.score().to_string(), 80.0, 48.0, area);
            draw_centered(TAP_TO_RETRY, 300.0, 18.0, area);
        }
    }
    if let (Some(best), Phase::Ready | Phase::Over) = (best, world.phase()) {
        draw_centered(&best, 120.0, 20.0, area);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let config = Config::default();
    let store = store();
    let sounds = load_sounds().await;
    let mut records = Records::load(store.as_ref());
    let mut session = Session::new(config, seed());
    loop {
        session = session.frame(f64::from(get_frame_time()), flapped());
        records = records.after_frame(&session, store.as_ref());
        if let Some(sounds) = &sounds {
            play_events(sounds, session.events());
        }
        let area = visible_area(screen_width(), screen_height());

        set_camera(&view_camera(area));
        draw_world(&session, &config, area);
        set_default_camera();
        draw_hud(&session, &records, area);

        next_frame().await;
    }
}
