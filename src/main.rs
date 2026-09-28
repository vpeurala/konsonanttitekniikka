mod audio;
mod curriculum;
mod effects;
mod game;
mod keyboard;
mod memory;
mod obstacles;
mod pairs;
mod portals;
mod rng;
mod sprites;
mod title;

use macroquad::prelude::*;

use audio::{Audio, Sfx};
use game::Game;
use title::{TitleAction, TitleScreen};

fn window_conf() -> Conf {
    Conf {
        window_title: "Konsonanttitekniikka".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut audio = Audio::load().await;
    let mut title = TitleScreen::new();
    // Created when the player leaves the title screen.
    let mut game: Option<Game> = None;

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Tab) {
            audio.toggle_music();
        }

        let Some(game) = &mut game else {
            if let TitleAction::StartGame = title.update() {
                game = Some(Game::new());
            }
            audio.set_music(true);
            title.draw();
            next_frame().await;
            continue;
        };

        let was_over = game.is_over();
        game.update();
        for sfx in game.take_sfx() {
            audio.play(sfx);
        }
        if !was_over && game.is_over() {
            audio.play(Sfx::GameOver);
        }
        audio.set_music(!game.is_over() && !game.is_paused());

        game.draw();
        next_frame().await
    }
}
