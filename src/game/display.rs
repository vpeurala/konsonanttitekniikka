//! What the game shows without it changing play: sparks and lightning, the
//! banner and feedback messages, and her raised hand. `Game` tells this
//! things to show; time moves it on; `render` reads it. Nothing here is
//! ever read by the rules.

use macroquad::prelude::{Color, Vec2};

use super::rules::{BANNER_SECONDS, CAST_SECONDS, FEEDBACK_SECONDS};
use crate::effects::Effects;

/// A short message under the arena, like "KUU = 2".
pub(super) struct Feedback {
    pub text: String,
    pub color: Color,
    pub seconds_left: f32,
}

/// A big message in the middle of the screen, like "Taso 2!".
pub(super) struct Banner {
    pub title: String,
    pub subtitle: String,
    pub color: Color,
    /// Stars earned, shown under a level-up banner.
    pub stars: Option<u8>,
    pub seconds_left: f32,
}

#[derive(Default)]
pub(super) struct Display {
    pub effects: Effects,
    pub banner: Option<Banner>,
    pub feedback: Option<Feedback>,
    /// Where she is casting toward, and for how much longer.
    pub cast: Option<(Vec2, f32)>,
}

impl Display {
    /// Shows a big message for a while.
    pub fn announce(&mut self, title: String, subtitle: String, color: Color, stars: Option<u8>) {
        self.banner = Some(Banner {
            title,
            subtitle,
            color,
            stars,
            seconds_left: BANNER_SECONDS,
        });
    }

    /// Shows a short message for a while.
    pub fn say(&mut self, text: String, color: Color) {
        self.feedback = Some(Feedback {
            text,
            color,
            seconds_left: FEEDBACK_SECONDS,
        });
    }

    /// Raises her hand toward `target`.
    pub fn cast_toward(&mut self, target: Vec2) {
        self.cast = Some((target, CAST_SECONDS));
    }

    /// Moves the sparks and her hand on. These carry on even when the game
    /// is over.
    pub fn update_motion(&mut self, dt: f32) {
        self.effects.update(dt);
        if let Some((_, seconds_left)) = &mut self.cast {
            *seconds_left -= dt;
            if *seconds_left <= 0.0 {
                self.cast = None;
            }
        }
    }

    /// Counts down the messages' time.
    pub fn update_messages(&mut self, dt: f32) {
        if let Some(banner) = &mut self.banner {
            banner.seconds_left -= dt;
            if banner.seconds_left <= 0.0 {
                self.banner = None;
            }
        }
        if let Some(feedback) = &mut self.feedback {
            feedback.seconds_left -= dt;
            if feedback.seconds_left <= 0.0 {
                self.feedback = None;
            }
        }
    }
}
