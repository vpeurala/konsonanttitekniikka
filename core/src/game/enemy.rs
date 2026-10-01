//! Monsters and bosses: what they ask, how they move, where they may be.

use crate::memory::Memory;
use std::collections::VecDeque;

use glam::{Vec2, vec2};

use super::answer::Slot;
use super::metrics::text_width;
use super::rules::{
    BIRD_AIM_SECONDS, BIRD_AIM_SPEED, BIRD_ATTACK_SPEED_FACTOR, BIRD_ENERGY, BIRD_POINTS,
    BOSS_ENERGY_PER_HIT, BOSS_POINTS_PER_HIT, BOSS_RADIUS, BOSS_SPEED_FACTOR, CYCLOPS_ENERGY,
    CYCLOPS_GROWTH_FACTOR, CYCLOPS_POINTS, CYCLOPS_START_SPEED_FACTOR, ENEMY_RADIUS,
    MOULD_BASE_RADIUS, MOULD_ENERGY_PER_HIT, MOULD_GROWTH_DISTANCE, MOULD_POINTS_PER_HIT,
    MOULD_RADIUS_PER_NUMBER, MOULD_SPEED, MOULD_TAIL_SPEED, MOULD_TUBE_RADIUS, PLAYER_SPEED,
    SPEED_GROWTH, STAR_ENERGY, STAR_POINTS, START_SPEED, enemy_speed, shows_hint, speed_growth,
};
use crate::arena::{ARENA_H, ARENA_W};
use crate::long_numbers::Question;
use crate::memory::{self, Happened, Lesson};

pub const LABEL_FONT_SIZE: u16 = 22;
pub(super) const LABEL_PAD: f32 = 6.0;
/// Gap between an enemy's body and the center of its label.
const LABEL_GAP: f32 = 22.0;
pub const LABEL_HEIGHT: f32 = LABEL_FONT_SIZE as f32 + 2.0;
pub const HINT_FONT_SIZE: u16 = 20;
/// Extra room below the label for a hint.
pub const HINT_SPACE: f32 = 28.0;

/// The kinds of enemy, from the easiest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// One eye, shows a word, answered with its number.
    Cyclops,
    /// Star-like, shows a number, answered with its word.
    Star,
    /// Flies in from the edges and swoops at her; shows a number.
    Bird,
    /// Creeps slowly, growing a trail, with a number for each blob.
    Mould,
    Boss,
}

/// What answering an enemy gives her.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Worth {
    pub points: u32,
    /// Percent of full energy.
    pub energy: f32,
}

/// A boss's extra lives: the numbers it shows after its current one.
#[derive(Clone)]
pub struct BossLives {
    pub queue: VecDeque<Question>,
    pub total: usize,
    /// Seconds left of the white flash after being hit.
    pub hit_flash: f32,
    /// Seconds left before it can hurt the player again.
    pub harmless_for: f32,
}

/// The mould: a head that creeps toward her, dragging a slimy body behind
/// it that is one connected piece from the tail to the head. It has a
/// number for the head and one more for each stretch of the body, and each
/// answer takes one number and the oldest stretch away.
#[derive(Clone)]
pub struct Mould {
    /// The numbers after the one the head shows, which are the blobs'.
    pub queue: VecDeque<Question>,
    /// Numbers it has not grown yet, one for each blob it will still leave.
    pub pending: VecDeque<Question>,
    /// The bends of the body between the tail and the head, the oldest
    /// first: where the head was each time it gained a number.
    pub trail: Vec<Vec2>,
    /// The tail end of the body, which draws in toward `tail_goal` when
    /// the mould shrinks. `None` until it first moves.
    pub tail: Option<Vec2>,
    /// Where the tail end is headed: where the body started, until an
    /// answer shortens it to the next bend.
    pub tail_goal: Option<Vec2>,
    /// How far it has crept since the last bend.
    pub crept: f32,
    /// Seconds left before it can hurt her again.
    pub harmless_for: f32,
}

/// The point of the segment from `a` to `b` that is closest to `p`.
fn closest_on_segment(p: Vec2, a: Vec2, b: Vec2) -> Vec2 {
    let ab = b - a;
    let length_squared = ab.length_squared();
    if length_squared < 1e-6 {
        return a;
    }
    a + ab * ((p - a).dot(ab) / length_squared).clamp(0.0, 1.0)
}

/// What a bird is doing: taking aim, drifting slowly toward her, or flying
/// straight along `dir`, which was fixed when the attack began.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flight {
    Aiming { left: f32 },
    Attacking { dir: Vec2 },
}

/// Names an enemy for as long as it lives, whatever happens to the others.
/// Positions in the list shift when an enemy leaves, so anything that
/// has to find the same enemy later holds one of these instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyId(pub u32);

#[derive(Clone)]
pub struct Enemy {
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
    /// How well she knows the current pair (for a long number, its hardest
    /// pair) when it was shown here, from 0 (learned) to 1; sets how soon
    /// the hint comes.
    pub difficulty: f32,
    /// Seconds on screen; sets the speed.
    pub age: f32,
    /// Speed gained per second on screen, which the level sets.
    pub speed_growth: f32,
    /// How fast it is when it appears.
    pub start_speed: f32,
    /// Whether it ever shows its answer as a hint; not in hardcore mode.
    pub hints_enabled: bool,
    /// Seconds the current pair has been shown; sets the hint.
    pub shown_for: f32,
    /// Offsets the animation so enemies don't move in sync.
    pub phase: f32,
    pub boss: Option<BossLives>,
    /// What a bird is doing; `None` for everything that is not a bird.
    pub flight: Option<Flight>,
    /// The mould's trail and numbers; `None` for everything else.
    pub mould: Option<Mould>,
}

impl Enemy {
    /// `phase` offsets its animation from other enemies'.
    pub fn new(question: Question, shows_word: bool, difficulty: f32, phase: f32) -> Self {
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
            difficulty: 0.0,
            age: 0.0,
            speed_growth: SPEED_GROWTH,
            start_speed: START_SPEED,
            hints_enabled: true,
            shown_for: 0.0,
            phase,
            boss: None,
            flight: None,
            mould: None,
        };
        enemy.show(question, difficulty);
        enemy
    }

    /// A mould showing `question`, which will grow as it creeps by taking
    /// the numbers in `pending` one by one.
    pub fn mould(question: Question, pending: Vec<Question>, difficulty: f32, phase: f32) -> Self {
        let mut mould = Enemy::new(question, false, difficulty, phase);
        mould.radius = MOULD_BASE_RADIUS;
        mould.mould = Some(Mould {
            queue: VecDeque::new(),
            pending: pending.into(),
            trail: Vec::new(),
            tail: None,
            tail_goal: None,
            crept: 0.0,
            harmless_for: 0.0,
        });
        mould
    }

    /// A bird showing `question` (a number, answered with a word), which
    /// starts by taking aim. `phase` offsets its animation.
    pub fn bird(question: Question, difficulty: f32, phase: f32) -> Self {
        let mut bird = Enemy::new(question, false, difficulty, phase);
        bird.flight = Some(Flight::Aiming {
            left: BIRD_AIM_SECONDS,
        });
        bird
    }

    /// A boss showing `numbers`, one after another, long numbers split
    /// into their pairs if `split`. Its `difficulty` is for the
    /// first number.
    pub fn boss(numbers: &[Question], split: bool, difficulty: f32, phase: f32) -> Self {
        let mut boss = Enemy::new(numbers[0].clone(), false, difficulty, phase);
        boss.split = split;
        boss.show(numbers[0].clone(), difficulty);
        boss.radius = BOSS_RADIUS;
        boss.boss = Some(BossLives {
            queue: numbers[1..].iter().cloned().collect(),
            total: numbers.len(),
            hit_flash: 0.0,
            harmless_for: 0.0,
        });
        boss
    }

    /// How well she knows `question` for the hint's sake: the difficulty of
    /// its hardest pair.
    pub fn difficulty_of(question: &Question, memory: &Memory) -> f32 {
        question
            .pairs()
            .iter()
            .map(|p| memory.difficulty(p))
            .fold(0.0, f32::max)
    }

    /// Switches to showing `question`.
    pub fn show(&mut self, question: Question, difficulty: f32) {
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
        self.difficulty = difficulty;
        self.shown_for = 0.0;
    }

    pub fn is_boss(&self) -> bool {
        self.boss.is_some()
    }

    pub fn kind(&self) -> Kind {
        if self.is_boss() {
            Kind::Boss
        } else if self.mould.is_some() {
            Kind::Mould
        } else if self.flight.is_some() {
            Kind::Bird
        } else if self.shows_word {
            Kind::Cyclops
        } else {
            Kind::Star
        }
    }

    /// How many numbers a mould has: one for the head and one for each
    /// blob. Zero for everything else.
    pub fn mould_numbers(&self) -> usize {
        self.mould.as_ref().map_or(0, |m| 1 + m.queue.len())
    }

    /// Makes the mould's head as big as its numbers say.
    fn resize_mould(&mut self) {
        let extra = self.mould_numbers().saturating_sub(1) as f32;
        self.radius = MOULD_BASE_RADIUS + MOULD_RADIUS_PER_NUMBER * extra;
    }

    /// Moves the mould to `to`, `dt` seconds on. Every
    /// `MOULD_GROWTH_DISTANCE` it creeps it takes on the next number and
    /// the body grows a segment longer; the tail end draws in toward the
    /// body after answers have shortened it.
    pub fn creep_to(&mut self, to: Vec2, dt: f32) {
        let from = self.pos;
        self.pos = to;
        let Some(mould) = &mut self.mould else {
            return;
        };
        let tail = mould.tail.get_or_insert(from);
        let goal = *mould.tail_goal.get_or_insert(from);
        mould.crept += from.distance(to);
        while mould.crept >= MOULD_GROWTH_DISTANCE {
            let Some(next) = mould.pending.pop_front() else {
                mould.crept = 0.0;
                break;
            };
            mould.crept -= MOULD_GROWTH_DISTANCE;
            mould.trail.push(to);
            mould.queue.push_back(next);
        }
        let step = MOULD_TAIL_SPEED * dt;
        let gap = goal - *tail;
        *tail = if gap.length() <= step {
            goal
        } else {
            *tail + gap.normalize() * step
        };
        self.resize_mould();
    }

    /// The mould's body from the tail to the head, as points to join up
    /// with lines; just the head if it has not moved yet.
    pub fn mould_body(&self) -> Vec<Vec2> {
        let Some(mould) = &self.mould else {
            return Vec::new();
        };
        let mut body = Vec::with_capacity(mould.trail.len() + 2);
        body.extend(mould.tail);
        body.extend(mould.trail.iter().copied());
        body.push(self.pos);
        body
    }

    /// Circles that cover the mould's body (not the head), for steering
    /// around it.
    pub fn mould_body_circles(&self) -> Vec<(Vec2, f32)> {
        let body = self.mould_body();
        let mut circles = Vec::new();
        for pair in body.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let steps = (a.distance(b) / MOULD_TUBE_RADIUS).ceil().max(1.0) as usize;
            for k in 0..steps {
                let p = a.lerp(b, k as f32 / steps as f32);
                circles.push((p, MOULD_TUBE_RADIUS));
            }
        }
        circles
    }

    /// Where on the mould's body (not the head) a circle at `center` with
    /// `radius` touches it, if it does.
    pub fn mould_body_touch(&self, center: Vec2, radius: f32) -> Option<Vec2> {
        let body = self.mould_body();
        body.windows(2)
            .map(|pair| closest_on_segment(center, pair[0], pair[1]))
            .filter(|p| p.distance(center) < radius + MOULD_TUBE_RADIUS)
            .min_by(|a, b| a.distance(center).total_cmp(&b.distance(center)))
    }

    /// An answer shrinks the mould: the head moves on to its next number
    /// and the oldest blob goes. Returns the number the head shows next, or
    /// `None` if that was its last.
    pub fn shrink(&mut self) -> Option<Question> {
        let mould = self.mould.as_mut()?;
        let next = mould.queue.pop_front()?;
        if !mould.trail.is_empty() {
            // The tail draws in to where the oldest bend was.
            mould.tail_goal = Some(mould.trail.remove(0));
        }
        self.resize_mould();
        Some(next)
    }

    /// Whether a bird is in the middle of its attack, flying straight.
    pub fn is_attacking(&self) -> bool {
        matches!(self.flight, Some(Flight::Attacking { .. }))
    }

    /// Moves a bird on by `dt` seconds. Taking aim, it drifts slowly toward
    /// `player`; then it flies in a straight line toward where she was when
    /// the attack began, never correcting, until it is past the other edge
    /// of the arena and takes aim again.
    pub fn fly(&mut self, dt: f32, player: Vec2) {
        let speed = self.speed();
        match self.flight {
            Some(Flight::Aiming { left }) => {
                self.pos += (player - self.pos).normalize_or_zero() * speed * dt;
                let left = left - dt;
                self.flight = Some(if left > 0.0 {
                    Flight::Aiming { left }
                } else {
                    Flight::Attacking {
                        dir: (player - self.pos).normalize_or(Vec2::Y),
                    }
                });
            }
            Some(Flight::Attacking { dir }) => {
                self.pos += dir * speed * dt;
                let margin = self.radius * 2.0;
                let out = self.pos.x < -margin
                    || self.pos.x > ARENA_W + margin
                    || self.pos.y < -margin
                    || self.pos.y > ARENA_H + margin;
                if out {
                    self.flight = Some(Flight::Aiming {
                        left: BIRD_AIM_SECONDS,
                    });
                }
            }
            None => {}
        }
    }

    /// Sets how it speeds up for `level`: the level's growth, and a one-eyed
    /// monster's head start and quicker growth.
    pub fn set_level(&mut self, level: u32) {
        let growth = speed_growth(level);
        if self.kind() == Kind::Cyclops {
            self.start_speed = START_SPEED * CYCLOPS_START_SPEED_FACTOR;
            self.speed_growth = growth * CYCLOPS_GROWTH_FACTOR;
        } else {
            self.start_speed = START_SPEED;
            self.speed_growth = growth;
        }
    }

    /// What answering it once is worth. A boss is worth this for each of
    /// its numbers.
    pub fn worth(&self) -> Worth {
        match self.kind() {
            Kind::Cyclops => Worth {
                points: CYCLOPS_POINTS,
                energy: CYCLOPS_ENERGY,
            },
            Kind::Star => Worth {
                points: STAR_POINTS,
                energy: STAR_ENERGY,
            },
            Kind::Boss => Worth {
                points: BOSS_POINTS_PER_HIT,
                energy: BOSS_ENERGY_PER_HIT,
            },
            Kind::Bird => Worth {
                points: BIRD_POINTS,
                energy: BIRD_ENERGY,
            },
            Kind::Mould => Worth {
                points: MOULD_POINTS_PER_HIT,
                energy: MOULD_ENERGY_PER_HIT,
            },
        }
    }

    /// Whether it steers around the other monsters as well as the
    /// obstacles on its way to her.
    pub fn avoids_monsters(&self) -> bool {
        self.kind() == Kind::Star
    }

    pub fn speed(&self) -> f32 {
        if self.mould.is_some() {
            return MOULD_SPEED;
        }
        match self.flight {
            Some(Flight::Aiming { .. }) => return BIRD_AIM_SPEED,
            Some(Flight::Attacking { .. }) => return BIRD_ATTACK_SPEED_FACTOR * PLAYER_SPEED,
            None => {}
        }
        let speed = enemy_speed(self.age, self.start_speed, self.speed_growth);
        if self.is_boss() {
            speed * BOSS_SPEED_FACTOR
        } else {
            speed
        }
    }

    /// Distance from the enemy's center to the center of its label.
    pub fn label_offset(&self) -> f32 {
        if self.kind() == Kind::Bird {
            // Its claws hang well below the body, so the label does too.
            self.radius * 3.2
        } else {
            self.radius + LABEL_GAP
        }
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
        // cancels out of the fraction. Hints follow the full growth rate,
        // not the level's, so they still appear after the same time on the
        // slow early levels.
        let speed_fraction = enemy_speed(self.age, START_SPEED, SPEED_GROWTH) / PLAYER_SPEED;
        self.hints_enabled && shows_hint(self.difficulty, self.shown_for, speed_fraction)
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

    /// What there is to learn from a right answer at time `now`: one
    /// lesson for each pair of the question. A long number's time is
    /// shared out between its pairs.
    pub fn answer_lessons(&self, now: f64) -> Vec<Lesson> {
        let pairs = self.question.pairs();
        let seconds = self.shown_for / pairs.len() as f32;
        let what = Happened::Answered {
            seconds,
            with_hint: self.shows_hint(),
        };
        pairs
            .iter()
            .map(|&pair| Lesson {
                pair,
                what,
                at: now,
            })
            .collect()
    }

    /// Whether the answer, if given now, is a quick one: at once, without
    /// a hint. A long number's time is shared out between its pairs.
    pub fn is_quick(&self) -> bool {
        let seconds = self.shown_for / self.question.pairs().len() as f32;
        memory::is_quick(seconds, self.shows_hint())
    }

    /// What there is to learn from a miss at time `now`: one lesson for
    /// each pair of the question.
    pub fn miss_lessons(&self, now: f64) -> Vec<Lesson> {
        self.question
            .pairs()
            .iter()
            .map(|&pair| Lesson {
                pair,
                what: Happened::Missed,
                at: now,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pairs::PAIRS;

    fn enemy(pair: usize, shows_word: bool) -> Enemy {
        Enemy::new(Question::single(PAIRS[pair]), shows_word, 0.5, 0.0)
    }

    fn boss() -> Enemy {
        let numbers: Vec<Question> = PAIRS[..5].iter().map(|p| Question::single(*p)).collect();
        Enemy::boss(&numbers, false, 0.5, 0.0)
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
        e.show(Question::single(PAIRS[4]), 0.5);
        assert_eq!(e.shown_for, 0.0);
        assert_eq!(e.label, "4");
    }

    #[test]
    fn the_kind_follows_the_label_and_the_boss() {
        assert_eq!(enemy(3, true).kind(), Kind::Cyclops);
        assert_eq!(enemy(3, false).kind(), Kind::Star);
        assert_eq!(boss().kind(), Kind::Boss);
    }

    #[test]
    fn the_kinds_are_worth_different_amounts() {
        let worth = |e: Enemy| (e.worth().points, e.worth().energy);
        assert_eq!(worth(enemy(3, true)), (1, 10.0));
        assert_eq!(worth(enemy(3, false)), (2, 20.0));
        assert_eq!(worth(boss()), (1, 10.0), "for each of its numbers");
    }

    #[test]
    fn a_one_eyed_monster_starts_faster_and_speeds_up_faster_than_a_star() {
        let (mut cyclops, mut star) = (enemy(3, true), enemy(3, false));
        for level in [1, 10, 40] {
            cyclops.set_level(level);
            star.set_level(level);
            assert!(cyclops.speed() > star.speed());
            cyclops.age = 5.0;
            star.age = 5.0;
            let (gain_c, gain_s) = (
                cyclops.speed() - cyclops.start_speed,
                star.speed() - star.start_speed,
            );
            assert!(gain_c > gain_s, "level {level}");
            cyclops.age = 0.0;
            star.age = 0.0;
        }
    }

    #[test]
    fn a_stars_speed_is_the_same_as_before_the_kinds() {
        let mut star = enemy(3, false);
        star.set_level(21);
        assert_eq!(star.start_speed, START_SPEED);
        assert_eq!(star.speed_growth, SPEED_GROWTH);
    }

    #[test]
    fn only_stars_steer_around_other_monsters() {
        assert!(enemy(3, false).avoids_monsters());
        assert!(!enemy(3, true).avoids_monsters());
        assert!(!boss().avoids_monsters());
    }

    fn bird() -> Enemy {
        Enemy::bird(Question::single(PAIRS[3]), 0.5, 0.0)
    }

    #[test]
    fn a_bird_shows_a_number_and_is_worth_three_points_and_30_percent() {
        let bird = bird();
        assert_eq!(bird.kind(), Kind::Bird);
        assert!(!bird.shows_word, "a number, answered with a word");
        assert_eq!(bird.answer_slot(), Slot::Word);
        assert_eq!((bird.worth().points, bird.worth().energy), (3, 30.0));
        assert!(!bird.avoids_monsters());
    }

    #[test]
    fn a_bird_takes_aim_slowly_for_three_seconds_before_it_attacks() {
        let mut bird = bird();
        bird.pos = vec2(400.0, -40.0);
        let player = vec2(400.0, 300.0);
        let mut seconds = 0.0;
        while !bird.is_attacking() {
            bird.fly(0.01, player);
            seconds += 0.01;
            assert!(seconds < 3.1, "it should have attacked by now");
        }
        assert!((seconds - BIRD_AIM_SECONDS).abs() < 0.05, "{seconds}");
        // It only drifted: 3 seconds at the aiming speed.
        let drifted = bird.pos.y + 40.0;
        assert!((drifted - 3.0 * BIRD_AIM_SPEED).abs() < 2.0, "{drifted}");
        assert!((bird.speed() - 1.2 * PLAYER_SPEED).abs() < 1e-3);
    }

    #[test]
    fn a_bird_flies_straight_at_where_she_was_and_never_corrects() {
        let mut bird = bird();
        bird.pos = vec2(100.0, 100.0);
        bird.flight = Some(Flight::Aiming { left: 0.001 });
        let aimed_at = vec2(500.0, 300.0);
        bird.fly(0.01, aimed_at);
        let Some(Flight::Attacking { dir }) = bird.flight else {
            panic!("the attack should have begun");
        };
        assert!(dir.dot((aimed_at - vec2(100.0, 100.0)).normalize()) > 0.999);
        // She moves away; the bird keeps its line, through her old spot.
        let start = bird.pos;
        for _ in 0..100 {
            bird.fly(0.01, vec2(50.0, 550.0));
        }
        let moved = (bird.pos - start).normalize();
        assert!(moved.dot(dir) > 0.9999, "{moved} vs {dir}");
        assert_eq!(bird.flight, Some(Flight::Attacking { dir }));
    }

    #[test]
    fn a_bird_that_misses_takes_aim_again_past_the_other_edge() {
        let mut bird = bird();
        bird.pos = vec2(400.0, 100.0);
        bird.flight = Some(Flight::Attacking {
            dir: vec2(0.0, 1.0),
        });
        let mut steps = 0;
        while bird.is_attacking() {
            bird.fly(0.01, vec2(50.0, 50.0));
            steps += 1;
            assert!(steps < 1000, "it never left");
        }
        assert!(bird.pos.y > ARENA_H, "{}", bird.pos);
        assert_eq!(
            bird.flight,
            Some(Flight::Aiming {
                left: BIRD_AIM_SECONDS
            })
        );
    }

    /// A mould that will grow through `pending` more numbers.
    fn mould(pending: usize) -> Enemy {
        let questions = (0..pending)
            .map(|i| Question::single(PAIRS[10 + i]))
            .collect();
        Enemy::mould(Question::single(PAIRS[3]), questions, 0.5, 0.0)
    }

    #[test]
    fn a_new_mould_is_small_and_has_a_single_number() {
        let mould = mould(7);
        assert_eq!(mould.kind(), Kind::Mould);
        assert_eq!(mould.mould_numbers(), 1);
        assert!(!mould.shows_word, "a number, answered with a word");
        assert_eq!(mould.radius, MOULD_BASE_RADIUS);
        assert!(mould.mould.as_ref().unwrap().trail.is_empty());
        assert_eq!(mould.speed(), MOULD_SPEED);
        assert_eq!(
            (mould.worth().points, mould.worth().energy),
            (1, 10.0),
            "for each number"
        );
    }

    /// Moves the mould `distance` to the right, a pixel at a time.
    fn crawl(mould: &mut Enemy, distance: f32) {
        let steps = distance as usize;
        for _ in 0..steps {
            mould.creep_to(mould.pos + vec2(1.0, 0.0), 0.01);
        }
    }

    #[test]
    fn creeping_adds_a_number_and_a_bend_to_the_body_for_every_stretch() {
        let mut mould = mould(7);
        mould.pos = vec2(100.0, 100.0);
        crawl(&mut mould, MOULD_GROWTH_DISTANCE - 1.0);
        assert_eq!(mould.mould_numbers(), 1);
        crawl(&mut mould, 1.0);
        assert_eq!(mould.mould_numbers(), 2);
        assert_eq!(mould.mould.as_ref().unwrap().trail.len(), 1);
        assert!(mould.radius > MOULD_BASE_RADIUS, "the head grows too");
        crawl(&mut mould, 3.0 * MOULD_GROWTH_DISTANCE);
        assert_eq!(mould.mould_numbers(), 5);
        assert_eq!(mould.mould.as_ref().unwrap().trail.len(), 4);
    }

    #[test]
    fn the_body_is_one_connected_piece_from_the_tail_to_the_head() {
        let mut mould = mould(7);
        mould.pos = vec2(100.0, 100.0);
        assert!(mould.mould_body().len() == 1, "just the head to begin with");
        crawl(&mut mould, 3.5 * MOULD_GROWTH_DISTANCE);
        let body = mould.mould_body();
        // The tail where it started, a bend for each number gained, and
        // the head, no two neighbours farther apart than a stretch.
        assert_eq!(body.len(), 5);
        assert_eq!(body[0], vec2(100.0, 100.0));
        assert_eq!(body[4], mould.pos);
        for pair in body.windows(2) {
            assert!(pair[0].distance(pair[1]) <= MOULD_GROWTH_DISTANCE + 1.0);
        }
        // The circles for steering cover it end to end.
        let circles = mould.mould_body_circles();
        assert!(circles.len() >= 8, "{}", circles.len());
    }

    #[test]
    fn touching_any_part_of_the_body_is_found_but_not_beside_it() {
        let mut mould = mould(7);
        mould.pos = vec2(100.0, 100.0);
        crawl(&mut mould, 3.0 * MOULD_GROWTH_DISTANCE);
        // Halfway along the body, on it and just beside it.
        assert!(mould.mould_body_touch(vec2(160.0, 100.0), 16.0).is_some());
        assert!(
            mould
                .mould_body_touch(vec2(160.0, 100.0 + 16.0 + MOULD_TUBE_RADIUS - 1.0), 16.0)
                .is_some()
        );
        assert!(
            mould
                .mould_body_touch(vec2(160.0, 100.0 + 16.0 + MOULD_TUBE_RADIUS + 1.0), 16.0)
                .is_none()
        );
        // Beyond the tail.
        assert!(mould.mould_body_touch(vec2(40.0, 100.0), 16.0).is_none());
    }

    #[test]
    fn a_mould_stops_growing_when_it_has_no_more_numbers() {
        let mut mould = mould(3);
        mould.pos = vec2(0.0, 0.0);
        crawl(&mut mould, 6.0 * MOULD_GROWTH_DISTANCE);
        assert_eq!(mould.mould_numbers(), 4);
        assert_eq!(mould.mould.as_ref().unwrap().trail.len(), 3);
    }

    #[test]
    fn answering_shrinks_the_mould_until_one_number_is_left_which_kills_it() {
        let mut mould = mould(4);
        mould.pos = vec2(0.0, 0.0);
        crawl(&mut mould, 5.0 * MOULD_GROWTH_DISTANCE);
        assert_eq!(mould.mould_numbers(), 5);
        let big = mould.radius;
        let next = mould.shrink().expect("there are more numbers");
        assert_eq!(next, Question::single(PAIRS[10]));
        assert_eq!(mould.mould_numbers(), 4);
        assert!(mould.radius < big, "it gets smaller");
        assert_eq!(mould.mould.as_ref().unwrap().trail.len(), 3);
        for _ in 0..3 {
            assert!(mould.shrink().is_some());
        }
        assert_eq!(mould.mould_numbers(), 1);
        assert_eq!(mould.radius, MOULD_BASE_RADIUS);
        assert!(mould.mould.as_ref().unwrap().trail.is_empty());
        assert!(
            mould.shrink().is_none(),
            "nothing left: the next answer kills it"
        );
    }

    #[test]
    fn the_tail_draws_in_after_answers_instead_of_jumping() {
        let mut mould = mould(4);
        mould.pos = vec2(0.0, 0.0);
        crawl(&mut mould, 5.0 * MOULD_GROWTH_DISTANCE);
        let tail = mould.mould.as_ref().unwrap().tail.unwrap();
        mould.shrink();
        mould.creep_to(mould.pos, 0.01);
        let after = mould.mould.as_ref().unwrap().tail.unwrap();
        let bend = mould.mould.as_ref().unwrap().tail_goal.unwrap();
        assert!(after.distance(tail) > 0.0 && after.distance(tail) < MOULD_TAIL_SPEED * 0.011);
        assert!(after.distance(bend) < tail.distance(bend));
        // Given time it reaches the bend.
        for _ in 0..200 {
            mould.creep_to(mould.pos, 0.01);
        }
        assert_eq!(mould.mould.as_ref().unwrap().tail, Some(bend));
    }
}
