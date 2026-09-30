//! A rectangle in the game's virtual units, for laying out screens and
//! finding what a finger touched.

use glam::{Vec2, vec2};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    /// The top left corner.
    pub fn point(&self) -> Vec2 {
        vec2(self.x, self.y)
    }

    pub fn size(&self) -> Vec2 {
        vec2(self.w, self.h)
    }

    pub fn center(&self) -> Vec2 {
        vec2(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Whether `point` is inside, the edges included.
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }

    /// Whether the two share any point; rectangles that only touch along
    /// an edge do.
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.x <= other.right()
            && self.right() >= other.x
            && self.y <= other.bottom()
            && self.bottom() >= other.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rectangle_contains_its_edges_but_nothing_beyond() {
        let r = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert!(r.contains(vec2(10.0, 20.0)));
        assert!(r.contains(vec2(40.0, 60.0)));
        assert!(!r.contains(vec2(40.1, 30.0)));
        assert!(!r.contains(vec2(20.0, 60.1)));
        assert!(!r.contains(vec2(9.9, 30.0)));
    }

    #[test]
    fn rectangles_that_touch_overlap_and_those_apart_do_not() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(a.overlaps(&Rect::new(5.0, 5.0, 10.0, 10.0)));
        assert!(a.overlaps(&Rect::new(10.0, 0.0, 10.0, 10.0)));
        assert!(!a.overlaps(&Rect::new(10.1, 0.0, 10.0, 10.0)));
        assert!(!a.overlaps(&Rect::new(0.0, 10.1, 10.0, 10.0)));
    }

    #[test]
    fn corners_and_center_follow_from_the_size() {
        let r = Rect::new(-5.0, 2.0, 10.0, 6.0);
        assert_eq!((r.right(), r.bottom()), (5.0, 8.0));
        assert_eq!(r.center(), vec2(0.0, 5.0));
        assert_eq!((r.point(), r.size()), (vec2(-5.0, 2.0), vec2(10.0, 6.0)));
    }
}
