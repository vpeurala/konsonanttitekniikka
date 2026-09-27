mod audio;
mod effects;
mod game;
mod keyboard;
mod pairs;
mod sprites;

use macroquad::prelude::*;

use audio::{Audio, Sfx};
use game::Game;

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
    rand::srand(miniquad::date::now() as u64);
    let mut audio = Audio::load().await;
    let mut game = Game::new();

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Tab) {
            audio.toggle_music();
        }

        let was_over = game.is_over();
        game.update();
        for sfx in game.take_sfx() {
            audio.play(sfx);
        }
        if !was_over && game.is_over() {
            audio.play(Sfx::GameOver);
        }
        audio.set_music(!game.is_over());

        game.draw();
        next_frame().await
    }
}
