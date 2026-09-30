//! The little notice at the top of the screen when a badge is earned. One
//! is shown at a time, for a few seconds, and the rest wait their turn. It
//! never takes input or pauses anything, so it can't get in the way of
//! typing.

use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::badge_art::draw_medal;
use crate::badges::Badge;
use crate::fonts::{self, Style};

/// How long a notice stays, fading included.
const SHOWN_SECONDS: f32 = 3.5;
/// How long the fading in and out each take.
const FADE_SECONDS: f32 = 0.3;

const WIDTH: f32 = 340.0;
const HEIGHT: f32 = 50.0;
const TOP: f32 = 8.0;

#[derive(Debug, Default)]
pub struct Toasts {
    /// The badges to announce, the one on show first.
    queue: VecDeque<&'static Badge>,
    /// How long the first has been on show.
    shown_for: f32,
}

impl Toasts {
    /// Announces `badge` once the earlier ones are done.
    pub fn push(&mut self, badge: &'static Badge) {
        self.queue.push_back(badge);
    }

    /// Moves time on by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        if self.queue.is_empty() {
            return;
        }
        self.shown_for += dt;
        if self.shown_for >= SHOWN_SECONDS {
            self.queue.pop_front();
            self.shown_for = 0.0;
        }
    }

    /// The badge on show and how visible it is, from 0 to 1.
    pub fn current(&self) -> Option<(&'static Badge, f32)> {
        let badge = *self.queue.front()?;
        let fade_in = self.shown_for / FADE_SECONDS;
        let fade_out = (SHOWN_SECONDS - self.shown_for) / FADE_SECONDS;
        Some((badge, fade_in.min(fade_out).clamp(0.0, 1.0)))
    }

    /// Draws the notice, if there is one, at the top of the arena.
    pub fn draw(&self, arena_width: f32) {
        let Some((badge, alpha)) = self.current() else {
            return;
        };
        let x = (arena_width - WIDTH) / 2.0;
        draw_rectangle(
            x,
            TOP,
            WIDTH,
            HEIGHT,
            Color::new(0.1, 0.1, 0.16, 0.92 * alpha),
        );
        draw_rectangle_lines(
            x,
            TOP,
            WIDTH,
            HEIGHT,
            2.0,
            Color::new(0.96, 0.77, 0.2, alpha),
        );
        draw_medal(vec2(x + 30.0, TOP + 19.0), 15.0, badge, true, alpha);
        let text_x = x + 62.0;
        let fade = |color: Color| Color::new(color.r, color.g, color.b, alpha);
        fonts::draw(
            "Uusi kunniamerkki!",
            text_x,
            TOP + 20.0,
            16,
            fade(GOLD),
            Style::Body,
        );
        fonts::draw(badge.name, text_x, TOP + 41.0, 22, fade(WHITE), Style::Bold);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::badges::BADGES;

    #[test]
    fn nothing_shows_until_a_badge_is_pushed() {
        let mut toasts = Toasts::default();
        toasts.update(1.0);
        assert!(toasts.current().is_none());
    }

    #[test]
    fn a_notice_fades_in_stays_and_fades_out() {
        let alpha = |toasts: &Toasts| toasts.current().map(|(_, a)| a);
        let mut toasts = Toasts::default();
        toasts.push(&BADGES[0]);
        assert_eq!(alpha(&toasts), Some(0.0));
        toasts.update(FADE_SECONDS / 2.0);
        assert!((alpha(&toasts).unwrap() - 0.5).abs() < 1e-4);
        toasts.update(SHOWN_SECONDS / 2.0);
        assert_eq!(alpha(&toasts), Some(1.0));
        // Half way through the fading out.
        toasts.update(SHOWN_SECONDS / 2.0 - FADE_SECONDS / 2.0 - FADE_SECONDS / 2.0);
        assert!((alpha(&toasts).unwrap() - 0.5).abs() < 1e-3);
        toasts.update(1.0);
        assert!(toasts.current().is_none());
    }

    #[test]
    fn notices_come_one_at_a_time_in_order() {
        let mut toasts = Toasts::default();
        toasts.push(&BADGES[0]);
        toasts.push(&BADGES[1]);
        assert_eq!(toasts.current().map(|(b, _)| b.id), Some(BADGES[0].id));
        toasts.update(SHOWN_SECONDS);
        assert_eq!(toasts.current().map(|(b, _)| b.id), Some(BADGES[1].id));
        // The second starts from nothing, not part way through.
        assert_eq!(toasts.current().map(|(_, a)| a), Some(0.0));
        toasts.update(SHOWN_SECONDS);
        assert!(toasts.current().is_none());
    }
}
