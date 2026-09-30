//! The arena the game is played in: a fixed 800 x 600 virtual area, and
//! the geometry of the heroine in it.

use glam::{Vec2, vec2};

pub const ARENA_W: f32 = 800.0;
pub const ARENA_H: f32 = 600.0;

/// Which side of her body faces `toward`: -1.0 for left, 1.0 for right.
pub fn side_toward(pos: Vec2, toward: Vec2) -> f32 {
    if toward.x < pos.x { -1.0 } else { 1.0 }
}

/// Where her raised hand is while casting toward `toward`.
pub fn girl_hand(pos: Vec2, toward: Vec2) -> Vec2 {
    let side = side_toward(pos, toward);
    let shoulder = vec2(pos.x + side * 6.0, pos.y - 3.0);
    shoulder + (toward - shoulder).normalize_or(vec2(side, 0.0)) * 13.0
}
