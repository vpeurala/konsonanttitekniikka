//! The little notice at the top of the screen when a badge is earned. One
//! is shown at a time, for a few seconds, and the rest wait their turn. It
//! never takes input or pauses anything, so it can't get in the way of
//! typing.

use std::collections::VecDeque;

use crate::badges::Badge;

/// How long a notice stays, fading included.
const SHOWN_SECONDS: f32 = 3.5;
/// How long the fading in and out each take.
const FADE_SECONDS: f32 = 0.3;

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
