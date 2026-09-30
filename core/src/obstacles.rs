//! Stones, trees and lakes that block the way. Each level's obstacles are
//! generated from the level number alone, so they are the same in every
//! game. Every obstacle blocks a circle, which keeps collisions simple and
//! lets anything slide around it.

use glam::{Vec2, vec2};

use crate::rng::{Rng, Stream};

/// Obstacles keep clear of the screen edges, so monsters coming in from
/// an edge are never blocked, and of the HUD at the top.
const MARGIN: f32 = 70.0;
const MARGIN_TOP: f32 = 130.0;
/// The new-pairs panel in the top right corner, at its largest.
const PANEL_LEFT_FROM_RIGHT: f32 = 190.0;
const PANEL_BOTTOM: f32 = 400.0;
/// Room left around the center, where the player starts.
const CENTER_CLEARANCE: f32 = 110.0;
/// Room left around each portal, so monsters can come out of it.
const PORTAL_CLEARANCE: f32 = 60.0;
/// The narrowest gap between two obstacles; wide enough for anyone.
const MIN_GAP: f32 = 80.0;
const MAX_OBSTACLES: usize = 8;
const PLACEMENT_ATTEMPTS: usize = 200;
const LAYOUT_ATTEMPTS: usize = 40;
/// Points on a stone's or lake's outline.
pub const OUTLINE_POINTS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Stone,
    Tree,
    Lake,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Obstacle {
    pub kind: Kind,
    pub pos: Vec2,
    /// The radius of the blocked circle.
    pub radius: f32,
    /// Per-obstacle shape variation in `0.0..1.0`, so no two look alike.
    pub shape: [f32; OUTLINE_POINTS],
}

/// How many obstacles `level` has: none on the first level, then one more
/// per level up to a limit.
pub fn obstacle_count(level: u32) -> usize {
    (level.saturating_sub(1) as usize).min(MAX_OBSTACLES)
}

/// The kinds of obstacle that can appear on `level`: stones at first,
/// then trees, then lakes.
fn kinds(level: u32) -> &'static [Kind] {
    match level {
        0..=2 => &[Kind::Stone],
        3 => &[Kind::Stone, Kind::Tree],
        _ => &[Kind::Stone, Kind::Tree, Kind::Lake],
    }
}

fn radius_range(kind: Kind) -> (f32, f32) {
    match kind {
        Kind::Stone => (20.0, 32.0),
        Kind::Tree => (22.0, 30.0),
        Kind::Lake => (45.0, 70.0),
    }
}

/// The obstacles of `level` on a screen of the given size, clear of the
/// level's `portals`.
pub fn obstacles_for_level(level: u32, width: f32, height: f32, portals: &[Vec2]) -> Vec<Obstacle> {
    let mut rng = Rng::new(Stream::Obstacles, u64::from(level));
    let wanted = obstacle_count(level);
    // Placing obstacles one by one can leave no room for the last ones, so
    // start the layout over when that happens, keeping the fullest one.
    let mut best = Vec::new();
    for _ in 0..LAYOUT_ATTEMPTS {
        let layout = try_layout(&mut rng, level, width, height, portals);
        if layout.len() > best.len() {
            best = layout;
        }
        if best.len() == wanted {
            break;
        }
    }
    best
}

/// One attempt at placing `level`'s obstacles; may place fewer than wanted.
fn try_layout(
    rng: &mut Rng,
    level: u32,
    width: f32,
    height: f32,
    portals: &[Vec2],
) -> Vec<Obstacle> {
    let center = vec2(width / 2.0, height / 2.0);
    let mut obstacles: Vec<Obstacle> = Vec::new();

    for _ in 0..obstacle_count(level) {
        let kind = *rng.pick(kinds(level));
        let (min_r, max_r) = radius_range(kind);
        let radius = rng.range(min_r, max_r);
        let mut shape = [0.0; OUTLINE_POINTS];
        for s in &mut shape {
            *s = rng.unit();
        }

        let (min_x, max_x) = (
            MARGIN + radius,
            (width - MARGIN - radius).max(MARGIN + radius),
        );
        let (min_y, max_y) = (
            MARGIN_TOP + radius,
            (height - MARGIN - radius).max(MARGIN_TOP + radius),
        );
        let fits = |pos: Vec2| {
            let under_panel =
                pos.x + radius > width - PANEL_LEFT_FROM_RIGHT && pos.y - radius < PANEL_BOTTOM;
            !under_panel
                && pos.distance(center) >= CENTER_CLEARANCE + radius
                && portals
                    .iter()
                    .all(|p| p.distance(pos) >= PORTAL_CLEARANCE + radius)
                && obstacles
                    .iter()
                    .all(|o| o.pos.distance(pos) >= o.radius + radius + MIN_GAP)
        };
        let found = (0..PLACEMENT_ATTEMPTS)
            .map(|_| vec2(rng.range(min_x, max_x), rng.range(min_y, max_y)))
            .find(|&pos| fits(pos));
        match found {
            Some(pos) => obstacles.push(Obstacle {
                kind,
                pos,
                radius,
                shape,
            }),
            None => break,
        }
    }
    obstacles
}

/// Moves a circle at `pos` with `radius` out of any obstacle it overlaps.
pub fn push_out(pos: Vec2, radius: f32, obstacles: &[Obstacle]) -> Vec2 {
    let mut pos = pos;
    // Obstacles never overlap, so one pass per obstacle is enough.
    for o in obstacles {
        let offset = pos - o.pos;
        let distance = offset.length();
        let min_distance = o.radius + radius;
        if distance < min_distance {
            let normal = if distance > 0.001 {
                offset / distance
            } else {
                vec2(0.0, -1.0)
            };
            pos = o.pos + normal * min_distance;
        }
    }
    pos
}

/// How far ahead a mover looks for obstacles to steer around.
const LOOK_AHEAD: f32 = 70.0;

/// Bends `desired` (a unit direction) so a circle at `pos` with `radius`
/// curves around obstacles in its path instead of pushing straight into
/// them. With an obstacle dead ahead it turns left if `prefer_left`, else
/// right.
pub fn steer(
    pos: Vec2,
    radius: f32,
    desired: Vec2,
    obstacles: &[Obstacle],
    prefer_left: bool,
) -> Vec2 {
    let mut dir = desired;
    for o in obstacles {
        let to_obstacle = o.pos - pos;
        let distance = to_obstacle.length();
        let clearance = o.radius + radius;
        if distance < 0.001 || distance > clearance + LOOK_AHEAD {
            continue;
        }
        let toward = to_obstacle / distance;
        let ahead = desired.dot(toward);
        // Only obstacles in front whose circle the straight path would hit.
        if ahead <= 0.0 || desired.perp_dot(to_obstacle).abs() >= clearance {
            continue;
        }
        // Go around on the side the path already leans to.
        let sideways = desired - toward * ahead;
        let around = if sideways.length() > 0.05 {
            sideways.normalize()
        } else if prefer_left {
            toward.perp()
        } else {
            -toward.perp()
        };
        let urgency = (1.0 - (distance - clearance) / LOOK_AHEAD).clamp(0.0, 1.0);
        dir = (dir + around * urgency * 2.0).normalize_or(desired);
    }
    dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portals::portal_positions;

    const W: f32 = 800.0;
    const H: f32 = 600.0;

    fn level(level: u32) -> Vec<Obstacle> {
        obstacles_for_level(level, W, H, &portal_positions(level, W, H))
    }

    #[test]
    fn first_level_is_clear_and_second_has_one_obstacle() {
        assert!(level(1).is_empty());
        assert_eq!(level(2).len(), 1);
    }

    #[test]
    fn obstacles_grow_gradually_up_to_a_limit() {
        for l in 1..40 {
            assert!(obstacle_count(l + 1) >= obstacle_count(l));
            assert!(obstacle_count(l + 1) <= obstacle_count(l) + 1);
        }
        assert_eq!(obstacle_count(100), MAX_OBSTACLES);
    }

    #[test]
    fn every_level_fits_all_its_obstacles() {
        for l in 1..40 {
            assert_eq!(level(l).len(), obstacle_count(l), "level {l}");
        }
    }

    #[test]
    fn a_level_always_has_the_same_obstacles() {
        for l in 1..20 {
            assert_eq!(level(l), level(l));
        }
    }

    #[test]
    fn lakes_wait_until_level_four() {
        for l in 1..4 {
            assert!(level(l).iter().all(|o| o.kind != Kind::Lake));
        }
    }

    #[test]
    fn obstacles_leave_room_for_the_player_portals_and_each_other() {
        let center = vec2(W / 2.0, H / 2.0);
        for l in 1..40 {
            let portals = portal_positions(l, W, H);
            let obstacles = level(l);
            for (i, o) in obstacles.iter().enumerate() {
                assert!(o.pos.distance(center) >= CENTER_CLEARANCE + o.radius);
                for p in &portals {
                    assert!(p.distance(o.pos) >= PORTAL_CLEARANCE + o.radius);
                }
                for other in &obstacles[i + 1..] {
                    assert!(o.pos.distance(other.pos) >= o.radius + other.radius + MIN_GAP);
                }
                assert!(o.pos.x - o.radius >= MARGIN && o.pos.x + o.radius <= W - MARGIN);
                assert!(o.pos.y - o.radius >= MARGIN_TOP && o.pos.y + o.radius <= H - MARGIN);
            }
        }
    }

    fn stone_at(pos: Vec2, radius: f32) -> Obstacle {
        Obstacle {
            kind: Kind::Stone,
            pos,
            radius,
            shape: [0.5; OUTLINE_POINTS],
        }
    }

    #[test]
    fn push_out_moves_a_circle_to_the_edge_of_an_obstacle() {
        let obstacles = [stone_at(vec2(100.0, 100.0), 30.0)];
        let pos = push_out(vec2(110.0, 100.0), 10.0, &obstacles);
        assert!((pos.distance(vec2(100.0, 100.0)) - 40.0).abs() < 0.001);
        // Something already clear stays put.
        assert_eq!(
            push_out(vec2(200.0, 100.0), 10.0, &obstacles),
            vec2(200.0, 100.0)
        );
    }

    #[test]
    fn steering_turns_away_from_an_obstacle_dead_ahead() {
        let obstacles = [stone_at(vec2(100.0, 0.0), 30.0)];
        let dir = steer(vec2(40.0, 0.0), 10.0, vec2(1.0, 0.0), &obstacles, true);
        assert!(dir.y.abs() > 0.1, "{dir}");
    }

    #[test]
    fn steering_ignores_obstacles_behind_or_off_the_path() {
        let behind = [stone_at(vec2(-100.0, 0.0), 30.0)];
        assert_eq!(
            steer(Vec2::ZERO, 10.0, vec2(1.0, 0.0), &behind, true),
            vec2(1.0, 0.0)
        );
        let aside = [stone_at(vec2(60.0, 80.0), 20.0)];
        assert_eq!(
            steer(Vec2::ZERO, 10.0, vec2(1.0, 0.0), &aside, true),
            vec2(1.0, 0.0)
        );
    }
}
