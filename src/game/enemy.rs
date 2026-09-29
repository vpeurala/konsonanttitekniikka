//! Monsters and bosses: what they ask, how they move, where they may be.

use macroquad::prelude::{Vec2, vec2};

use super::TextWidth;
use super::answer::Slot;
use super::rules::{
    BOSS_RADIUS, BOSS_SPEED_FACTOR, ENEMY_RADIUS, PLAYER_SPEED, enemy_speed, shows_hint,
};
use crate::long_numbers::Question;
use crate::memory::Memory;
use crate::view::{ARENA_H, ARENA_W};

pub(super) const LABEL_FONT_SIZE: u16 = 22;
pub(super) const LABEL_PAD: f32 = 6.0;
/// Gap between an enemy's body and the center of its label.
const LABEL_GAP: f32 = 22.0;
pub(super) const LABEL_HEIGHT: f32 = LABEL_FONT_SIZE as f32 + 2.0;
pub(super) const HINT_FONT_SIZE: u16 = 20;
/// Extra room below the label for a hint.
pub(super) const HINT_SPACE: f32 = 28.0;

/// A boss's extra lives: the numbers it shows after its current one.
pub(super) struct BossLives {
    pub queue: Vec<Question>,
    pub total: usize,
    /// Seconds left of the white flash after being hit.
    pub hit_flash: f32,
    /// Seconds left before it can hurt the player again.
    pub harmless_for: f32,
}

pub(super) struct Enemy {
    pub pos: Vec2,
    pub radius: f32,
    pub question: Question,
    /// Whether a long number is shown split into its pairs ("20 1").
    pub split: bool,
    pub shows_word: bool,
    pub label: String,
    pub label_width: f32,
    /// The answer, as shown in a hint.
    pub hint: String,
    pub hint_width: f32,
    /// How many times the current pair (or, for a long number, any long
    /// number) had appeared in the game before it was shown here.
    pub earlier_appearances: u32,
    /// Seconds on screen; sets the speed.
    pub age: f32,
    /// Seconds the current pair has been shown; sets the hint.
    pub shown_for: f32,
    /// Offsets the animation so enemies don't move in sync.
    pub phase: f32,
    pub boss: Option<BossLives>,
}

impl Enemy {
    /// `phase` offsets its animation from other enemies'.
    pub fn new(
        question: Question,
        shows_word: bool,
        earlier_appearances: u32,
        phase: f32,
        text_width: TextWidth,
    ) -> Self {
        let mut enemy = Enemy {
            pos: Vec2::ZERO,
            radius: ENEMY_RADIUS,
            question: question.clone(),
            split: false,
            shows_word,
            label: String::new(),
            label_width: 0.0,
            hint: String::new(),
            hint_width: 0.0,
            earlier_appearances: 0,
            age: 0.0,
            shown_for: 0.0,
            phase,
            boss: None,
        };
        enemy.show(question, earlier_appearances, text_width);
        enemy
    }

    /// A boss showing `numbers`, one after another, long numbers split
    /// into their pairs if `split`. Its `earlier_appearances` are for the
    /// first number.
    pub fn boss(
        numbers: &[Question],
        split: bool,
        earlier_appearances: u32,
        phase: f32,
        text_width: TextWidth,
    ) -> Self {
        let mut boss = Enemy::new(numbers[0].clone(), false, 0, phase, text_width);
        boss.split = split;
        boss.show(numbers[0].clone(), earlier_appearances, text_width);
        boss.radius = BOSS_RADIUS;
        boss.boss = Some(BossLives {
            queue: numbers[1..].to_vec(),
            total: numbers.len(),
            hit_flash: 0.0,
            harmless_for: 0.0,
        });
        boss
    }

    /// Switches to showing `question`.
    pub fn show(&mut self, question: Question, earlier_appearances: u32, text_width: TextWidth) {
        let (label, hint) = if self.shows_word {
            (question.words(), question.number(false))
        } else {
            (question.number(self.split), question.words())
        };
        self.label_width = text_width(&label, LABEL_FONT_SIZE) + 2.0 * LABEL_PAD;
        self.label = label;
        self.hint = format!("= {hint}");
        self.hint_width = text_width(&self.hint, HINT_FONT_SIZE);
        self.question = question;
        self.earlier_appearances = earlier_appearances;
        self.shown_for = 0.0;
    }

    pub fn is_boss(&self) -> bool {
        self.boss.is_some()
    }

    pub fn speed(&self) -> f32 {
        let speed = enemy_speed(self.age);
        if self.is_boss() {
            speed * BOSS_SPEED_FACTOR
        } else {
            speed
        }
    }

    /// Distance from the enemy's center to the center of its label.
    pub fn label_offset(&self) -> f32 {
        self.radius + LABEL_GAP
    }

    /// The radius of the circle enemies keep clear of each other: wide
    /// enough to cover both the body and the label hanging below it.
    pub fn reach(&self) -> f32 {
        let hint_width = if self.shows_hint() {
            self.hint_width
        } else {
            0.0
        };
        (self.radius * 1.6 + self.hint_space() / 2.0)
            .max(self.label_width / 2.0 + 4.0)
            .max(hint_width / 2.0 + 4.0)
    }

    pub fn shows_hint(&self) -> bool {
        // The boss's speed factor applies to its top speed as well, so it
        // cancels out of the fraction.
        let speed_fraction = enemy_speed(self.age) / PLAYER_SPEED;
        shows_hint(self.earlier_appearances, self.shown_for, speed_fraction)
    }

    fn hint_space(&self) -> f32 {
        if self.shows_hint() { HINT_SPACE } else { 0.0 }
    }

    /// Moves the enemy so its body, label and any boss pips are on screen.
    pub fn keep_on_screen(&mut self) {
        let top = self.radius * 1.6 + 16.0;
        let bottom = self.label_offset() + LABEL_HEIGHT / 2.0 + self.hint_space();
        // A boss's wings reach well past its body.
        let body_half_width = if self.is_boss() {
            self.radius * 1.7
        } else {
            self.radius
        };
        let half_width = body_half_width.max(self.label_width / 2.0);
        self.pos.x = self
            .pos
            .x
            .clamp(half_width, (ARENA_W - half_width).max(half_width));
        self.pos.y = self.pos.y.clamp(top, (ARENA_H - bottom).max(top));
    }

    /// The point `reach` is measured from, between the body and the label.
    pub fn reach_center(&self) -> Vec2 {
        self.pos + vec2(0.0, (self.label_offset() + self.hint_space()) / 2.0)
    }

    pub fn answer_slot(&self) -> Slot {
        if self.shows_word {
            Slot::Number
        } else {
            Slot::Word
        }
    }

    pub fn answer(&self) -> String {
        if self.shows_word {
            self.question.number(false)
        } else {
            self.question.typed_words()
        }
    }

    /// Records a right answer for each pair of the question, at time
    /// `now`. A long number's time is shared out between its pairs.
    pub fn record_answer(&self, memory: &mut Memory, now: f64) {
        let pairs = self.question.pairs();
        let seconds = self.shown_for / pairs.len() as f32;
        for &pair in pairs {
            memory.record_answer(pair, seconds, self.shows_hint(), now);
        }
    }

    /// Records a miss for each pair of the question, at time `now`.
    pub fn record_miss(&self, memory: &mut Memory, now: f64) {
        for &pair in self.question.pairs() {
            memory.record_miss(pair, now);
        }
    }
}
