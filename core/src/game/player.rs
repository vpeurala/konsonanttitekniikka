//! The heroine: where she is and whether she is walking. A `Player` is a
//! small value; moving her makes a new one.

use glam::Vec2;

use super::rules::{PLAYER_RADIUS, PLAYER_SPEED};
use crate::arena::{ARENA_H, ARENA_W};
use crate::obstacles::{Obstacle, push_out};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Player {
    pub pos: Vec2,
    pub moving: bool,
}

impl Player {
    /// In the middle of the arena, standing still.
    pub fn at_center() -> Player {
        Player {
            pos: Vec2::new(ARENA_W / 2.0, ARENA_H / 2.0),
            moving: false,
        }
    }

    /// Where she is after `dt` seconds of the arrow keys' direction, or
    /// else of the touch joystick's `stick` direction, whose length says
    /// how fast. She stays in the arena and out of the obstacles.
    pub fn moved(self, dt: f32, arrows: Vec2, stick: Vec2, obstacles: &[Obstacle]) -> Player {
        let velocity = if arrows != Vec2::ZERO {
            arrows.normalize()
        } else {
            stick.clamp_length_max(1.0)
        };
        let mut pos = self.pos + velocity * PLAYER_SPEED * dt;
        pos.x = pos.x.clamp(PLAYER_RADIUS, ARENA_W - PLAYER_RADIUS);
        pos.y = pos
            .y
            .clamp(PLAYER_RADIUS * 1.5, ARENA_H - PLAYER_RADIUS * 1.5);
        Player {
            pos: push_out(pos, PLAYER_RADIUS, obstacles),
            moving: velocity != Vec2::ZERO,
        }
    }

    /// She may be standing where a new obstacle appeared.
    pub fn pushed_out(self, obstacles: &[Obstacle]) -> Player {
        Player {
            pos: push_out(self.pos, PLAYER_RADIUS, obstacles),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obstacles::{Kind, Obstacle};

    const DT: f32 = 0.1;

    #[test]
    fn she_walks_along_the_arrows_and_stops_at_the_edge() {
        let start = Player::at_center();
        let moved = start.moved(DT, Vec2::X, Vec2::ZERO, &[]);
        assert!(moved.moving);
        assert!((moved.pos.x - start.pos.x - PLAYER_SPEED * DT).abs() < 1e-3);
        assert_eq!(start, Player::at_center(), "the old value is untouched");
        let mut edge = start;
        for _ in 0..100 {
            edge = edge.moved(DT, Vec2::X, Vec2::ZERO, &[]);
        }
        assert_eq!(edge.pos.x, ARENA_W - PLAYER_RADIUS);
    }

    #[test]
    fn standing_still_is_not_moving() {
        let still = Player::at_center().moved(DT, Vec2::ZERO, Vec2::ZERO, &[]);
        assert!(!still.moving);
        assert_eq!(still.pos, Player::at_center().pos);
    }

    #[test]
    fn the_stick_is_slower_when_pushed_less() {
        let start = Player::at_center();
        let half = start.moved(DT, Vec2::ZERO, Vec2::new(0.5, 0.0), &[]);
        assert!((half.pos.x - start.pos.x - PLAYER_SPEED * DT * 0.5).abs() < 1e-3);
    }

    #[test]
    fn arrows_win_over_the_stick() {
        let start = Player::at_center();
        let moved = start.moved(DT, Vec2::Y, Vec2::new(1.0, 0.0), &[]);
        assert_eq!(moved.pos.x, start.pos.x);
        assert!(moved.pos.y > start.pos.y);
    }

    #[test]
    fn she_is_pushed_out_of_an_obstacle() {
        let start = Player::at_center();
        let stone = Obstacle {
            kind: Kind::Stone,
            pos: start.pos,
            radius: 30.0,
            shape: [0.5; crate::obstacles::OUTLINE_POINTS],
        };
        let out = start.pushed_out(&[stone]);
        assert!(out.pos.distance(start.pos) > 30.0);
    }
}
