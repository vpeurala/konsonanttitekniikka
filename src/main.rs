use macroquad::prelude::*;

const PLAYER_SPEED: f32 = 300.0;
const PLAYER_SIZE: f32 = 32.0;

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
    let mut player = vec2(screen_width() / 2.0, screen_height() / 2.0);

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        let mut dir = Vec2::ZERO;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            dir.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            dir.x += 1.0;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            dir.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            dir.y += 1.0;
        }
        player += dir.normalize_or_zero() * PLAYER_SPEED * get_frame_time();

        let half = PLAYER_SIZE / 2.0;
        player.x = player.x.clamp(half, screen_width() - half);
        player.y = player.y.clamp(half, screen_height() - half);

        clear_background(Color::from_rgba(24, 24, 32, 255));
        draw_rectangle(
            player.x - half,
            player.y - half,
            PLAYER_SIZE,
            PLAYER_SIZE,
            ORANGE,
        );
        draw_text(
            "Move with WASD / arrows, Esc to quit",
            16.0,
            28.0,
            24.0,
            LIGHTGRAY,
        );
        draw_text(format!("FPS: {}", get_fps()), 16.0, 52.0, 20.0, GRAY);

        next_frame().await
    }
}
