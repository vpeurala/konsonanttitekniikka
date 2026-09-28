mod audio;
mod curriculum;
mod effects;
mod fonts;
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
mod touch;
mod view;
#[cfg(target_arch = "wasm32")]
mod web;

use macroquad::prelude::*;

use audio::{Audio, Sfx};
use game::{Game, GameEvent};
use keyboard::Keyboard;
use levels::{LevelAction, LevelSelect};
use practice::{PracticeAction, PracticeScreen};
use progress::ProgressScreen;
use save::SaveData;
use title::{TitleAction, TitleScreen};
use touch::{Button, TouchControls, TouchInput};
use view::{ARENA_H, ARENA_W};

/// How often progress is saved while playing, in seconds, so little is lost
/// if the app is closed or killed in the background.
const SAVE_INTERVAL: f64 = 5.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Lukuloitsu".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

/// What is on screen.
enum Screen {
    Title,
    /// Choosing the level to start from.
    Levels(LevelSelect),
    Game(Box<Game>),
    Practice(Box<PracticeScreen>),
    Progress(ProgressScreen),
}

/// Toggles the music and remembers the choice.
fn toggle_music(audio: &mut Audio, progress: &mut SaveData) {
    audio.toggle_music();
    progress.music_on = audio.music_on();
    save::store(progress);
}

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
        std::fs::write(path, audio::wav(&music::music())).expect("the music file should be writable");
        println!("wrote {path}");
        return;
    }
    // Touches are handled directly, so they shouldn't also act as a mouse.
    simulate_mouse_with_touch(false);
    let touch_mode = touch::enabled();
    let mut audio = Audio::load().await;
    #[cfg(target_arch = "wasm32")]
    web::loaded();
    let mut progress = save::load();
    progress.record_play_day(save::day_of(miniquad::date::now()));
    save::store(&progress);
    audio.set_music_on(progress.music_on);
    let mut last_save = miniquad::date::now();

    let mut title = TitleScreen::new(touch_mode);
    let mut controls = TouchControls::default();
    let touches = touch::TouchReader::new();
    let lifecycle = lifecycle::Lifecycle::new();
    // Typing on the practice screen; the game reads its own keyboard.
    let practice_keys = Keyboard::new();
    let mut screen = Screen::Title;

    loop {
        // Esc leaves the game or a menu screen, and quits from the title.
        let escape = is_key_pressed(KeyCode::Escape);
        if is_key_pressed(KeyCode::Tab) {
            toggle_music(&mut audio, &mut progress);
        }
        // Coming back after the app was in the background (or the phone was
        // locked) finds the game paused rather than lost.
        let was_away = lifecycle.was_away();
        // Read every frame, so keys typed elsewhere don't pile up.
        let typed = practice_keys.typed();

        let title_rect = Rect::new(0.0, 0.0, ARENA_W, ARENA_H);
        let mut next = None;
        match &mut screen {
            Screen::Title => {
                // A web page can't be quit, only left.
                if escape && !cfg!(target_arch = "wasm32") {
                    break;
                }
                let view = view::begin(title_rect);
                match title.update(&touches.pointers(&view)) {
                    TitleAction::Stay => {}
                    TitleAction::StartGame => {
                        // With only the first level to start from, there
                        // is nothing to choose.
                        let levels = LevelSelect::new(progress.best_level, touch_mode);
                        next = Some(if levels.len() > 1 {
                            Screen::Levels(levels)
                        } else {
                            Screen::Game(Box::new(Game::new(touch_mode, progress.memory(), 1)))
                        });
                    }
                    TitleAction::Practice => {
                        let memory = progress.memory();
                        next = Some(Screen::Practice(Box::new(PracticeScreen::new(
                            progress.best_level,
                            &memory,
                            touch_mode,
                        ))));
                    }
                    TitleAction::Progress => next = Some(Screen::Progress(ProgressScreen)),
                }
                audio.set_music(true);
                title.draw(&progress);
                view::mask_outside(title_rect, BLACK);
            }

            Screen::Levels(levels) => {
                let view = view::begin(title_rect);
                match levels.update(&touches.pointers(&view)) {
                    LevelAction::Stay => {}
                    LevelAction::Back => next = Some(Screen::Title),
                    LevelAction::Start(level) => {
                        next = Some(Screen::Game(Box::new(Game::new(
                            touch_mode,
                            progress.memory(),
                            level,
                        ))));
                    }
                }
                audio.set_music(true);
                levels.draw();
                view::mask_outside(title_rect, BLACK);
            }

            Screen::Progress(progress_screen) => {
                let view = view::begin(title_rect);
                if progress_screen.update(&touches.pointers(&view)) {
                    next = Some(Screen::Title);
                }
                audio.set_music(true);
                progress_screen.draw(&progress, touch_mode);
                view::mask_outside(title_rect, BLACK);
            }

            Screen::Practice(practice) => {
                let content = touch::content_rect(touch_mode);
                let view = view::begin(content);
                let input = if touch_mode {
                    controls.update(&touches.pointers(&view), get_frame_time())
                } else {
                    TouchInput::default()
                };
                if input.buttons.contains(&Button::Music) {
                    toggle_music(&mut audio, &mut progress);
                }
                let mut keys = typed;
                keys.extend(input.keys.iter().copied());
                // The pause button leaves practice.
                let back = escape || input.buttons.contains(&Button::Pause);
                let mut memory = progress.memory();
                let action =
                    practice.update(&keys, input.arena_taps, back, get_frame_time(), &mut memory);
                progress.set_memory(&memory);
                for sfx in practice.take_sfx() {
                    audio.play(sfx);
                }
                let now = miniquad::date::now();
                let leaving = matches!(action, PracticeAction::Back);
                if leaving || now - last_save > SAVE_INTERVAL {
                    save::store(&progress);
                    last_save = now;
                }
                if leaving {
                    next = Some(Screen::Title);
                }
                audio.set_music(true);
                practice.draw();
                if touch_mode {
                    controls.draw(audio.music_on(), false, false);
                }
                view::mask_outside(content, BLACK);
            }

            Screen::Game(game) => {
                let content = touch::content_rect(touch_mode);
                let view = view::begin(content);
                let input = if touch_mode {
                    controls.update(&touches.pointers(&view), get_frame_time())
                } else {
                    TouchInput::default()
                };
                if input.buttons.contains(&Button::Music) {
                    toggle_music(&mut audio, &mut progress);
                }

                if was_away {
                    game.pause();
                }
                let was_over = game.is_over();
                game.update(&input);
                for sfx in game.take_sfx() {
                    audio.play(sfx);
                }
                let mut save_now = escape;
                for event in game.take_events() {
                    match event {
                        GameEvent::LevelCompleted { level, stars } => {
                            progress.record_level(level, stars);
                            save_now = true;
                        }
                    }
                }
                if !was_over && game.is_over() {
                    audio.play(Sfx::GameOver);
                    save_now = true;
                }
                let now = miniquad::date::now();
                if save_now || now - last_save > SAVE_INTERVAL {
                    progress.set_memory(game.memory());
                    save::store(&progress);
                    last_save = now;
                }
                if escape {
                    next = Some(Screen::Title);
                }
                audio.set_music(!game.is_over() && !game.is_paused());

                game.draw();
                if touch_mode {
                    controls.draw(audio.music_on(), game.is_paused(), true);
                }
                view::mask_outside(content, BLACK);
            }
        }
        if let Some(next) = next {
            screen = next;
        }
        next_frame().await
    }
}
