mod effects;
mod game;
mod keyboard;
mod pairs;
mod sprites;

use macroquad::prelude::*;

use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Herigone".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let mut game = Game::new();

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        game.update();
        game.draw();
        next_frame().await
    }
}
