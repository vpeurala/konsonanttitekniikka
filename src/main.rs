mod analytics;
mod app;
mod audio;
mod badge_art;
mod badge_screen;
mod badges;
mod booklet;
mod curriculum;
mod effects;
mod fonts;
mod frame;
mod game;
mod icon;
mod keyboard;
mod levels;
mod lifecycle;
mod long_numbers;
mod memory;
mod music;
mod obstacles;
mod pairs;
mod pictures;
mod portals;
mod practice;
mod progress;
mod rng;
mod save;
mod sprites;
mod title;
mod toast;
mod touch;
mod view;
#[cfg(target_arch = "wasm32")]
mod web;

use macroquad::prelude::*;

use app::{App, Effect};
use audio::Audio;
use frame::Inputs;

fn window_conf() -> Conf {
    Conf {
        window_title: "Lukuloitsu".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

/// Everything that touches the outside world happens here: reading the
/// window's input, and doing what the app asks for. The app itself is a
/// pure state machine that turns each frame of input into effects.
#[macroquad::main(window_conf)]
async fn main() {
    fonts::init();
    // `cargo run -- --render-icons` regenerates the app icon files.
    if std::env::args().any(|a| a == "--render-icons") {
        icon::render_all(env!("CARGO_MANIFEST_DIR"));
        return;
    }
    // `cargo run --release -- --render-music FILE.wav` writes the music
    // loop to a file, for listening to it outside the game.
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--render-music") {
        let path = args.get(i + 1).map_or("music.wav", String::as_str);
        std::fs::write(path, audio::wav(&music::music()))
            .expect("the music file should be writable");
        println!("wrote {path}");
        return;
    }
    // `cargo run --release -- --render-booklet FILE.html` writes the user
    // instruction booklet, which `scripts/booklet.sh` turns into a PDF.
    if let Some(i) = args.iter().position(|a| a == "--render-booklet") {
        let path = args.get(i + 1).map_or("opas.html", String::as_str);
        booklet::write(path);
        println!("wrote {path}");
        return;
    }
    // Touches are handled directly, so they shouldn't also act as a mouse.
    simulate_mouse_with_touch(false);
    let touch_mode = touch::enabled();
    let audio = Audio::load().await;
    #[cfg(target_arch = "wasm32")]
    web::loaded();
    let mut progress = save::load();
    progress.record_play_day(save::day_of(miniquad::date::now()));
    save::store(&progress.to_text());
    let mut audio = audio;

    let mut app = App::new(
        progress,
        touch_mode,
        // A web page can't be quit, only left.
        !cfg!(target_arch = "wasm32"),
        miniquad::date::now(),
    );
    let inputs = Inputs::new();

    loop {
        let mut quit = false;
        for effect in app.update(&inputs.read()) {
            match effect {
                Effect::Play(sfx) => audio.play(sfx),
                Effect::Save(text) => save::store(&text),
                Effect::Count(event) => analytics::send(&event),
                Effect::Quit => quit = true,
            }
        }
        if quit {
            break;
        }
        audio.set_music_on(app.music_on());
        audio.set_music(app.wants_music());
        app.draw();
        next_frame().await
    }
}
