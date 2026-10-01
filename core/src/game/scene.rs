//! What a game looks like right now, for the shell to draw. Everything in
//! it is borrowed or copied out of the game, so drawing can't change play.

use glam::Vec2;

use super::answer::Slot;
use super::display::{Banner, Feedback};
use super::enemy::Enemy;
use super::world::Spell;
use super::{Game, rules};
use crate::effects::Effects;
use crate::obstacles::Obstacle;
use crate::pairs::Pair;

/// One of the two places where what she has typed is shown.
#[derive(Debug, Clone, Copy)]
pub struct SlotView<'a> {
    pub slot: Slot,
    pub text: &'a str,
    /// Whether the text can no longer become any answer on screen.
    pub dead_end: bool,
}

/// A read-only picture of a `Game`.
pub struct Scene<'a> {
    pub player: Vec2,
    pub player_moving: bool,
    /// Where she is casting toward, if she is.
    pub casting: Option<Vec2>,
    /// Seconds played, not counting pauses, which animations follow.
    pub time: f32,
    pub enemies: &'a [Enemy],
    pub spells: &'a [Spell],
    pub portals: &'a [Vec2],
    pub obstacles: &'a [Obstacle],
    pub effects: &'a Effects,
    pub banner: Option<&'a Banner>,
    pub feedback: Option<&'a Feedback>,
    pub is_over: bool,
    pub is_paused: bool,
    /// Whether she plays with touch controls, which changes some texts.
    pub touch: bool,
    pub score: u32,
    pub level: u32,
    /// From 0 to `energy_cap`; `rules::FULL_ENERGY` is 100%.
    pub energy: f32,
    /// The most energy she can have on this level.
    pub energy_cap: f32,
    /// Points scored on this level, and how many the level needs.
    pub points: u32,
    pub points_needed: u32,
    pub boss_fight: bool,
    /// The pairs this level introduces.
    pub new_pairs: Vec<Pair>,
    /// Pairs from earlier levels, kept in the panel while there is room
    /// (`NEW_PAIRS_ROWS` in all).
    pub earlier_pairs: Vec<Pair>,
    /// The number slot and the word slot.
    pub slots: [SlotView<'a>; 2],
}

/// How many rows the "new pairs" panel has room for.
pub const NEW_PAIRS_ROWS: usize = 17;

impl Game {
    /// What to draw.
    pub fn scene(&self) -> Scene<'_> {
        let slot = |slot| SlotView {
            slot,
            text: self.typed.get(slot),
            dead_end: self.is_dead_end(slot),
        };
        Scene {
            player: self.player.pos,
            player_moving: self.player.moving,
            casting: self.display.cast.map(|(toward, _)| toward),
            time: self.play_time as f32,
            enemies: self.world.enemies(),
            spells: self.world.spells(),
            portals: self.world.portals(),
            obstacles: self.world.obstacles(),
            effects: &self.display.effects,
            banner: self.display.banner.as_ref(),
            feedback: self.display.feedback.as_ref(),
            is_over: self.is_over(),
            is_paused: self.paused,
            touch: self.touch,
            score: self.vitals.score(),
            level: self.level,
            energy: self.vitals.energy(),
            energy_cap: rules::energy_cap(self.level),
            points: self.stage.points,
            points_needed: rules::points_to_clear(self.level),
            boss_fight: self.stage.boss_fight,
            new_pairs: self.curriculum.new_pairs(),
            earlier_pairs: self.curriculum.earlier_pairs(NEW_PAIRS_ROWS),
            slots: [slot(Slot::Number), slot(Slot::Word)],
        }
    }
}
