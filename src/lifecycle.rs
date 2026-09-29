//! Notices when the game was away: the app went to the background, the
//! phone was locked, or the window was minimized. The game uses it to
//! pause itself, so nothing happens while nobody is looking.

use macroquad::input::utils::{register_input_subscriber, repeat_all_miniquad_input};
use macroquad::miniquad::EventHandler;

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

    /// Whether the window was minimized since the previous call. Must be
    /// called every frame.
    pub fn minimized(&self) -> bool {
        let mut collector = Collector { minimized: false };
        repeat_all_miniquad_input(&mut collector, self.subscriber);
        collector.minimized
    }
}

/// Whether the game was away: the window was `minimized`, or the last
/// frame took `dt` seconds, far longer than the game runs a frame.
pub fn was_away(minimized: bool, dt: f32) -> bool {
    minimized || dt > AWAY_GAP_SECONDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_minimized_window_was_away() {
        assert!(was_away(true, 0.016));
    }

    #[test]
    fn a_long_gap_between_frames_was_away() {
        assert!(was_away(false, AWAY_GAP_SECONDS + 0.1));
    }

    #[test]
    fn ordinary_frames_were_not() {
        assert!(!was_away(false, 0.016));
        assert!(!was_away(false, 0.2));
    }
}
