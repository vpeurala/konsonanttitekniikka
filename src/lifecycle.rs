//! Notices when the game was away: the app went to the background, the
//! phone was locked, or the window was minimized. The game uses it to
//! pause itself, so nothing happens while nobody is looking.

use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::miniquad::EventHandler;
use macroquad::prelude::*;

/// A gap between frames longer than this means the game wasn't running,
/// as on iOS, where no event says so.
const AWAY_GAP_SECONDS: f32 = 1.0;

pub struct Lifecycle {
    subscriber: usize,
}

struct Collector {
    minimized: bool,
}

impl EventHandler for Collector {
    fn update(&mut self) {}
    fn draw(&mut self) {}

    // On Android this comes when the app is paused.
    fn window_minimized_event(&mut self) {
        self.minimized = true;
    }
}

impl Lifecycle {
    pub fn new() -> Self {
        Lifecycle {
            subscriber: register_input_subscriber(),
        }
    }

    /// Whether the game was away since the previous call. Must be called
    /// every frame.
    pub fn was_away(&self) -> bool {
        let mut collector = Collector { minimized: false };
        repeat_all_miniquad_input(&mut collector, self.subscriber);
        collector.minimized || get_frame_time() > AWAY_GAP_SECONDS
    }
}
