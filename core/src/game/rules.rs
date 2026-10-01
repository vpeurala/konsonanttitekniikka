//! The game's numbers and the pure rules built on them: how much energy
//! things cost, how many points a level needs, how fast monsters move.

use crate::color::{
    Color, GOLD, MAGENTA, MAROON, MOULD_GREEN, ORANGE, PINK, RED, SKYBLUE, SLIME, VIOLET, WHITE,
    YELLOW,
};

pub const PLAYER_SPEED: f32 = 260.0;
pub const PLAYER_RADIUS: f32 = 16.0;
pub const ENEMY_RADIUS: f32 = 20.0;
pub const MAX_ENEMIES: usize = 6;

/// Enemies never appear closer than this to the player.
pub const MIN_SPAWN_DISTANCE: f32 = 300.0;
pub const SPAWN_ATTEMPTS: usize = 40;

/// A pair she meets for the first time shows its hint after this long.
pub const FIRST_HINT_SECONDS: f32 = 3.0;
/// Every enemy shows its answer once it is this fast, as a fraction of its
/// top speed, however well she knows it.
pub const HINT_SPEED_FRACTION: f32 = 0.75;
/// The difficulty (see `memory`) from which a pair counts as new to her and
/// gets the quickest hint; the better she knows it, the longer the hint
/// waits.
const NEW_TO_HER: f32 = 0.5;

/// How long a pair of this `difficulty` waits before it shows its hint:
/// `FIRST_HINT_SECONDS` while it is new to her, then longer and longer as
/// she learns it, until it is as long as it takes an enemy to reach
/// `HINT_SPEED_FRACTION` of its top speed, when every hint shows anyway.
pub fn hint_delay(difficulty: f32) -> f32 {
    let learned = ((NEW_TO_HER - difficulty) / NEW_TO_HER).clamp(0.0, 1.0);
    let slowest = (HINT_SPEED_FRACTION * PLAYER_SPEED - START_SPEED) / SPEED_GROWTH;
    FIRST_HINT_SECONDS + (slowest - FIRST_HINT_SECONDS) * learned
}

/// Whether an enemy shows its answer. Every enemy starts without a hint.
/// It comes after `hint_delay` of the pair's `difficulty`, counted from
/// when the pair was shown (`shown_for` seconds ago), or once the enemy has
/// reached `HINT_SPEED_FRACTION` of its top speed (`speed_fraction`).
pub fn shows_hint(difficulty: f32, shown_for: f32, speed_fraction: f32) -> bool {
    shown_for >= hint_delay(difficulty) || speed_fraction >= HINT_SPEED_FRACTION
}

/// 100%: the energy she starts with, and what the energy is measured
/// against (the percentage shown, and the stars).
pub const FULL_ENERGY: f32 = 100.0;
/// The most energy she can build up on the first level, by defeating
/// monsters...
const FIRST_LEVEL_ENERGY_CAP: f32 = 150.0;
/// ...which grows by this much with every level, to make up for the harder
/// levels...
const ENERGY_CAP_PER_LEVEL: f32 = 2.0;
/// ...but never goes past this.
const HARD_ENERGY_CAP: f32 = 200.0;

/// The most energy she can have on `level`.
pub fn energy_cap(level: u32) -> f32 {
    (FIRST_LEVEL_ENERGY_CAP + ENERGY_CAP_PER_LEVEL * level.saturating_sub(1) as f32)
        .min(HARD_ENERGY_CAP)
}
/// What answering a one-eyed monster (the one showing a word) is worth:
/// the easiest kind.
pub const CYCLOPS_POINTS: u32 = 1;
pub const CYCLOPS_ENERGY: f32 = 10.0;
/// What answering a star-like monster (the one showing a number) is worth.
pub const STAR_POINTS: u32 = 2;
pub const STAR_ENERGY: f32 = 20.0;
/// Each hit on a boss is worth this, so a boss with `n` numbers gives `n`
/// points and `n` times the energy in all.
pub const BOSS_POINTS_PER_HIT: u32 = 1;
pub const BOSS_ENERGY_PER_HIT: f32 = 10.0;
/// What answering a bird is worth: the hardest kind.
pub const BIRD_POINTS: u32 = 3;
pub const BIRD_ENERGY: f32 = 30.0;
/// Each answered number of the mould is worth this, like a boss's hit.
pub const MOULD_POINTS_PER_HIT: u32 = 1;
pub const MOULD_ENERGY_PER_HIT: f32 = 10.0;
/// What answering a golem is worth: it takes a whole long number.
pub const GOLEM_POINTS: u32 = 5;
pub const GOLEM_ENERGY: f32 = 50.0;
pub const WRONG_PENALTY: f32 = 10.0;
/// Wrong keys never take energy below this, so only collisions can end
/// the game.
pub const LOW_ENERGY: f32 = 30.0;
pub const COLLISION_PENALTY: f32 = 20.0;

pub const FEEDBACK_SECONDS: f32 = 2.5;

const FIRST_LEVEL_POINTS: u32 = 10;
/// How many more points each level needs than the one before...
const LEVEL_POINTS_INCREASE: u32 = 5;
/// ...up to this many, so no level drags on. Later levels get harder
/// through faster spawns, tougher bosses and longer numbers instead.
pub const MAX_LEVEL_POINTS: u32 = 40;
pub const BANNER_SECONDS: f32 = 2.5;
/// No new enemies appear for this long after a level starts.
pub const LEVEL_BREAK_SECONDS: f32 = 2.0;

/// A new enemy's speed, except a one-eyed monster's.
pub const START_SPEED: f32 = 12.0;
/// A one-eyed monster is a little faster than the others to begin with...
pub const CYCLOPS_START_SPEED_FACTOR: f32 = 1.3;
/// ...and speeds up a little faster, to make up for being the easiest.
pub const CYCLOPS_GROWTH_FACTOR: f32 = 1.25;
/// Speed an enemy gains per second on screen, up to the player's speed, on
/// the level where the ramp ends (`SPEED_GROWTH_LEVEL`) and after it.
pub const SPEED_GROWTH: f32 = 3.0;
/// ...and on the first level, where monsters speed up very slowly.
const FIRST_LEVEL_SPEED_GROWTH: f32 = 0.5;
/// The level from which monsters gain speed at the full rate; the growth
/// rises evenly until then (level 21 is when all pairs have been met).
const SPEED_GROWTH_LEVEL: u32 = 21;
/// The spawn interval at the start of a level, once the easy levels are over.
pub const START_SPAWN_INTERVAL: f32 = 4.0;
/// ...and on the first level, where newcomers arrive less often.
const FIRST_LEVEL_SPAWN_INTERVAL: f32 = 5.0;
pub const MIN_SPAWN_INTERVAL: f32 = 1.2;
/// Spawn interval lost per second spent on a level, on the later levels...
const SPAWN_INTERVAL_SHRINK: f32 = 0.05;
/// ...and on the first level.
const FIRST_LEVEL_SPAWN_INTERVAL_SHRINK: f32 = 0.03;
/// The level from which spawns follow the later levels' numbers.
const SPAWN_EASE_LEVEL: u32 = 21;

pub const BOSS_RADIUS: f32 = 36.0;
/// The first level's boss takes this many hits.
const BOSS_FIRST_HITS: usize = 3;
/// Bosses take one more hit every this many levels.
const BOSS_LEVELS_PER_EXTRA_HIT: u32 = 2;
pub const BOSS_MAX_HITS: usize = 10;
/// The boss moves at this fraction of a normal enemy's speed.
pub const BOSS_SPEED_FACTOR: f32 = 0.6;
/// How far a spell pushes the boss back.
pub const BOSS_HIT_KNOCKBACK: f32 = 50.0;
/// How far the boss bounces back after running into the player.
pub const BOSS_COLLISION_KNOCKBACK: f32 = 220.0;
pub const BOSS_HIT_FLASH_SECONDS: f32 = 0.25;
/// After running into the player, the boss can't hurt her again for this
/// long, even if it is stuck next to her against an edge.
pub const BOSS_HARMLESS_SECONDS: f32 = 1.5;

pub const SPELL_SPEED: f32 = 700.0;
/// How long she keeps her hand raised after casting.
pub const CAST_SECONDS: f32 = 0.35;

pub const KILL_PALETTE: [Color; 4] = [ORANGE, YELLOW, GOLD, WHITE];
pub const SPELL_PALETTE: [Color; 4] = [PINK, MAGENTA, VIOLET, WHITE];
/// The splashes of the mould.
pub const MOULD_PALETTE: [Color; 3] = [SLIME, MOULD_GREEN, YELLOW];
pub const COLLISION_PALETTE: [Color; 3] = [RED, MAROON, ORANGE];

/// How often a new monster uses one of the level's new pairs, when one is
/// free, so new pairs get extra practice.
/// Birds appear from this level on...
pub const BIRD_FIRST_LEVEL: u32 = 25;
/// ...as this share of the newcomers, and never more than `MAX_BIRDS` at
/// once.
pub const BIRD_SHARE: f32 = 0.15;
pub const MAX_BIRDS: usize = 2;
/// A bird takes aim for this long before each attack, drifting slowly
/// toward her so she has time to see it.
pub const BIRD_AIM_SECONDS: f32 = 3.0;
pub const BIRD_AIM_SPEED: f32 = 40.0;
/// A bird in its attack flies this fast, as a fraction of her speed.
pub const BIRD_ATTACK_SPEED_FACTOR: f32 = 1.2;
/// The mould appears from this level on, as this share of the newcomers,
/// and never more than one at a time.
pub const MOULD_FIRST_LEVEL: u32 = 35;
pub const MOULD_SHARE: f32 = 0.08;
/// It creeps this fast, very slowly...
pub const MOULD_SPEED: f32 = 8.0;
/// ...and every time it has crept this far it gains a number, and the
/// body it drags behind it grows a segment longer.
pub const MOULD_GROWTH_DISTANCE: f32 = 40.0;
/// It has at most this many numbers, which is also how many it brings in
/// all: answering them makes room for no new ones beyond these.
pub const MOULD_MAX_NUMBERS: usize = 8;
/// The size of its head with one number, and how much each more adds.
pub const MOULD_BASE_RADIUS: f32 = 22.0;
pub const MOULD_RADIUS_PER_NUMBER: f32 = 1.5;
/// The slimy body the head drags behind it is this thick (its radius)...
pub const MOULD_TUBE_RADIUS: f32 = 16.0;
/// ...and its tail end draws in this fast when it shrinks.
pub const MOULD_TAIL_SPEED: f32 = 90.0;
/// Golems appear from this level on, as this share of the newcomers, and
/// never more than `MAX_GOLEMS` at once.
pub const GOLEM_FIRST_LEVEL: u32 = 45;
pub const GOLEM_SHARE: f32 = 0.1;
pub const MAX_GOLEMS: usize = 2;
/// A golem is bigger than the others, but smaller than a boss.
pub const GOLEM_RADIUS: f32 = 28.0;
/// It starts a little slower than a star monster...
pub const GOLEM_START_SPEED_FACTOR: f32 = 0.85;
/// ...speeds up a little slower...
pub const GOLEM_GROWTH_FACTOR: f32 = 0.8;
/// ...and is never faster than this, as a fraction of her speed.
pub const GOLEM_TOP_SPEED_FACTOR: f32 = 0.6;
pub const NEW_PAIR_SHARE: f32 = 0.5;

/// How often a new monster comes out of a portal rather than a screen
/// edge.
pub const PORTAL_SPAWN_SHARE: f32 = 0.75;
/// Monsters don't come out of a portal closer than this to the player.
pub const MIN_PORTAL_SPAWN_DISTANCE: f32 = 200.0;
pub const PORTAL_PALETTE: [Color; 3] = [VIOLET, SKYBLUE, WHITE];

/// A frame longer than this counts as this long, so a stall doesn't make
/// the game jump ahead.
pub const MAX_FRAME_SECONDS: f32 = 0.25;
/// The game always advances in steps of exactly this long, however long
/// the frames are, so a game plays out the same on a slow phone and a fast
/// monitor, and a fast spell or monster can't skip past what it should
/// hit. It is short enough that a screen showing fewer or more frames
/// than steps per second looks smooth without blending between steps.
pub const STEP_SECONDS: f32 = 1.0 / 120.0;
/// How far short of a step still counts as one, to absorb rounding.
const STEP_TOLERANCE: f32 = 1e-6;

/// The energy left after a wrong key: the penalty, but never below
/// `LOW_ENERGY`.
pub fn after_wrong_key(energy: f32) -> f32 {
    if energy <= LOW_ENERGY {
        energy
    } else {
        (energy - WRONG_PENALTY).max(LOW_ENERGY)
    }
}

/// The points needed to summon the boss of `level` (counting from 1).
pub fn points_to_clear(level: u32) -> u32 {
    (FIRST_LEVEL_POINTS + LEVEL_POINTS_INCREASE * (level - 1)).min(MAX_LEVEL_POINTS)
}

/// How many numbers (and so hits) the boss of `level` has.
pub fn boss_hits(level: u32) -> usize {
    let extra = ((level - 1) / BOSS_LEVELS_PER_EXTRA_HIT) as usize;
    (BOSS_FIRST_HITS + extra).min(BOSS_MAX_HITS)
}

/// Blends from `first` on level 1 to `later` on `full_level` and after.
fn ramp(first: f32, later: f32, level: u32, full_level: u32) -> f32 {
    let t = (level.saturating_sub(1) as f32 / (full_level - 1) as f32).min(1.0);
    first + (later - first) * t
}

/// How fast enemies of `level` speed up, per second on screen: very slowly
/// at first, then a little more every level.
pub fn speed_growth(level: u32) -> f32 {
    ramp(
        FIRST_LEVEL_SPEED_GROWTH,
        SPEED_GROWTH,
        level,
        SPEED_GROWTH_LEVEL,
    )
}

/// Every enemy starts slow (at `start`) and speeds up as it ages (`growth`
/// per second, see `speed_growth`), until it is as fast as the player.
pub fn enemy_speed(age: f32, start: f32, growth: f32) -> f32 {
    (start + growth * age).min(PLAYER_SPEED)
}

/// Difficulty comes from the spawn rate: enemies appear more often the
/// longer a level lasts, and a new level starts calm again. The early
/// levels start calmer and tighten more slowly.
pub fn spawn_interval(level: u32, level_time: f32) -> f32 {
    let start = ramp(
        FIRST_LEVEL_SPAWN_INTERVAL,
        START_SPAWN_INTERVAL,
        level,
        SPAWN_EASE_LEVEL,
    );
    let shrink = ramp(
        FIRST_LEVEL_SPAWN_INTERVAL_SHRINK,
        SPAWN_INTERVAL_SHRINK,
        level,
        SPAWN_EASE_LEVEL,
    );
    (start - shrink * level_time).max(MIN_SPAWN_INTERVAL)
}

/// Stars for finishing a level with this fraction of energy left: three
/// for most of it, two for some, one for scraping through.
pub fn stars_for(energy_fraction: f32) -> u8 {
    if energy_fraction >= 0.7 {
        3
    } else if energy_fraction >= 0.35 {
        2
    } else {
        1
    }
}

/// Turns frames of varying length into fixed steps: time that doesn't add
/// up to a whole step waits for the next frame.
#[derive(Debug, Default)]
pub(super) struct Timestep {
    leftover: f32,
}

impl Timestep {
    /// How many steps of `STEP_SECONDS` a frame of `dt` seconds makes
    /// due. A frame is never counted longer than `MAX_FRAME_SECONDS`.
    pub(super) fn steps_due(&mut self, dt: f32) -> u32 {
        self.leftover += dt.clamp(0.0, MAX_FRAME_SECONDS);
        let mut steps = 0;
        // Rounding in `f32` mustn't make a frame that is exactly a whole
        // number of steps lose one.
        while self.leftover >= STEP_SECONDS - STEP_TOLERANCE {
            self.leftover -= STEP_SECONDS;
            steps += 1;
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_enemy_appears_without_a_hint() {
        for difficulty in [0.0, 0.25, 0.5, 1.0] {
            assert!(!shows_hint(
                difficulty,
                0.0,
                enemy_speed(0.0, START_SPEED, SPEED_GROWTH) / PLAYER_SPEED
            ));
        }
    }

    #[test]
    fn a_new_pair_gets_its_hint_after_three_seconds() {
        for difficulty in [0.5, 0.75, 1.0] {
            assert_eq!(hint_delay(difficulty), FIRST_HINT_SECONDS);
            assert!(!shows_hint(difficulty, FIRST_HINT_SECONDS - 0.1, 0.0));
            assert!(shows_hint(difficulty, FIRST_HINT_SECONDS, 0.0));
        }
    }

    #[test]
    fn the_better_a_pair_is_known_the_longer_its_hint_waits() {
        let mut difficulty = NEW_TO_HER;
        while difficulty > 0.0 {
            let better = difficulty - 0.05;
            assert!(hint_delay(better.max(0.0)) > hint_delay(difficulty));
            difficulty = better;
        }
    }

    #[test]
    fn a_learned_pairs_hint_comes_when_the_enemy_reaches_the_hint_speed() {
        let delay = hint_delay(0.0);
        let speed = |age| enemy_speed(age, START_SPEED, SPEED_GROWTH) / PLAYER_SPEED;
        assert!(speed(delay - 0.1) < HINT_SPEED_FRACTION);
        assert!(speed(delay + 0.1) >= HINT_SPEED_FRACTION);
    }

    #[test]
    fn every_hint_shows_once_the_enemy_is_fast() {
        for difficulty in [0.0, 0.3, 1.0] {
            assert!(!shows_hint(difficulty, 0.0, HINT_SPEED_FRACTION - 0.01));
            assert!(shows_hint(difficulty, 0.0, HINT_SPEED_FRACTION));
        }
    }

    #[test]
    fn the_energy_cap_starts_at_150_and_grows_slowly_to_200() {
        assert_eq!(energy_cap(1), 150.0);
        assert_eq!(energy_cap(2), 152.0);
        assert_eq!(energy_cap(10), 168.0);
        for level in 1..100 {
            assert!(energy_cap(level + 1) >= energy_cap(level));
            assert!(energy_cap(level) <= 200.0);
        }
        assert_eq!(energy_cap(26), 200.0);
        assert_eq!(energy_cap(1000), 200.0);
    }

    #[test]
    fn wrong_keys_cost_energy_above_the_low_level() {
        assert_eq!(after_wrong_key(100.0), 100.0 - WRONG_PENALTY);
    }

    #[test]
    fn wrong_keys_never_go_below_the_low_level() {
        assert_eq!(after_wrong_key(LOW_ENERGY + 1.0), LOW_ENERGY);
        assert_eq!(after_wrong_key(LOW_ENERGY), LOW_ENERGY);
        assert_eq!(after_wrong_key(5.0), 5.0);
    }

    #[test]
    fn more_energy_left_earns_more_stars() {
        assert_eq!(stars_for(1.0), 3);
        assert_eq!(stars_for(0.5), 2);
        assert_eq!(stars_for(0.1), 1);
        for i in 0..100 {
            let e = i as f32 / 100.0;
            assert!(stars_for(e + 0.01) >= stars_for(e));
        }
    }

    #[test]
    fn first_level_needs_ten_points() {
        assert_eq!(points_to_clear(1), 10);
    }

    #[test]
    fn each_level_needs_more_points_than_the_last_up_to_a_limit() {
        for level in 1..50 {
            let (this, next) = (points_to_clear(level), points_to_clear(level + 1));
            assert!(next > this || next == MAX_LEVEL_POINTS);
        }
        assert_eq!(points_to_clear(7), MAX_LEVEL_POINTS);
        assert_eq!(points_to_clear(100), MAX_LEVEL_POINTS);
    }

    #[test]
    fn first_boss_takes_three_hits() {
        assert_eq!(boss_hits(1), 3);
    }

    #[test]
    fn bosses_get_tougher_up_to_a_limit() {
        for level in 1..100 {
            assert!(boss_hits(level + 1) >= boss_hits(level));
        }
        assert!(boss_hits(10) > boss_hits(1));
        assert_eq!(boss_hits(1000), BOSS_MAX_HITS);
    }

    #[test]
    fn enemies_speed_up_with_age_to_the_players_speed() {
        assert_eq!(enemy_speed(0.0, START_SPEED, SPEED_GROWTH), START_SPEED);
        assert!(
            enemy_speed(10.0, START_SPEED, SPEED_GROWTH)
                > enemy_speed(0.0, START_SPEED, SPEED_GROWTH)
        );
        assert_eq!(enemy_speed(1000.0, START_SPEED, SPEED_GROWTH), PLAYER_SPEED);
    }

    #[test]
    fn speed_growth_starts_slow_and_rises_gradually_to_the_full_rate() {
        assert_eq!(speed_growth(1), FIRST_LEVEL_SPEED_GROWTH);
        for level in 1..SPEED_GROWTH_LEVEL {
            assert!(speed_growth(level + 1) > speed_growth(level));
        }
        assert_eq!(speed_growth(SPEED_GROWTH_LEVEL), SPEED_GROWTH);
        assert_eq!(speed_growth(1000), SPEED_GROWTH);
    }

    #[test]
    fn spawns_get_more_frequent_within_a_level_down_to_a_floor() {
        for level in [1, 5, SPAWN_EASE_LEVEL, 30] {
            assert!(spawn_interval(level, 10.0) < spawn_interval(level, 0.0));
            assert_eq!(spawn_interval(level, 1000.0), MIN_SPAWN_INTERVAL);
        }
        assert_eq!(spawn_interval(SPAWN_EASE_LEVEL, 0.0), START_SPAWN_INTERVAL);
    }

    #[test]
    fn early_levels_spawn_less_often_than_later_ones() {
        for time in [0.0, 20.0, 40.0] {
            assert!(spawn_interval(1, time) > spawn_interval(SPAWN_EASE_LEVEL, time));
            assert!(spawn_interval(5, time) >= spawn_interval(6, time));
        }
    }

    #[test]
    fn a_60_hz_frame_is_two_steps() {
        assert_eq!(Timestep::default().steps_due(1.0 / 60.0), 2);
    }

    #[test]
    fn a_long_frame_is_many_steps() {
        let steps = Timestep::default().steps_due(0.2);
        assert_eq!(steps, (0.2 / STEP_SECONDS) as u32);
    }

    #[test]
    fn a_stall_counts_as_a_bounded_frame() {
        let steps = Timestep::default().steps_due(30.0);
        assert!((steps as f32 * STEP_SECONDS - MAX_FRAME_SECONDS).abs() <= STEP_SECONDS);
    }

    #[test]
    fn a_frame_shorter_than_a_step_waits_for_the_next() {
        let mut timestep = Timestep::default();
        assert_eq!(timestep.steps_due(STEP_SECONDS * 0.6), 0);
        assert_eq!(timestep.steps_due(STEP_SECONDS * 0.6), 1);
    }

    #[test]
    fn uneven_frames_add_up_to_the_same_number_of_steps() {
        let mut timestep = Timestep::default();
        let total: u32 = (0..1000)
            .map(|i| timestep.steps_due(if i % 2 == 0 { 0.010 } else { 0.023 }))
            .sum();
        // 16.5 seconds at 120 steps per second, less what still waits.
        assert!((1978..=1980).contains(&total), "{total}");
    }

    #[test]
    fn no_time_is_no_steps() {
        assert_eq!(Timestep::default().steps_due(0.0), 0);
    }
}
