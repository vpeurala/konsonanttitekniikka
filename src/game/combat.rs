//! Spells, collisions and the movement of monsters.

use crate::memory::Memory;
use macroquad::prelude::{Vec2, YELLOW};

use super::enemy::{Enemy, EnemyId};
use super::rules::*;
use super::{Game, GameEvent};
use crate::audio::Sfx;
use crate::obstacles::{push_out, steer};
use crate::sprites::girl_hand;

/// What a spell is flying toward.
pub(super) enum SpellTarget {
    /// An enemy that is already out of play and explodes when hit.
    Doomed(Enemy),
    /// The boss, which is still in play and only loses a life.
    Boss,
}

/// A magic bolt flying toward an answered enemy.
pub(super) struct Spell {
    pub pos: Vec2,
    pub target: SpellTarget,
    /// Where the target was last seen, in case the boss dies first.
    pub target_pos: Vec2,
}

impl Spell {
    /// The enemy that is out of play and waiting for this spell.
    pub fn doomed(&self) -> Option<&Enemy> {
        match &self.target {
            SpellTarget::Doomed(enemy) => Some(enemy),
            SpellTarget::Boss => None,
        }
    }
}

impl Game {
    /// Casts a spell at the enemy at `index`. Ordinary enemies and a boss
    /// on its last life leave play at once; a boss with lives left moves on
    /// to its next pair.
    pub(super) fn hit_enemy(&mut self, id: EnemyId, memory: &mut Memory) {
        let Some(index) = self.enemies.iter().position(|e| e.id == id) else {
            return;
        };
        self.enemies[index].record_answer(memory, self.now);
        self.combo += 1;
        self.out.events.push(GameEvent::Answered {
            quick: self.enemies[index].is_quick(),
            long: self.enemies[index].question.is_long(),
            combo: self.combo,
        });
        let next = self.enemies[index]
            .boss
            .as_mut()
            .and_then(|lives| lives.queue.pop_front());
        match next {
            Some(question) => {
                let target_pos = self.enemies[index].pos;
                let earlier = self.count_appearance(&question);
                self.enemies[index].show(question, earlier);
                self.cast_spell(SpellTarget::Boss, target_pos);
            }
            None => {
                let enemy = self.enemies.remove(index);
                if !enemy.is_boss() {
                    self.out.events.push(GameEvent::MonsterDefeated);
                }
                let target_pos = enemy.pos;
                self.cast_spell(SpellTarget::Doomed(enemy), target_pos);
            }
        }
    }

    fn cast_spell(&mut self, target: SpellTarget, target_pos: Vec2) {
        self.out.sfx.push(Sfx::Cast);
        self.display.cast_toward(target_pos);
        self.spells.push(Spell {
            pos: girl_hand(self.player, target_pos),
            target,
            target_pos,
        });
    }

    pub(super) fn update_spells(&mut self, dt: f32) {
        for enemy in &mut self.enemies {
            if let Some(lives) = &mut enemy.boss {
                lives.hit_flash = (lives.hit_flash - dt).max(0.0);
            }
        }

        let step = SPELL_SPEED * dt;
        let player = self.player;
        let effects = &mut self.display.effects;
        let sfx = &mut self.out.sfx;
        let enemies = &mut self.enemies;
        let mut boss_fell_at = None;
        self.spells.retain_mut(|spell| {
            // A spell at the boss follows it around.
            let boss = enemies.iter_mut().find(|e| e.is_boss());
            if let (SpellTarget::Boss, Some(boss)) = (&spell.target, &boss) {
                spell.target_pos = boss.pos;
            }

            let to_target = spell.target_pos - spell.pos;
            if to_target.length() > step {
                spell.pos += to_target.normalize() * step;
                effects.trail(spell.pos, &SPELL_PALETTE);
                return true;
            }

            let pos = spell.target_pos;
            sfx.push(Sfx::Explode);
            effects.explode(pos, ENEMY_RADIUS * 0.5, &SPELL_PALETTE);
            match (&spell.target, boss) {
                (SpellTarget::Doomed(enemy), _) => {
                    effects.explode(pos, enemy.radius, &KILL_PALETTE);
                    if enemy.is_boss() {
                        // A boss goes out with a bigger bang.
                        effects.explode_around(pos, 30.0, 3, enemy.radius, &KILL_PALETTE);
                        boss_fell_at = Some(pos);
                    }
                }
                (SpellTarget::Boss, Some(boss)) => {
                    if let Some(lives) = &mut boss.boss {
                        lives.hit_flash = BOSS_HIT_FLASH_SECONDS;
                    }
                    boss.pos += (boss.pos - player).normalize_or_zero() * BOSS_HIT_KNOCKBACK;
                    boss.keep_on_screen();
                }
                // The boss fell before this spell arrived.
                (SpellTarget::Boss, None) => {}
            }
            false
        });

        if let Some(pos) = boss_fell_at
            && !self.is_over()
        {
            self.complete_level(pos);
        }
    }

    pub(super) fn move_enemies(&mut self, dt: f32, memory: &mut Memory) {
        let player = self.player;
        for enemy in &mut self.enemies {
            let speed = enemy.speed();
            let toward = (player - enemy.pos).normalize_or_zero();
            // Enemies with an even phase go left around obstacles, the rest
            // right, so they don't all bunch up on one side.
            let prefer_left = (enemy.phase as u32).is_multiple_of(2);
            let dir = steer(
                enemy.pos,
                enemy.radius,
                toward,
                &self.obstacles,
                prefer_left,
            );
            enemy.pos += dir * speed * dt;
            enemy.age += dt;
            enemy.shown_for += dt;
        }
        self.separate_enemies();
        for enemy in &mut self.enemies {
            if enemy.is_boss() {
                enemy.keep_on_screen();
            }
            enemy.pos = push_out(enemy.pos, enemy.radius, &self.obstacles);
        }

        let touches = |e: &Enemy| e.pos.distance(player) < PLAYER_RADIUS + e.radius;
        let mut hurt = false;

        // The boss bounces off her and starts slow again.
        for boss in self.enemies.iter_mut() {
            let touching = touches(boss);
            let Some(lives) = &mut boss.boss else {
                continue;
            };
            lives.harmless_for = (lives.harmless_for - dt).max(0.0);
            if !touching || lives.harmless_for > 0.0 {
                continue;
            }
            lives.harmless_for = BOSS_HARMLESS_SECONDS;
            boss.record_miss(memory, self.now);
            hurt = true;
            self.energy -= COLLISION_PENALTY;
            self.out.sfx.push(Sfx::Hurt);
            self.display.effects.explode(
                (boss.pos + player) / 2.0,
                ENEMY_RADIUS,
                &COLLISION_PALETTE,
            );
            boss.pos += (boss.pos - player).normalize_or_zero() * BOSS_COLLISION_KNOCKBACK;
            boss.keep_on_screen();
            boss.age = 0.0;
        }

        let (collided, remaining): (Vec<Enemy>, Vec<Enemy>) = self
            .enemies
            .drain(..)
            .partition(|e| !e.is_boss() && touches(e));
        self.enemies = remaining;
        for enemy in &collided {
            enemy.record_miss(memory, self.now);
            hurt = true;
            self.energy -= COLLISION_PENALTY;
            self.out.sfx.extend([Sfx::Explode, Sfx::Hurt]);
            self.display
                .effects
                .explode(enemy.pos, enemy.radius, &COLLISION_PALETTE);
            // Show the pair so a collision still teaches something.
            self.show_question(&enemy.question, YELLOW);
        }
        if hurt {
            self.mistake();
            self.clear_typed();
        }
    }

    /// Pushes overlapping enemies apart so their labels stay readable.
    fn separate_enemies(&mut self) {
        for _ in 0..4 {
            for i in 0..self.enemies.len() {
                for j in i + 1..self.enemies.len() {
                    let (a, b) = (&self.enemies[i], &self.enemies[j]);
                    let offset = b.reach_center() - a.reach_center();
                    let min_distance = a.reach() + b.reach();
                    let distance = offset.length();
                    if distance >= min_distance {
                        continue;
                    }
                    let dir = if distance > 0.001 {
                        offset / distance
                    } else {
                        Vec2::from_angle(self.rng.range(0.0, std::f32::consts::TAU))
                    };
                    let push = dir * (min_distance - distance) / 2.0;
                    self.enemies[i].pos -= push;
                    self.enemies[j].pos += push;
                }
            }
        }
    }
}
