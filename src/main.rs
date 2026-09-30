#![cfg_attr(test, allow(clippy::float_cmp))]

mod app;
mod booklet;
mod cli;
mod gfx;
mod input;
mod platform;
mod screens;
mod sound;

// The rules live in the core crate; these names keep the paths short.
use lukuloitsu_core::{
    badges, curriculum, game, long_numbers, memory, obstacles, pairs, progress, rng,
};
use macroquad::prelude::*;

use app::{App, Effect};
use cli::Command;
use gfx::{fonts, icon};
use input::{frame::Inputs, touch};
use platform::{analytics, save};
use sound::{
    audio::{self, Audio},
    music,
};

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
    let args: Vec<String> = std::env::args().skip(1).collect();
    match Command::parse(&args) {
        Command::Play => play().await,
        Command::RenderIcons => icon::render_all(env!("CARGO_MANIFEST_DIR")),
        Command::RenderMusic { path } => {
            std::fs::write(&path, audio::wav(&music::music()))
                .expect("the music file should be writable");
            println!("wrote {path}");
        }
        Command::RenderBooklet { path } => {
            booklet::write(&path);
            println!("wrote {path}");
        }
    }
}

/// Runs the game until the player quits.
async fn play() {
    // Touches are handled directly, so they shouldn't also act as a mouse.
    simulate_mouse_with_touch(false);
    let touch_mode = touch::enabled();
    let mut audio = Audio::load().await;
    #[cfg(target_arch = "wasm32")]
    platform::web::loaded();
    let (mut app, launch) = App::start(
        save::load(),
        touch_mode,
        // A web page can't be quit, only left.
        !cfg!(target_arch = "wasm32"),
        Inputs::now(),
    );
    let inputs = Inputs::new();

    perform(launch, &mut audio);
    loop {
        if perform(app.update(&inputs.read()), &mut audio) {
            break;
        }
        audio.set_music_on(app.music_on());
        audio.set_music(app.wants_music());
        app.draw();
        next_frame().await
    }
}

/// Does what the app asked for, and says whether it asked to quit.
fn perform(effects: Vec<Effect>, audio: &mut Audio) -> bool {
    let mut quit = false;
    for effect in effects {
        match effect {
            Effect::Play(sfx) => audio.play(sfx),
            Effect::Save(text) => save::store(&text),
            Effect::Count(event) => analytics::send(&event),
            Effect::Quit => quit = true,
        }
    }
    quit
}
