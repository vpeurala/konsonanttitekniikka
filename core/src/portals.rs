//! Where each level's portals are. The positions are random but depend
//! only on the level and the screen size, so a level always has its
//! portals in the same places.

use glam::{Vec2, vec2};

use crate::rng::{Rng, Stream};

/// Keeps portals clear of the screen edges and the HUD: the energy bar
/// and score at the top, and the new-pairs panel on the right.
const MARGIN_LEFT: f32 = 80.0;
const MARGIN_RIGHT: f32 = 220.0;
const MARGIN_TOP: f32 = 140.0;
const MARGIN_BOTTOM: f32 = 100.0;
/// Portals stay this far from the center, where the player starts, so
/// they are usable from the first moment (see `MIN_PORTAL_SPAWN_DISTANCE`).
const MIN_CENTER_DISTANCE: f32 = 230.0;
const MIN_PORTAL_DISTANCE: f32 = 220.0;
const PLACEMENT_ATTEMPTS: usize = 100;
const LAYOUT_ATTEMPTS: usize = 50;

/// How many portals `level` has: one up to level 10, two up to level 20,
/// three after that.
pub fn portal_count(level: u32) -> usize {
    match level {
        0..=10 => 1,
        11..=20 => 2,
        _ => 3,
    }
}

/// The portal positions for `level` on a screen of the given size.
pub fn portal_positions(level: u32, width: f32, height: f32) -> Vec<Vec2> {
    let mut rng = Rng::new(Stream::Portals, u64::from(level));
    let center = vec2(width / 2.0, height / 2.0);
    let (min_x, max_x) = (MARGIN_LEFT, (width - MARGIN_RIGHT).max(MARGIN_LEFT));
    let (min_y, max_y) = (MARGIN_TOP, (height - MARGIN_BOTTOM).max(MARGIN_TOP));

    // Placing portals one by one can leave no room for the last one, so
    // start the whole layout over when that happens.
    let mut portals: Vec<Vec2> = Vec::new();
    for _ in 0..LAYOUT_ATTEMPTS {
        portals.clear();
        for _ in 0..portal_count(level) {
            let found = (0..PLACEMENT_ATTEMPTS)
                .map(|_| vec2(rng.range(min_x, max_x), rng.range(min_y, max_y)))
                .find(|&candidate| {
                    candidate.distance(center) >= MIN_CENTER_DISTANCE
                        && portals
                            .iter()
                            .all(|p| p.distance(candidate) >= MIN_PORTAL_DISTANCE)
                });
            match found {
                Some(candidate) => portals.push(candidate),
                None => break,
            }
        }
        if portals.len() == portal_count(level) {
            return portals;
        }
    }
    // On a screen too small for the rules, crowd the rest in anywhere.
    while portals.len() < portal_count(level) {
        portals.push(vec2(rng.range(min_x, max_x), rng.range(min_y, max_y)));
    }
    portals
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f32 = 800.0;
    const H: f32 = 600.0;

    #[test]
    fn portal_counts_grow_with_levels() {
        assert_eq!(portal_count(1), 1);
        assert_eq!(portal_count(10), 1);
        assert_eq!(portal_count(11), 2);
        assert_eq!(portal_count(20), 2);
        assert_eq!(portal_count(21), 3);
        assert_eq!(portal_count(100), 3);
    }

    #[test]
    fn a_level_always_has_the_same_portals() {
        for level in 1..40 {
            assert_eq!(portal_positions(level, W, H), portal_positions(level, W, H));
        }
    }

    #[test]
    fn different_levels_have_different_portals() {
        assert_ne!(portal_positions(1, W, H), portal_positions(2, W, H));
    }

    #[test]
    fn portals_stay_inside_the_margins_and_apart() {
        for level in 1..60 {
            let portals = portal_positions(level, W, H);
            assert_eq!(portals.len(), portal_count(level));
            for (i, p) in portals.iter().enumerate() {
                assert!((MARGIN_LEFT..=W - MARGIN_RIGHT).contains(&p.x), "{p}");
                assert!((MARGIN_TOP..=H - MARGIN_BOTTOM).contains(&p.y), "{p}");
                assert!(p.distance(vec2(W / 2.0, H / 2.0)) >= MIN_CENTER_DISTANCE);
                for other in &portals[i + 1..] {
                    assert!(p.distance(*other) >= MIN_PORTAL_DISTANCE);
                }
            }
        }
    }
}
