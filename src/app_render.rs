//! Drawing the app: whichever screen is showing, scaled to the window,
//! with the notice for a newly earned badge over it. The app itself, and
//! what it knows, is `lukuloitsu_core::app::App`; drawing only reads it.

use macroquad::prelude::BLACK;

use crate::gfx::view::{self, ARENA_W};
use crate::gfx::{game_render, touch_render};
use crate::screens::{badge_screen, levels, practice, progress, title, toast};
use lukuloitsu_core::app::{App, Screen, menu_rect};
use lukuloitsu_core::geometry::Rect;
use lukuloitsu_core::touch;

/// What the screen shows, in virtual units: the menus fill the arena, and
/// the game and practice add the keypad beside it on touch screens.
fn content_of(app: &App) -> Rect {
    match app.screen() {
        Screen::Game(_) | Screen::Practice(_) => touch::content_rect(app.touch_mode()),
        Screen::Title | Screen::Levels(_) | Screen::Progress | Screen::Badges(_) => menu_rect(),
    }
}

/// Draws the current screen.
pub fn draw(app: &App) {
    let content = content_of(app);
    let view = view::begin(content);
    let (progress_data, touch_mode) = (app.progress(), app.touch_mode());
    let time = app.time() as f32;
    match app.screen() {
        Screen::Title => title::draw(app.title(), progress_data, app.hardcore(), view, time),
        Screen::Levels(select) => levels::draw(select),
        Screen::Progress => progress::draw(progress_data, app.hardcore(), touch_mode),
        Screen::Badges(screen) => {
            badge_screen::draw(screen, progress_data, app.hardcore(), touch_mode)
        }
        Screen::Practice(screen) => {
            practice::draw(screen, time);
            if touch_mode {
                touch_render::draw(app.controls(), app.music_on(), false, false);
            }
        }
        Screen::Game(game) => {
            game_render::draw(&game.scene(), view);
            if touch_mode {
                touch_render::draw(app.controls(), app.music_on(), game.is_paused(), true);
            }
        }
    }
    view::mask_outside(view, content, BLACK);
    // The notice goes over whatever is showing; the camera set for that
    // screen is still in place, and the arena's middle is the same in all
    // of them.
    toast::draw(app.toasts(), ARENA_W);
}
