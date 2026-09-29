//! The game's numbers and the pure rules built on them: how much energy
//! things cost, how many points a level needs, how fast monsters move.

use macroquad::prelude::{
    Color, GOLD, MAGENTA, MAROON, ORANGE, PINK, RED, SKYBLUE, VIOLET, WHITE, YELLOW,
};

pub(super) const PLAYER_SPEED: f32 = 260.0;
pub(super) const PLAYER_RADIUS: f32 = 16.0;
pub(super) const ENEMY_RADIUS: f32 = 20.0;
pub(super) const MAX_ENEMIES: usize = 6;

/// Enemies never appear closer than this to the player.
pub(super) const MIN_SPAWN_DISTANCE: f32 = 300.0;
pub(super) const SPAWN_ATTEMPTS: usize = 40;

/// A pair's first appearances in a game get an early hint.
pub(super) const HINTED_APPEARANCES: u32 = 3;
/// How long an early hint waits after the pair is shown.
pub(super) const EARLY_HINT_SECONDS: f32 = 2.0;
/// Any other enemy shows its answer once it is this fast, as a fraction of
/// its top speed. With the current speeds that takes about 22 seconds.
pub(super) const HINT_SPEED_FRACTION: f32 = 0.3;

/// Whether an enemy shows its answer. Every enemy starts without a hint.
/// One of the first appearances of its pair in this game
/// (`earlier_appearances`) gets it soon after the pair is shown
/// (`shown_for` seconds ago); any other only once it has reached
/// `HINT_SPEED_FRACTION` of its top speed (`speed_fraction`).
pub fn shows_hint(earlier_appearances: u32, shown_for: f32, speed_fraction: f32) -> bool {
    if earlier_appearances < HINTED_APPEARANCES {
        shown_for >= EARLY_HINT_SECONDS
    } else {
        speed_fraction >= HINT_SPEED_FRACTION
    }
}

pub(super) const MAX_ENERGY: f32 = 100.0;
pub(super) const HIT_REWARD: f32 = 5.0;
pub(super) const WRONG_PENALTY: f32 = 10.0;
/// Wrong keys never take energy below this, so only collisions can end
/// the game.
pub(super) const LOW_ENERGY: f32 = 20.0;
pub(super) const COLLISION_PENALTY: f32 = 20.0;

pub(super) const FEEDBACK_SECONDS: f32 = 2.5;

const FIRST_LEVEL_POINTS: u32 = 10;
/// How many more points each level needs than the one before...
const LEVEL_POINTS_INCREASE: u32 = 5;
/// ...up to this many, so no level drags on. Later levels get harder
/// through faster spawns, tougher bosses and longer numbers instead.
pub(super) const MAX_LEVEL_POINTS: u32 = 40;
pub(super) const BANNER_SECONDS: f32 = 2.5;
/// No new enemies appear for this long after a level starts.
pub(super) const LEVEL_BREAK_SECONDS: f32 = 2.0;

/// A new enemy's speed.
pub(super) const START_SPEED: f32 = 12.0;
/// Speed an enemy gains per second on screen, up to the player's speed.
const SPEED_GROWTH: f32 = 3.0;
pub(super) const START_SPAWN_INTERVAL: f32 = 4.0;
pub(super) const MIN_SPAWN_INTERVAL: f32 = 1.2;
/// Spawn interval lost per second spent on a level.
const SPAWN_INTERVAL_SHRINK: f32 = 0.05;

pub(super) const BOSS_RADIUS: f32 = 36.0;
/// The first level's boss takes this many hits.
const BOSS_FIRST_HITS: usize = 3;
/// Bosses take one more hit every this many levels.
const BOSS_LEVELS_PER_EXTRA_HIT: u32 = 2;
pub(super) const BOSS_MAX_HITS: usize = 10;
/// The boss moves at this fraction of a normal enemy's speed.
pub(super) const BOSS_SPEED_FACTOR: f32 = 0.6;
/// How far a spell pushes the boss back.
pub(super) const BOSS_HIT_KNOCKBACK: f32 = 50.0;
/// How far the boss bounces back after running into the player.
pub(super) const BOSS_COLLISION_KNOCKBACK: f32 = 220.0;
pub(super) const BOSS_HIT_FLASH_SECONDS: f32 = 0.25;
/// After running into the player, the boss can't hurt her again for this
/// long, even if it is stuck next to her against an edge.
pub(super) const BOSS_HARMLESS_SECONDS: f32 = 1.5;

pub(super) const SPELL_SPEED: f32 = 700.0;
/// How long she keeps her hand raised after casting.
pub(super) const CAST_SECONDS: f32 = 0.35;

pub(super) const KILL_PALETTE: [Color; 4] = [ORANGE, YELLOW, GOLD, WHITE];
pub(super) const SPELL_PALETTE: [Color; 4] = [PINK, MAGENTA, VIOLET, WHITE];
pub(super) const COLLISION_PALETTE: [Color; 3] = [RED, MAROON, ORANGE];

/// How often a new monster uses one of the level's new pairs, when one is
/// free, so new pairs get extra practice.
pub(super) const NEW_PAIR_SHARE: f32 = 0.5;

/// How often a new monster comes out of a portal rather than a screen
/// edge.
pub(super) const PORTAL_SPAWN_SHARE: f32 = 0.75;
/// Monsters don't come out of a portal closer than this to the player.
pub(super) const MIN_PORTAL_SPAWN_DISTANCE: f32 = 200.0;
pub(super) const PORTAL_PALETTE: [Color; 3] = [VIOLET, SKYBLUE, WHITE];

/// A frame longer than this counts as this long, so a stall doesn't make
/// the game jump ahead.
pub(super) const MAX_FRAME_SECONDS: f32 = 0.25;
/// The game advances in steps no longer than this, so a fast spell or
/// monster can't skip past what it should hit however slow a frame is.
pub(super) const MAX_STEP_SECONDS: f32 = 1.0 / 30.0;

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

/// Every enemy starts slow and speeds up as it ages, until it is as fast
/// as the player.
pub fn enemy_speed(age: f32) -> f32 {
    (START_SPEED + SPEED_GROWTH * age).min(PLAYER_SPEED)
}

/// Difficulty comes from the spawn rate: enemies appear more often the
/// longer a level lasts, and a new level starts calm again.
pub fn spawn_interval(level_time: f32) -> f32 {
    (START_SPAWN_INTERVAL - SPAWN_INTERVAL_SHRINK * level_time).max(MIN_SPAWN_INTERVAL)
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

/// How many steps a frame of `dt` seconds is advanced in, and how long
/// each is. A frame is never longer than `MAX_FRAME_SECONDS`.
pub(super) fn frame_steps(dt: f32) -> (u32, f32) {
    let dt = dt.clamp(0.0, MAX_FRAME_SECONDS);
    let steps = (dt / MAX_STEP_SECONDS).ceil().max(1.0) as u32;
    (steps, dt / steps as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_enemy_appears_without_a_hint() {
        for earlier in 0..10 {
            assert!(!shows_hint(earlier, 0.0, enemy_speed(0.0) / PLAYER_SPEED));
        }
    }

    #[test]
    fn first_appearances_of_a_pair_get_a_hint_soon() {
        for earlier in 0..HINTED_APPEARANCES {
            assert!(!shows_hint(earlier, EARLY_HINT_SECONDS - 0.1, 0.0));
            assert!(shows_hint(earlier, EARLY_HINT_SECONDS, 0.0));
        }
    }

    #[test]
    fn later_appearances_get_a_hint_only_when_fast() {
        let earlier = HINTED_APPEARANCES;
        assert!(!shows_hint(earlier, 1000.0, HINT_SPEED_FRACTION - 0.01));
        assert!(shows_hint(earlier, 0.0, HINT_SPEED_FRACTION));
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
        assert_eq!(enemy_speed(0.0), START_SPEED);
        assert!(enemy_speed(10.0) > enemy_speed(0.0));
        assert_eq!(enemy_speed(1000.0), PLAYER_SPEED);
    }

    #[test]
    fn spawns_get_more_frequent_within_a_level_down_to_a_floor() {
        assert_eq!(spawn_interval(0.0), START_SPAWN_INTERVAL);
        assert!(spawn_interval(10.0) < spawn_interval(0.0));
        assert_eq!(spawn_interval(1000.0), MIN_SPAWN_INTERVAL);
    }

    #[test]
    fn a_normal_frame_is_one_step() {
        assert_eq!(frame_steps(1.0 / 60.0), (1, 1.0 / 60.0));
    }

    #[test]
    fn a_long_frame_is_split_into_short_steps() {
        let (steps, step) = frame_steps(0.2);
        assert!(steps > 1);
        assert!(step <= MAX_STEP_SECONDS);
        assert!((steps as f32 * step - 0.2).abs() < 1e-6);
    }

    #[test]
    fn a_stall_counts_as_a_bounded_frame() {
        let (steps, step) = frame_steps(30.0);
        assert!((steps as f32 * step - MAX_FRAME_SECONDS).abs() < 1e-5);
    }

    #[test]
    fn no_time_is_still_one_step() {
        assert_eq!(frame_steps(0.0), (1, 0.0));
    }
}
