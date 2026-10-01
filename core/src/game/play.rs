//! What happens in a game when she types, when spells land and monsters
//! reach her: the world's reports turned into changes to her state, sounds
//! and things to show. Each step returns the `Outputs` it caused.

use super::answer::{InputOutcome, Slot, resolve_input};
use super::display::Tone;
use super::enemy::{Enemy, EnemyId};
use super::rules::{
    COLLISION_PALETTE, ENEMY_RADIUS, KILL_PALETTE, LEVEL_BREAK_SECONDS, PORTAL_PALETTE,
    SPELL_PALETTE, hit_reward, points_to_clear, stars_for,
};
use super::spawn::SpawnContext;
use super::stage::Stage;
use super::world::{ContactKind, Hit, HitOutcome, ImpactTarget, SpellEvent};
use super::{Game, GameEvent, Known, Outputs};
use crate::key::Key;
use crate::long_numbers::{self, Question};
use crate::memory::{Lesson, Memory};
use crate::pairs;
use crate::sfx::Sfx;

impl Game {
    /// A key typed: Backspace starts over by emptying both slots, an
    /// answer character goes to its slot.
    pub(super) fn handle_key(&mut self, key: Key, now: f64, memory: &mut Known) -> Outputs {
        match key {
            Key::Backspace => {
                self.typed.clear();
                Outputs::default()
            }
            Key::Char(c) if pairs::is_answer_char(c) => {
                self.type_into(Slot::of_char(c), c, now, memory)
            }
            Key::Char(_) => Outputs::default(),
        }
    }

    /// A wrong key or a hit ends the combo and the level's clean record.
    pub(super) fn mistake(&mut self) {
        self.vitals = self.vitals.mistaken();
        self.stage = self.stage.mistaken();
    }

    /// The enemies this slot can answer.
    fn candidates(&self, slot: Slot) -> impl Iterator<Item = &Enemy> + Clone {
        self.world
            .enemies()
            .iter()
            .filter(move |e| e.answer_slot() == slot)
    }

    /// What the slot's text means for the enemies it can answer.
    fn outcome(&self, slot: Slot) -> InputOutcome {
        resolve_input(
            self.typed.get(slot),
            self.candidates(slot).map(Enemy::answer),
        )
    }

    pub(super) fn is_dead_end(&self, slot: Slot) -> bool {
        self.outcome(slot) == InputOutcome::DeadEnd
    }

    /// A dead-end slot is only shown in red at first. Typing past it costs
    /// energy, which leaves room to fix a typo with backspace. Both slots
    /// are then emptied and the key starts afresh, so a leftover letter
    /// can't spoil the word she is really typing.
    fn type_into(&mut self, slot: Slot, c: char, now: f64, memory: &mut Known) -> Outputs {
        let mut out = Outputs::default();
        if self.is_dead_end(slot) {
            self.mistake();
            self.vitals = self.vitals.after_wrong_key();
            out.sfx.push(Sfx::Wrong);
            self.display.say(
                format!("Väärin: {}", self.typed.get(slot).to_uppercase()),
                Tone::Wrong,
            );
            self.typed.clear();
        }

        self.typed.get_mut(slot).push(c);
        out.sfx.push(Sfx::Type);
        if let InputOutcome::Hit(hits) = self.outcome(slot) {
            let ids: Vec<EnemyId> = self
                .candidates(slot)
                .enumerate()
                .filter(|(i, _)| hits.contains(i))
                .map(|(_, e)| e.id)
                .collect();
            let player = self.player.pos;
            let made: Vec<Hit> = ids
                .into_iter()
                .filter_map(|id| self.world.hit(id, player, memory))
                .collect();
            let Some(first) = made.first().map(|h| h.answered.question.clone()) else {
                return out;
            };
            for hit in &made {
                out.extend(self.learn_from(hit, now, memory));
            }
            self.vitals = self.vitals.rewarded(hits.len(), hit_reward(self.level));
            self.show_question(&first, Tone::Right);
            self.typed.get_mut(slot).clear();
            out.extend(self.add_points(hits.len() as u32, now, memory));
        }
        out
    }

    /// Learns `lessons` into `memory`, and reports them in `out`.
    fn learn(lessons: Vec<Lesson>, memory: &mut Known, out: &mut Outputs) {
        for lesson in &lessons {
            memory.to_mut().learn(lesson);
        }
        out.lessons.extend(lessons);
    }

    /// What a right answer means for her: she learns the pair, the combo
    /// grows, and a spell is cast.
    pub(super) fn learn_from(&mut self, hit: &Hit, now: f64, memory: &mut Known) -> Outputs {
        let mut out = Outputs::default();
        Self::learn(hit.answered.answer_lessons(now), memory, &mut out);
        self.vitals = self.vitals.answered();
        out.events.push(GameEvent::Answered {
            quick: hit.answered.is_quick(),
            long: hit.answered.question.is_long(),
            combo: self.vitals.combo(),
        });
        if let HitOutcome::Defeated { boss: false } = hit.outcome {
            out.events.push(GameEvent::MonsterDefeated);
        }
        out.sfx.push(Sfx::Cast);
        self.display.cast_toward(hit.answered.pos);
        out
    }

    pub(super) fn add_points(&mut self, points: u32, now: f64, memory: &Memory) -> Outputs {
        self.vitals = self.vitals.scored(points);
        let (stage, boss_due) = self.stage.scored(points, points_to_clear(self.level));
        self.stage = stage;
        if boss_due {
            self.summon_boss(now, memory)
        } else {
            Outputs::default()
        }
    }

    pub(super) fn spawn_enemy(&mut self, now: f64, memory: &Memory) {
        let ctx = SpawnContext {
            curriculum: &self.curriculum,
            memory,
            now,
            player: self.player.pos,
            level: self.level,
        };
        if let Some(portal) = self.world.spawn_enemy(&ctx, &mut self.rng) {
            self.display.effects.explode(portal, 24.0, &PORTAL_PALETTE);
        }
    }

    fn summon_boss(&mut self, now: f64, memory: &Memory) -> Outputs {
        let mut out = Outputs::default();
        let ctx = SpawnContext {
            curriculum: &self.curriculum,
            memory,
            now,
            player: self.player.pos,
            level: self.level,
        };
        let Some(summoned) = self.world.summon_boss(&ctx, &mut self.rng) else {
            return out;
        };
        if let Some(portal) = summoned.portal {
            self.display.effects.explode(portal, 24.0, &PORTAL_PALETTE);
        }
        self.stage = self.stage.boss_summoned();
        out.sfx.push(Sfx::Boss);
        let (count, long) = (summoned.count, summoned.long);
        let subtitle = if long > 0 && self.level == long_numbers::first_long_level() {
            format!("Tarvitaan {count} osumaa. Viimeinen on pitkä luku!")
        } else if long > 0 {
            format!("Tarvitaan {count} osumaa, niistä {long} pitkää lukua")
        } else {
            format!("Tarvitaan {count} osumaa")
        };
        self.display
            .announce("Pomo saapuu!".to_owned(), subtitle, Tone::Boss, None);
        out
    }

    /// Flies the spells on and shows what they do. A boss that falls
    /// finishes the level.
    pub(super) fn update_spells(&mut self, dt: f32) -> Outputs {
        let mut out = Outputs::default();
        let mut boss_fell_at = None;
        for event in self.world.advance_spells(dt, self.player.pos) {
            match event {
                SpellEvent::Trail(pos) => self.display.effects.trail(pos, &SPELL_PALETTE),
                SpellEvent::Impact(impact) => {
                    let pos = impact.pos;
                    out.sfx.push(Sfx::Explode);
                    self.display
                        .effects
                        .explode(pos, ENEMY_RADIUS * 0.5, &SPELL_PALETTE);
                    if let ImpactTarget::Doomed { radius, boss } = impact.target {
                        self.display.effects.explode(pos, radius, &KILL_PALETTE);
                        if boss {
                            self.display.effects.explode_around(
                                pos,
                                30.0,
                                3,
                                radius,
                                &KILL_PALETTE,
                            );
                            boss_fell_at = Some(pos);
                        }
                    }
                }
            }
        }
        if let Some(pos) = boss_fell_at
            && !self.is_over()
        {
            out.extend(self.complete_level(pos));
        }
        out
    }

    /// Moves the monsters on and deals with those that reach her.
    pub(super) fn move_enemies(&mut self, dt: f32, now: f64, memory: &mut Known) -> Outputs {
        let mut out = Outputs::default();
        let contacts = self
            .world
            .advance_enemies(dt, self.player.pos, &mut self.rng);
        let hurt = !contacts.is_empty();
        for contact in contacts {
            let enemy = contact.enemy;
            Self::learn(enemy.miss_lessons(now), memory, &mut out);
            self.vitals = self.vitals.hurt();
            match contact.kind {
                ContactKind::Boss { at } => {
                    out.sfx.push(Sfx::Hurt);
                    self.display
                        .effects
                        .explode(at, ENEMY_RADIUS, &COLLISION_PALETTE);
                }
                ContactKind::Ordinary => {
                    out.sfx.extend([Sfx::Explode, Sfx::Hurt]);
                    self.display
                        .effects
                        .explode(enemy.pos, enemy.radius, &COLLISION_PALETTE);
                    self.show_question(&enemy.question, Tone::Missed);
                }
            }
        }
        if hurt {
            self.mistake();
            self.typed.clear();
        }
        out
    }

    /// Starts the next level with a lightning strike at `pos`, where the
    /// boss fell. Any monsters left on screen explode with it, so the next
    /// level starts from a clear slate.
    pub(super) fn complete_level(&mut self, pos: glam::Vec2) -> Outputs {
        let mut out = Outputs::default();
        let stars = stars_for(self.vitals.energy_fraction());
        if self.stage.flawless {
            out.events.push(GameEvent::FlawlessLevel);
        }
        out.events.push(GameEvent::LevelCompleted {
            level: self.level,
            stars,
        });
        self.level += 1;
        self.stage = Stage::default();
        self.spawn_timer = LEVEL_BREAK_SECONDS;
        for enemy in self.world.next_level(self.level) {
            self.display
                .effects
                .explode(enemy.pos, enemy.radius, &KILL_PALETTE);
        }
        self.typed.clear();
        self.display.effects.lightning(pos);
        out.sfx.extend([Sfx::Explode, Sfx::Thunder, Sfx::LevelUp]);
        let new_pairs = self.curriculum.next_level();
        self.player = self.player.pushed_out(self.world.obstacles());
        let subtitle = if new_pairs > 0 {
            format!("Hienoa! {new_pairs} uutta paria")
        } else {
            "Hienoa!".to_owned()
        };
        self.display.announce(
            format!("Taso {}!", self.level),
            subtitle,
            Tone::Celebrate,
            Some(stars),
        );
        out
    }

    fn show_question(&mut self, question: &Question, tone: Tone) {
        self.display.say(
            format!("{} = {}", question.words(), question.number(false)),
            tone,
        );
    }
}
