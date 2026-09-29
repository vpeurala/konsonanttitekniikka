//! Monsters and bosses: what they ask, how they move, where they may be.

use std::collections::VecDeque;

use macroquad::prelude::{Vec2, vec2};

use super::answer::Slot;
use super::metrics::text_width;
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
    pub queue: VecDeque<Question>,
    pub total: usize,
    /// Seconds left of the white flash after being hit.
    pub hit_flash: f32,
    /// Seconds left before it can hurt the player again.
    pub harmless_for: f32,
}

/// Names an enemy for as long as it lives, whatever happens to the others.
/// Positions in the list shift when an enemy leaves, so anything that
/// has to find the same enemy later holds one of these instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EnemyId(pub u32);

pub(super) struct Enemy {
    /// Set by `Game::admit` when the enemy joins the game.
    pub id: EnemyId,
    pub pos: Vec2,
    pub radius: f32,
    pub question: Question,
    /// Whether a long number is shown split into its pairs ("20 1").
    pub split: bool,
    pub shows_word: bool,
    pub label: String,
    pub label_width: f32,
    /// What has to be typed to answer it.
    answer: String,
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
    pub fn new(question: Question, shows_word: bool, earlier_appearances: u32, phase: f32) -> Self {
        let mut enemy = Enemy {
            id: EnemyId(0),
            pos: Vec2::ZERO,
            radius: ENEMY_RADIUS,
            question: question.clone(),
            split: false,
            shows_word,
            label: String::new(),
            label_width: 0.0,
            answer: String::new(),
            hint: String::new(),
            hint_width: 0.0,
            earlier_appearances: 0,
            age: 0.0,
            shown_for: 0.0,
            phase,
            boss: None,
        };
        enemy.show(question, earlier_appearances);
        enemy
    }

    /// A boss showing `numbers`, one after another, long numbers split
    /// into their pairs if `split`. Its `earlier_appearances` are for the
    /// first number.
    pub fn boss(numbers: &[Question], split: bool, earlier_appearances: u32, phase: f32) -> Self {
        let mut boss = Enemy::new(numbers[0].clone(), false, 0, phase);
        boss.split = split;
        boss.show(numbers[0].clone(), earlier_appearances);
        boss.radius = BOSS_RADIUS;
        boss.boss = Some(BossLives {
            queue: numbers[1..].iter().cloned().collect(),
            total: numbers.len(),
            hit_flash: 0.0,
            harmless_for: 0.0,
        });
        boss
    }

    /// Switches to showing `question`.
    pub fn show(&mut self, question: Question, earlier_appearances: u32) {
        let (label, hint) = if self.shows_word {
            (question.words(), question.number(false))
        } else {
            (question.number(self.split), question.words())
        };
        self.label_width = text_width(&label, LABEL_FONT_SIZE) + 2.0 * LABEL_PAD;
        self.label = label;
        self.answer = if self.shows_word {
            question.number(false)
        } else {
            question.typed_words()
        };
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

    pub fn answer(&self) -> &str {
        &self.answer
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pairs::PAIRS;

    fn enemy(pair: usize, shows_word: bool) -> Enemy {
        Enemy::new(Question::single(PAIRS[pair]), shows_word, 0, 0.0)
    }

    fn boss() -> Enemy {
        let numbers: Vec<Question> = PAIRS[..5].iter().map(|p| Question::single(*p)).collect();
        Enemy::boss(&numbers, false, 0, 0.0)
    }

    fn label_rect(e: &Enemy) -> (f32, f32, f32, f32) {
        let center = e.pos + vec2(0.0, e.label_offset());
        (
            center.x - e.label_width / 2.0,
            center.y - LABEL_HEIGHT / 2.0,
            center.x + e.label_width / 2.0,
            center.y + LABEL_HEIGHT / 2.0,
        )
    }

    fn grid() -> impl Iterator<Item = Vec2> {
        (-2..=22).flat_map(|i| {
            (-2..=17).map(move |j| vec2(i as f32 * ARENA_W / 20.0, j as f32 * ARENA_H / 15.0))
        })
    }

    #[test]
    fn a_kept_enemys_label_is_inside_the_arena() {
        // The longest words, as words and as numbers, hint showing or not.
        let longest = PAIRS.iter().position(|p| p.word == "muumio").unwrap();
        for shows_word in [true, false] {
            for shown_for in [0.0, 10.0] {
                for pos in grid() {
                    let mut e = enemy(longest, shows_word);
                    e.shown_for = shown_for;
                    e.pos = pos;
                    e.keep_on_screen();
                    let (left, top, right, mut bottom) = label_rect(&e);
                    if e.shows_hint() {
                        bottom += HINT_SPACE;
                    }
                    assert!(left >= 0.0 && right <= ARENA_W, "{pos}: {left}..{right}");
                    assert!(top >= 0.0 && bottom <= ARENA_H, "{pos}: {top}..{bottom}");
                }
            }
        }
    }

    #[test]
    fn a_kept_enemys_body_is_inside_the_arena() {
        for pos in grid() {
            let mut e = enemy(3, true);
            e.pos = pos;
            e.keep_on_screen();
            assert!(
                e.pos.x >= e.radius && e.pos.x <= ARENA_W - e.radius,
                "{pos}"
            );
            assert!(e.pos.y >= e.radius, "{pos}");
        }
    }

    #[test]
    fn a_kept_boss_has_room_for_its_wings_and_lives() {
        for pos in grid() {
            let mut b = boss();
            b.pos = pos;
            b.keep_on_screen();
            assert!(b.pos.x - b.radius * 1.7 >= -1e-3, "{pos}");
            assert!(b.pos.x + b.radius * 1.7 <= ARENA_W + 1e-3, "{pos}");
            // The pips sit above its head.
            assert!(b.pos.y - b.radius * 1.6 - 8.0 >= 0.0, "{pos}");
        }
    }

    #[test]
    fn keeping_an_enemy_on_screen_changes_nothing_when_it_already_is() {
        let mut e = enemy(3, false);
        e.pos = vec2(400.0, 300.0);
        e.keep_on_screen();
        assert_eq!(e.pos, vec2(400.0, 300.0));
    }

    #[test]
    fn enemies_are_kept_apart_by_their_reach() {
        let e = enemy(3, false);
        assert!(e.reach() > e.radius, "the label is part of it");
        let longer = enemy(PAIRS.iter().position(|p| p.word == "muumio").unwrap(), true);
        assert!(longer.reach() > e.reach());
    }

    #[test]
    fn an_enemy_shows_its_word_or_its_number_and_takes_the_other_as_answer() {
        let word = enemy(3, true);
        assert_eq!(word.label, "LUU");
        assert_eq!(word.answer(), "3");
        assert_eq!(word.answer_slot(), Slot::Number);
        let number = enemy(3, false);
        assert_eq!(number.label, "3");
        assert_eq!(number.answer(), "luu");
        assert_eq!(number.answer_slot(), Slot::Word);
    }

    #[test]
    fn the_hint_shows_the_other_side_of_the_pair() {
        assert_eq!(enemy(3, true).hint, "= 3");
        assert_eq!(enemy(3, false).hint, "= LUU");
    }

    #[test]
    fn a_boss_is_bigger_and_slower_than_a_monster() {
        let (monster, boss) = (enemy(3, false), boss());
        assert!(boss.radius > monster.radius);
        assert!(boss.speed() < monster.speed());
        assert!(boss.is_boss() && !monster.is_boss());
    }

    #[test]
    fn showing_the_next_question_restarts_the_hint_timer() {
        let mut e = enemy(3, false);
        e.shown_for = 10.0;
        e.show(Question::single(PAIRS[4]), 0);
        assert_eq!(e.shown_for, 0.0);
        assert_eq!(e.label, "4");
    }
}
