//! What is in the arena: monsters, spells in flight, portals and obstacles.
//!
//! A `World` moves things and keeps its own books, but knows nothing of
//! energy, sounds, sparks or what she has learned. When something happens
//! it says so in a report (`Hit`, `Contact`, `SpellEvent`), and the game
//! decides what that means for her.

use glam::Vec2;

use super::enemy::{Enemy, EnemyId, Kind};
use super::rules::{
    BOSS_COLLISION_KNOCKBACK, BOSS_HARMLESS_SECONDS, BOSS_HIT_FLASH_SECONDS, BOSS_HIT_KNOCKBACK,
    PLAYER_RADIUS, SPELL_SPEED,
};
use crate::arena::{ARENA_H, ARENA_W, girl_hand};
use crate::memory::Memory;
use crate::obstacles::{Obstacle, obstacles_for_level, push_out, steer_around};
use crate::portals::portal_positions;
use crate::rng::Rng;

/// What a spell is flying toward.
pub enum SpellTarget {
    /// An enemy that is already out of play and explodes when hit.
    Doomed(Box<Enemy>),
    /// The boss, which is still in play and only loses a life.
    Boss,
    /// The mould called this, which is still in play and shrinks.
    Mould(EnemyId),
}

/// A magic bolt flying toward an answered enemy.
pub struct Spell {
    pub pos: Vec2,
    pub target: SpellTarget,
    /// Where the target was last seen, in case the boss dies first.
    pub target_pos: Vec2,
}

impl Spell {
    /// The enemy that is out of play and waiting for this spell.
    pub fn doomed(&self) -> Option<&Enemy> {
        match &self.target {
            SpellTarget::Doomed(enemy) => Some(enemy.as_ref()),
            SpellTarget::Boss | SpellTarget::Mould(_) => None,
        }
    }
}

/// An enemy was answered.
pub struct Hit {
    /// The enemy as it was when answered, for what she learns from it.
    pub answered: Enemy,
    pub outcome: HitOutcome,
}

pub enum HitOutcome {
    /// A boss with lives left moved on to its next number.
    Wounded,
    /// The enemy left play; a spell is on its way to finish it.
    Defeated { boss: bool },
}

/// A monster or boss reached her.
pub struct Contact {
    /// The enemy as it was when it touched her.
    pub enemy: Enemy,
    pub kind: ContactKind,
}

pub enum ContactKind {
    /// A boss touched her at this point, and bounced away. It stays.
    Boss { at: Vec2 },
    /// The mould touched her at this point. It stays and creeps on.
    Mould { at: Vec2 },
    /// An ordinary monster touched her and left play.
    Ordinary,
}

/// One thing a spell did while flying.
pub enum SpellEvent {
    /// It moved on, to this point.
    Trail(Vec2),
    Impact(Impact),
}

/// A spell reached its target.
pub struct Impact {
    pub pos: Vec2,
    pub target: ImpactTarget,
}

pub enum ImpactTarget {
    /// An enemy already out of play exploded.
    Doomed { radius: f32, boss: bool },
    /// The boss lost a life and was knocked back.
    Boss,
    /// The mould shrank.
    Mould { radius: f32 },
    /// The boss it followed was gone.
    Nothing,
}

#[cfg(test)]
mod test_support;

pub struct World {
    enemies: Vec<Enemy>,
    spells: Vec<Spell>,
    /// The id the next enemy to join gets.
    next_enemy_id: u32,
    /// Whether the monsters that join may show hints.
    hints: bool,
    /// The current level's portals.
    portals: Vec<Vec2>,
    /// The current level's stones, trees and lakes.
    obstacles: Vec<Obstacle>,
}

impl World {
    /// The arena of `level`, with nobody in it.
    pub fn for_level(level: u32) -> World {
        let (portals, obstacles) = layout(level);
        World {
            enemies: Vec::new(),
            spells: Vec::new(),
            next_enemy_id: 0,
            hints: true,
            portals,
            obstacles,
        }
    }

    /// Lets the monsters that join afterwards show hints, or not.
    pub fn set_hints(&mut self, hints: bool) {
        self.hints = hints;
    }

    pub fn enemies(&self) -> &[Enemy] {
        &self.enemies
    }

    pub fn spells(&self) -> &[Spell] {
        &self.spells
    }

    pub fn portals(&self) -> &[Vec2] {
        &self.portals
    }

    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }

    /// Adds `enemy` to the world, giving it an id of its own.
    pub fn admit(&mut self, mut enemy: Enemy) {
        enemy.hints_enabled = self.hints;
        enemy.id = EnemyId(self.next_enemy_id);
        self.next_enemy_id += 1;
        self.enemies.push(enemy);
    }

    /// Moves on to the arena of `level`. Every monster in play leaves, and
    /// is handed back; spells already flying carry on.
    pub fn next_level(&mut self, level: u32) -> Vec<Enemy> {
        let (portals, obstacles) = layout(level);
        self.portals = portals;
        self.obstacles = obstacles;
        std::mem::take(&mut self.enemies)
    }

    /// Answers the enemy called `id`, if it is still in play: a boss with
    /// lives left moves on to its next number, anything else leaves play.
    /// Either way a spell flies from `player`'s hand toward it.
    pub fn hit(&mut self, id: EnemyId, player: Vec2, memory: &Memory) -> Option<Hit> {
        let index = self.enemies.iter().position(|e| e.id == id)?;
        let answered = self.enemies[index].clone();
        let target_pos = answered.pos;
        let is_mould = answered.mould.is_some();
        let next = if is_mould {
            self.enemies[index].shrink()
        } else {
            self.enemies[index]
                .boss
                .as_mut()
                .and_then(|lives| lives.queue.pop_front())
        };
        let (target, outcome) = match next {
            Some(question) => {
                let difficulty = Enemy::difficulty_of(&question, memory);
                self.enemies[index].show(question, difficulty);
                let target = if is_mould {
                    SpellTarget::Mould(id)
                } else {
                    SpellTarget::Boss
                };
                (target, HitOutcome::Wounded)
            }
            None => {
                let enemy = self.enemies.remove(index);
                let boss = enemy.is_boss();
                (
                    SpellTarget::Doomed(Box::new(enemy)),
                    HitOutcome::Defeated { boss },
                )
            }
        };
        self.spells.push(Spell {
            pos: girl_hand(player, target_pos),
            target,
            target_pos,
        });
        Some(Hit { answered, outcome })
    }

    /// Flies every spell on by `dt` seconds, and says what they did.
    pub fn advance_spells(&mut self, dt: f32, player: Vec2) -> Vec<SpellEvent> {
        for enemy in &mut self.enemies {
            if let Some(lives) = &mut enemy.boss {
                lives.hit_flash = (lives.hit_flash - dt).max(0.0);
            }
        }

        let step = SPELL_SPEED * dt;
        let mut events = Vec::new();
        let enemies = &mut self.enemies;
        self.spells.retain_mut(|spell| {
            // A spell at the boss, or the mould, follows it around.
            let boss = match &spell.target {
                SpellTarget::Mould(id) => enemies.iter_mut().find(|e| e.id == *id),
                _ => enemies.iter_mut().find(|e| e.is_boss()),
            };
            if let (SpellTarget::Boss | SpellTarget::Mould(_), Some(boss)) = (&spell.target, &boss)
            {
                spell.target_pos = boss.pos;
            }

            let to_target = spell.target_pos - spell.pos;
            if to_target.length() > step {
                spell.pos += to_target.normalize() * step;
                events.push(SpellEvent::Trail(spell.pos));
                return true;
            }

            let target = match (&spell.target, boss) {
                (SpellTarget::Doomed(enemy), _) => ImpactTarget::Doomed {
                    radius: enemy.radius,
                    boss: enemy.is_boss(),
                },
                (SpellTarget::Boss, Some(boss)) => {
                    if let Some(lives) = &mut boss.boss {
                        lives.hit_flash = BOSS_HIT_FLASH_SECONDS;
                    }
                    boss.pos += (boss.pos - player).normalize_or_zero() * BOSS_HIT_KNOCKBACK;
                    boss.keep_on_screen();
                    ImpactTarget::Boss
                }
                (SpellTarget::Mould(_), Some(mould)) => ImpactTarget::Mould {
                    radius: mould.radius,
                },
                (SpellTarget::Boss | SpellTarget::Mould(_), None) => ImpactTarget::Nothing,
            };
            events.push(SpellEvent::Impact(Impact {
                pos: spell.target_pos,
                target,
            }));
            false
        });
        events
    }

    /// Moves every monster on by `dt` seconds toward `player`, and says
    /// which reached her. Monsters that touch her leave play, apart from a
    /// boss, which bounces away and is harmless for a while.
    pub fn advance_enemies(&mut self, dt: f32, player: Vec2, rng: &mut Rng) -> Vec<Contact> {
        self.walk_toward(dt, player, rng);
        let mut contacts = self.bounce_bosses(dt, player);
        contacts.extend(self.touch_moulds(dt, player));
        contacts.extend(self.take_ordinary_contacts(player));
        contacts
    }

    /// Walks every monster toward `player`, around the obstacles, and keeps
    /// them apart, on the screen (the bosses) and out of the obstacles.
    fn walk_toward(&mut self, dt: f32, player: Vec2, rng: &mut Rng) {
        // Where everyone stood at the start of the step, for those who steer
        // around the others.
        // Birds fly over everything, so nobody steers around them.
        let crowd: Vec<(Vec2, f32)> = self
            .enemies
            .iter()
            .map(|e| {
                (
                    e.pos,
                    if matches!(e.kind(), Kind::Bird | Kind::Mould) {
                        0.0
                    } else {
                        e.radius
                    },
                )
            })
            .collect();
        // The moulds' heads and blobs, which everyone but the birds and the
        // moulds themselves goes around.
        let mould_circles: Vec<(Vec2, f32)> = self
            .enemies
            .iter()
            .filter(|e| e.mould.is_some())
            .flat_map(|e| std::iter::once((e.pos, e.radius)).chain(e.mould_body_circles()))
            .collect();
        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            if enemy.kind() == Kind::Bird {
                enemy.fly(dt, player);
                enemy.age += dt;
                enemy.shown_for += dt;
                continue;
            }
            let speed = enemy.speed();
            let toward = (player - enemy.pos).normalize_or_zero();
            // Enemies with an even phase go left around obstacles, the rest
            // right, so they don't all bunch up on one side.
            let prefer_left = (enemy.phase as u32).is_multiple_of(2);
            let obstacles = self.obstacles.iter().map(|o| (o.pos, o.radius));
            let others = crowd
                .iter()
                .enumerate()
                .filter(|(j, (_, radius))| enemy.avoids_monsters() && *j != i && *radius > 0.0)
                .map(|(_, &circle)| circle);
            let around_moulds = mould_circles
                .iter()
                .copied()
                .filter(|_| enemy.mould.is_none());
            let dir = steer_around(
                enemy.pos,
                enemy.radius,
                toward,
                obstacles.chain(others).chain(around_moulds),
                prefer_left,
            );
            enemy.creep_to(enemy.pos + dir * speed * dt, dt);
            enemy.age += dt;
            enemy.shown_for += dt;
        }
        self.separate_enemies(rng);
        for enemy in &mut self.enemies {
            if enemy.kind() == Kind::Bird {
                continue;
            }
            if enemy.is_boss() {
                enemy.keep_on_screen();
            }
            enemy.pos = push_out(enemy.pos, enemy.radius, &self.obstacles);
        }
    }

    /// A boss that touches her bounces off and starts slow again, unless it
    /// is still harmless from the last time.
    fn bounce_bosses(&mut self, dt: f32, player: Vec2) -> Vec<Contact> {
        let mut contacts = Vec::new();
        for boss in &mut self.enemies {
            let touching = touches(boss, player);
            let Some(lives) = &mut boss.boss else {
                continue;
            };
            lives.harmless_for = (lives.harmless_for - dt).max(0.0);
            if !touching || lives.harmless_for > 0.0 {
                continue;
            }
            lives.harmless_for = BOSS_HARMLESS_SECONDS;
            contacts.push(Contact {
                enemy: boss.clone(),
                kind: ContactKind::Boss {
                    at: (boss.pos + player) / 2.0,
                },
            });
            boss.pos += (boss.pos - player).normalize_or_zero() * BOSS_COLLISION_KNOCKBACK;
            boss.keep_on_screen();
            boss.age = 0.0;
        }
        contacts
    }

    /// A mould whose head or body touches her hurts her, once in a while. It stays, and
    /// creeps on.
    fn touch_moulds(&mut self, dt: f32, player: Vec2) -> Vec<Contact> {
        let mut contacts = Vec::new();
        for enemy in &mut self.enemies {
            if enemy.mould.is_none() {
                continue;
            }
            // Touching the head or any part of the body hurts.
            let at = if touches(enemy, player) {
                Some((enemy.pos + player) / 2.0)
            } else {
                enemy.mould_body_touch(player, PLAYER_RADIUS)
            };
            let Some(mould) = &mut enemy.mould else {
                continue;
            };
            mould.harmless_for = (mould.harmless_for - dt).max(0.0);
            let Some(at) = at else {
                continue;
            };
            if mould.harmless_for > 0.0 {
                continue;
            }
            mould.harmless_for = BOSS_HARMLESS_SECONDS;
            contacts.push(Contact {
                enemy: enemy.clone(),
                kind: ContactKind::Mould { at },
            });
        }
        contacts
    }

    /// The ordinary monsters that touch her leave play.
    fn take_ordinary_contacts(&mut self, player: Vec2) -> Vec<Contact> {
        let (collided, remaining): (Vec<Enemy>, Vec<Enemy>) = std::mem::take(&mut self.enemies)
            .into_iter()
            .partition(|e| !e.is_boss() && e.mould.is_none() && touches(e, player));
        self.enemies = remaining;
        collided
            .into_iter()
            .map(|enemy| Contact {
                enemy,
                kind: ContactKind::Ordinary,
            })
            .collect()
    }

    /// Pushes overlapping enemies apart, a few passes so a crowd settles.
    fn separate_enemies(&mut self, rng: &mut Rng) {
        for _ in 0..4 {
            for i in 0..self.enemies.len() {
                for j in i + 1..self.enemies.len() {
                    let (a, b) = (&self.enemies[i], &self.enemies[j]);
                    // Birds fly over the others.
                    if a.kind() == Kind::Bird || b.kind() == Kind::Bird {
                        continue;
                    }
                    let offset = b.reach_center() - a.reach_center();
                    let min_distance = a.reach() + b.reach();
                    let distance = offset.length();
                    if distance >= min_distance {
                        continue;
                    }
                    let dir = if distance > 0.001 {
                        offset / distance
                    } else {
                        Vec2::from_angle(rng.range(0.0, std::f32::consts::TAU))
                    };
                    let push = dir * (min_distance - distance) / 2.0;
                    self.enemies[i].pos -= push;
                    self.enemies[j].pos += push;
                }
            }
        }
    }
}

/// Whether the monster touches her.
fn touches(enemy: &Enemy, player: Vec2) -> bool {
    enemy.pos.distance(player) < PLAYER_RADIUS + enemy.radius
}

/// The portals and obstacles of a level. The first level has no obstacles.
fn layout(level: u32) -> (Vec<Vec2>, Vec<Obstacle>) {
    let portals = portal_positions(level, ARENA_W, ARENA_H);
    let obstacles = if level > 1 {
        obstacles_for_level(level, ARENA_W, ARENA_H, &portals)
    } else {
        Vec::new()
    };
    (portals, obstacles)
}
