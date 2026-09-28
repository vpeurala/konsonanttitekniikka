mod audio;
mod curriculum;
mod effects;
mod fonts;
mod game;
mod icon;
mod keyboard;
mod lifecycle;
mod memory;
mod obstacles;
mod pairs;
mod pictures;
mod portals;
mod rng;
mod sprites;
mod title;
mod touch;
mod view;

use macroquad::prelude::*;

use audio::{Audio, Sfx};
use game::Game;
use title::{TitleAction, TitleScreen};
use touch::{Button, TouchControls, TouchInput};
use view::{ARENA_H, ARENA_W};

fn window_conf() -> Conf {
    Conf {
        window_title: "Lukuloitsu".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    fonts::init();
    // `cargo run -- --render-icons` regenerates the app icon files.
    if std::env::args().any(|a| a == "--render-icons") {
        icon::render_all(env!("CARGO_MANIFEST_DIR"));
        return;
    }
    // Touches are handled directly, so they shouldn't also act as a mouse.
    simulate_mouse_with_touch(false);
    let touch_mode = touch::enabled();
    let mut audio = Audio::load().await;
    let mut title = TitleScreen::new(touch_mode);
    let mut controls = TouchControls::default();
    let touches = touch::TouchReader::new();
    let lifecycle = lifecycle::Lifecycle::new();
    // Created when the player leaves the title screen.
    let mut game: Option<Game> = None;

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Tab) {
            audio.toggle_music();
        }
        // Coming back after the app was in the background (or the phone was
        // locked) finds the game paused rather than lost.
        let was_away = lifecycle.was_away();

        let Some(game) = &mut game else {
            let title_rect = Rect::new(0.0, 0.0, ARENA_W, ARENA_H);
            let view = view::begin(title_rect);
            if let TitleAction::StartGame = title.update(&touches.pointers(&view)) {
                game = Some(Game::new(touch_mode));
            }
            audio.set_music(true);
            title.draw();
            view::mask_outside(title_rect, BLACK);
            next_frame().await;
            continue;
        };

        let content = touch::content_rect(touch_mode);
        let view = view::begin(content);
        let input = if touch_mode {
            controls.update(&touches.pointers(&view), get_frame_time())
        } else {
            TouchInput::default()
        };
        if input.buttons.contains(&Button::Music) {
            audio.toggle_music();
        }

        if was_away {
            game.pause();
        }
        let was_over = game.is_over();
        game.update(&input);
        for sfx in game.take_sfx() {
            audio.play(sfx);
        }
        if !was_over && game.is_over() {
            audio.play(Sfx::GameOver);
        }
        audio.set_music(!game.is_over() && !game.is_paused());

        game.draw();
        if touch_mode {
            controls.draw(audio.music_on(), game.is_paused());
        }
        view::mask_outside(content, BLACK);
        next_frame().await
    }
}
